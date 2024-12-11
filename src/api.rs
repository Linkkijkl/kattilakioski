use actix_session::Session;
use actix_web::{error, get, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use futures::try_join;
use serde::Deserialize;
use serde::Serialize;
use std::sync::LazyLock;

use crate::models::Item;
use crate::models::User;
use crate::BB8Pool;

const LOGGED_IN_KEY: &str = "logged_in";

#[derive(Serialize, Deserialize)]
struct UserQuery {
    username: String,
    password: String,
}

fn get_login_uid(session: &Session) -> Result<Option<i32>, Error> {
    session
        .get::<i32>(LOGGED_IN_KEY)
        .map_err(error::ErrorInternalServerError)
}

fn set_login_uid(session: &Session, uid: i32) -> Result<(), Error> {
    session
        .insert(LOGGED_IN_KEY, uid)
        .map_err(error::ErrorInternalServerError)
}

// Get salt from environment on first access
static SALT: LazyLock<String> = LazyLock::new(|| match std::env::var("SALT") {
    Ok(val) => val,
    Err(_) => "defaultsalt".to_string(),
});

// Hash password with salt
fn hash(pass: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SALT.as_bytes());
    hasher.update(pass.as_bytes());
    hasher.finalize().to_string()
}

#[get("/hello")]
pub async fn hello_world() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello World!"))
}

#[get("/user/new")]
pub async fn new_user(
    pool: web::Data<BB8Pool>,
    query: web::Json<UserQuery>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::users::dsl::*;

    // Validate username
    let whitespace = query.username.as_str().split_whitespace().count();
    if whitespace > 1 {
        return Err(error::ErrorBadRequest("No whitespace allowed in username"));
    }

    // Aquire a connection hande to db
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    // Check if username is already registered
    let results = users
        .filter(username.eq(&query.username))
        .limit(1)
        .select(User::as_select())
        .load(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    if !results.is_empty() {
        return Err(error::ErrorBadRequest("User already registered"));
    }

    let password = hash(&query.password);

    // Insert into db
    let user = diesel::insert_into(crate::schema::users::table)
        .values((
            username.eq(&query.username),
            password_hash.eq(password),
            created_at.eq(chrono::offset::Utc::now()),
        ))
        .returning(User::as_returning())
        .get_result(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Log new user in
    set_login_uid(&session, user.id)?;
    Ok(HttpResponse::Ok().body("OK"))
}

#[get("/user/login")]
pub async fn login(
    pool: web::Data<BB8Pool>,
    query: web::Json<UserQuery>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::users::dsl::*;

    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let results = users
        .filter(username.eq(&query.username))
        .limit(1)
        .select(User::as_select())
        .load(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    if let [user] = &results[..] {
        if user.password_hash == hash(&query.password) {
            set_login_uid(&session, user.id)?;
            return Ok(HttpResponse::Ok().body("OK"));
        }
    }

    Err(error::ErrorUnauthorized("Incorrect login"))
}

#[get("/user/logout")]
pub async fn logout(session: Session) -> Result<HttpResponse, Error> {
    if session.remove(LOGGED_IN_KEY).is_some() {
        Ok(HttpResponse::Ok().body("OK"))
    } else {
        Err(error::ErrorUnauthorized("Not logged in"))
    }
}

#[get("/user/info")]
pub async fn user_info(pool: web::Data<BB8Pool>, session: Session) -> Result<HttpResponse, Error> {
    use crate::schema::users::dsl::*;

    let uid = get_login_uid(&session)?.ok_or_else(|| error::ErrorUnauthorized("Not logged in"))?;
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let user = users
        .filter(id.eq(uid))
        .select(User::as_select())
        .get_result(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(user))
}

#[get("/debug/db/clear")]
pub async fn clear_db(pool: web::Data<BB8Pool>) -> Result<HttpResponse, Error> {
    use crate::schema::items::dsl::*;
    use crate::schema::users::dsl::*;

    // Prevent access when not running a debug build
    if !cfg!(debug_assertions) {
        return Err(error::ErrorNotFound(
            "Feature only available in debug builds",
        ));
    }

    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    // Remove everything ( in correct order! )
    diesel::delete(items)
        .execute(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    diesel::delete(users)
        .execute(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("OK"))
}

#[derive(Serialize, Deserialize)]
struct ItemQuery {
    search_term: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
}

#[get("/item/list")]
pub async fn get_items(
    pool: web::Data<BB8Pool>,
    query: Option<web::Json<ItemQuery>>,
) -> Result<HttpResponse, Error> {
    use crate::schema::items::dsl::*;

    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    // Default values
    let mut offset = 0;
    let mut limit = 20;

    // Limits
    const SEARCH_MAX_LENGTH: usize = 50;
    const OFFSET_MIN: i64 = 0;
    const LIMIT_CONSTRAINTS: (i64, i64) = (1, 100);

    let mut db_query = items.into_boxed();

    // Overwrite default values with ones provided in item query
    if let Some(query) = &query {
        if let Some(search_term) = &query.search_term {
            if search_term.len() > SEARCH_MAX_LENGTH {
                return Err(error::ErrorBadRequest("Search term too long"));
            }

            // Surround search term with wildmarks and escape accidental (or not) wildmarks provided by user
            let escaped: String = search_term
                .chars()
                .flat_map(|c| match c {
                    '%' => vec!['\\', '%'],
                    '\\' => vec!['\\', '\\'],
                    c => vec![c],
                })
                .collect();
            let wildmarked = format!("%{escaped}%");
            db_query = db_query.filter(title.ilike(wildmarked));
        }
        if let Some(val) = &query.offset {
            if *val < OFFSET_MIN {
                return Err(error::ErrorBadRequest(format!(
                    "Offset must be at least {OFFSET_MIN}"
                )));
            }
            offset = val.to_owned();
        }
        if let Some(val) = &query.limit {
            if *val < LIMIT_CONSTRAINTS.0 || *val > LIMIT_CONSTRAINTS.1 {
                return Err(error::ErrorBadRequest(format!(
                    "Limit must be at least {} and at max {}",
                    LIMIT_CONSTRAINTS.0, LIMIT_CONSTRAINTS.1
                )));
            }
            limit = val.to_owned();
        }
    }

    // Query db and return results
    let result = db_query
        .offset(offset)
        .limit(limit)
        .select(Item::as_select())
        .load(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(Serialize, Deserialize)]
struct NewItemQuery {
    title: String,
    description: String,
    amount: usize,
    price: String,
}

// Parses string of form 1.23 to number like 123, see tests
fn parse_decimal_to_cents(string: String) -> Result<usize, ()> {
    let split: Vec<&str> = string.split(['.', ',']).collect();
    let flattened = split.concat();
    let splits = split.len();
    if (splits >= 2 && split[1].len() > 2) || !flattened.chars().all(char::is_numeric) {
        return Err(());
    }
    let exp = if splits < 2 { 2 } else { 2 - split[1].len() };
    let mult = 10_usize.pow(exp as u32);
    let cents = flattened.parse::<usize>().map_err(|_| ())? * mult;
    Ok(cents)
}

#[get("/item/new")]
pub async fn new_item(
    pool: web::Data<BB8Pool>,
    query: web::Json<NewItemQuery>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::{items, users};

    // Limits
    const MAX_TITLE_LENGTH: usize = 50;
    const MAX_DESCRIPTION_LENGTH: usize = 500;
    const MAX_ITEM_AMOUNT: usize = 50;
    const MAX_PRICE_CENTS: usize = 15_00;
    const MIN_PRICE_CENTS: usize = 1;

    // Gather and validate input

    let user_id =
        get_login_uid(&session)?.ok_or_else(|| error::ErrorUnauthorized("Not logged in"))?;

    let item_title = query.title.trim().to_string();
    if item_title.len() > MAX_TITLE_LENGTH {
        return Err(error::ErrorBadRequest(format!(
            "Title can be at most {MAX_TITLE_LENGTH} characters long"
        )));
    }

    let item_description = query.description.trim().to_string();
    if item_description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(error::ErrorBadRequest(format!(
            "Description can be at most {MAX_DESCRIPTION_LENGTH} characters long"
        )));
    }

    let item_amount = query.amount;
    if !(1..MAX_ITEM_AMOUNT).contains(&item_amount) {
        return Err(error::ErrorBadRequest(format!(
            "Amount must be at least 1 and at most {MAX_ITEM_AMOUNT}"
        )));
    }
    let item_amount = item_amount as i32;

    let item_price_cents = parse_decimal_to_cents(query.price.clone()).map_err(|_| {
        error::ErrorBadRequest("Price must be in decimal format with cents, i.e 9.95")
    })?;
    if !(MIN_PRICE_CENTS..MAX_PRICE_CENTS).contains(&item_price_cents) {
        return Err(error::ErrorBadRequest(format!(
            "Price must be at least {MIN_PRICE_CENTS} cents and at most {MAX_PRICE_CENTS} cents"
        )));
    }
    let item_price_cents = item_price_cents as i32;

    // Aquire db connection hande only when needed, to avoid aquiring it for no use on bad user input
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    // Insert into db
    let item = diesel::insert_into(items::table)
        .values((
            items::columns::title.eq(item_title),
            items::columns::description.eq(item_description),
            items::columns::amount.eq(item_amount),
            items::columns::price_cents.eq(item_price_cents),
            items::columns::seller_id.eq(user_id),
            items::columns::created_at.eq(chrono::offset::Utc::now()),
        ))
        .returning(Item::as_returning())
        .get_result(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Add appropriate amount of cents to sellers balance
    diesel::update(users::table)
        .filter(users::columns::id.eq(user_id))
        .set(
            users::columns::balance_cents
                .eq(users::columns::balance_cents + item_price_cents * item_amount),
        )
        .execute(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(item))
}

#[derive(Serialize, Deserialize)]
struct BuyQuery {
    item_id: i32,
    amount: Option<i32>,
}

#[get("/item/buy")]
pub async fn buy_item(
    pool: web::Data<BB8Pool>,
    query: web::Json<BuyQuery>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::{items, users};

    // Gather and validate input
    let user_id =
        get_login_uid(&session)?.ok_or_else(|| error::ErrorUnauthorized("Not logged in"))?;
    let item_id = query.item_id;
    let item_amount = query.amount.unwrap_or(1);

    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    // Run the whole buy operation inside a transaction to prevent double spending
    let result: Result<Result<(), &str>, diesel::result::Error> = con
        .transaction(move |con| {
            Box::pin(async move {
                // Fetch item from db and perform checks
                let items = items::table
                    .filter(items::columns::id.eq(item_id))
                    .select(Item::as_select())
                    .load(con)
                    .await?;
                let item = match &items[..] {
                    [item] => item,
                    _ => return Ok(Err("Item not found")),
                };
                if item.amount < item_amount {
                    return Ok(Err("Not enough item in stock"));
                }
                let total_price = item_amount * item.price_cents;

                // Same for user
                let users = users::table
                    .filter(users::columns::id.eq(user_id))
                    .select(User::as_select())
                    .load(con)
                    .await?;
                let user = match &users[..] {
                    [user] => user,
                    _ => return Ok(Err("Your user does not exist")), // Weird but possible using 2 sessions and deleting users account from one
                };
                if user.balance_cents < total_price {
                    return Ok(Err("You don't have enough balance on your account"));
                }

                // All checks ok, do the transaction
                try_join!(
                    // Update item
                    diesel::update(items::table)
                        .filter(items::columns::id.eq(item_id))
                        .set(items::columns::amount.eq(items::columns::amount - item_amount))
                        .execute(con),
                    // Update user
                    diesel::update(users::table)
                        .filter(users::columns::id.eq(user_id))
                        .set(
                            users::columns::balance_cents
                                .eq(users::columns::balance_cents - total_price)
                        )
                        .execute(con)
                )?;

                Ok(Ok(()))
            })
        })
        .await;

    // Propagate errors from transaction
    result
        .map_err(error::ErrorInternalServerError)?
        .map_err(error::ErrorBadRequest)?;

    // If we got here, the transaction was a success
    Ok(HttpResponse::Ok().body("OK"))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(hello_world)
            .service(login)
            .service(logout)
            .service(user_info)
            .service(new_user)
            .service(clear_db)
            .service(get_items)
            .service(new_item)
            .service(buy_item),
    );
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use reqwest::Result;

    use super::*;
    const URL: &str = "http://backend:3030";

    // Currency parsing tests
    #[test]
    fn currency_formatter_works() {
        assert_eq!(parse_decimal_to_cents("0.01".to_string()), Ok(1));
        assert_eq!(parse_decimal_to_cents("5,1".to_string()), Ok(510));
        assert_eq!(parse_decimal_to_cents("1".to_string()), Ok(100));
        assert_eq!(parse_decimal_to_cents("1,".to_string()), Ok(100));
        assert_eq!(parse_decimal_to_cents("1.12".to_string()), Ok(112));

        assert_eq!(parse_decimal_to_cents("0.001".to_string()), Err(()));
        assert_eq!(parse_decimal_to_cents("1.123".to_string()), Err(()));
        assert_eq!(parse_decimal_to_cents("5€".to_string()), Err(()));
        assert_eq!(parse_decimal_to_cents("-5.14,".to_string()), Err(()));
    }

    // Test if api is available
    #[test]
    fn api_is_responsive() -> Result<()> {
        let status = reqwest::blocking::get(format!("{URL}/api/hello"))?.status();
        assert_eq!(status, 200, "Could not reach API. Make sure server is running and available in `{URL}` before running tests.");
        Ok(())
    }

    // Test account operations
    #[test]
    fn account_operations() -> Result<()> {
        // Set variables and objects up for testing
        let cookie_provider = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_provider(cookie_provider.clone())
            .build()?;
        let user_query = UserQuery {
            username: "test".to_string(),
            password: "test".to_string(),
        };

        // Clear database for testing
        let result = client.get(format!("{URL}/api/debug/db/clear")).send()?;
        assert_eq!(
            result.status(),
            200,
            "Could not clear db. Make sure the server is compiled in debug mode."
        );

        // Log in with nonexsistent user
        let result = client
            .get(format!("{URL}/api/user/login"))
            .json(&user_query)
            .send()?;
        assert_ne!(
            result.status(),
            200,
            "Login returned ok status with bad user info"
        );

        // Create a new user
        let result = client
            .get(format!("{URL}/api/user/new"))
            .json(&user_query)
            .send()?;
        assert_eq!(result.status(), 200, "Could not create a new user");

        // Inspect cookies
        // println!("{:?}", cookie_provider);

        // Get user info
        let result = client.get(format!("{URL}/api/user/info")).send()?;
        assert_eq!(result.status(), 200, "Could not get user info");

        // Log user out
        let result = client.get(format!("{URL}/api/user/logout")).send()?;
        assert_eq!(result.status(), 200, "Could not log out");

        // Get user info without a session
        let result = client.get(format!("{URL}/api/user/info")).send()?;
        assert_ne!(
            result.status(),
            200,
            "User info returned wrong status without valid session"
        );

        // Log out without a session
        let result = client.get(format!("{URL}/api/user/logout")).send()?;
        assert_ne!(
            result.status(),
            200,
            "Logout returned wrong status without valid session"
        );

        Ok(())
    }

    // Test selling and buying
    #[test]
    fn item_operations() -> Result<()> {
        // Set variables and objects up for testing
        let cookie_provider = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_provider(cookie_provider.clone())
            .build()?;
        let cookie_provider2 = Arc::new(reqwest::cookie::Jar::default());
        let client2 = reqwest::blocking::ClientBuilder::new()
            .cookie_provider(cookie_provider2.clone())
            .build()?;

        // Clear database for testing
        let result = client.get(format!("{URL}/api/debug/db/clear")).send()?;
        assert_eq!(
            result.status(),
            200,
            "Could not clear db. Make sure the server is compiled in debug mode."
        );

        // Register test users and log them in to their clients
        let result = client
            .get(format!("{URL}/api/user/new"))
            .json(&UserQuery {
                username: "test".to_string(),
                password: "test".to_string(),
            })
            .send()?;
        assert_eq!(result.status(), 200, "Could not create a new user");

        let result = client2
            .get(format!("{URL}/api/user/new"))
            .json(&UserQuery {
                username: "test2".to_string(),
                password: "test".to_string(),
            })
            .send()?;
        assert_eq!(result.status(), 200, "Could not create a second user");

        // Test selling items
        let result = client
            .get(format!("{URL}/api/item/new"))
            .json(&NewItemQuery {
                title: "test item".to_string(),
                description: "test description".to_string(),
                amount: 3,
                price: "1.11".to_string(),
            })
            .send()?;
        let item: Item = result.json()?;
        assert_eq!(item.price_cents, 111, "Could not create new item for sale");

        let result = client2
            .get(format!("{URL}/api/item/new"))
            .json(&NewItemQuery {
                title: "the best item".to_string(),
                description: "the best item description".to_string(),
                amount: 1,
                price: "2,5".to_string(),
            })
            .send()?;
        let item2: Item = result.json()?;

        assert_eq!(
            item2.price_cents, 250,
            "Could not create new item for sale from second user"
        );

        #[derive(Deserialize)]
        struct TestUserQuery {
            balance_cents: usize,
        }

        // Test user balances
        let result = client.get(format!("{URL}/api/user/info")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(user.balance_cents, 333, "User has unexpected balance");

        let result = client2.get(format!("{URL}/api/user/info")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(user.balance_cents, 250, "User has unexpected balance");

        // Item listing
        let result = client.get(format!("{URL}/api/item/list")).send()?;
        let items: Vec<Item> = result.json()?;
        assert_eq!(
            items.len(),
            2,
            "Item query resulted in incorrect amount of available items"
        );

        // Item searching
        let result = client
            .get(format!("{URL}/api/item/list"))
            .json(&ItemQuery {
                search_term: Some("best".to_string()),
                limit: None,
                offset: None,
            })
            .send()?;
        let items: Vec<Item> = result.json()?;
        println!("{:?}", items);
        let got_item = items.get(0).unwrap();
        assert_eq!(*got_item, item2);

        // Item buying
        let result = client
            .get(format!("{URL}/api/item/buy"))
            .json(&BuyQuery {
                item_id: item2.id,
                amount: Some(1),
            })
            .send()?;
        assert_eq!(result.status(), 200, "Could not buy an item");

        let result = client2
            .get(format!("{URL}/api/item/buy"))
            .json(&BuyQuery {
                item_id: item.id,
                amount: Some(2),
            })
            .send()?;
        assert_eq!(result.status(), 200, "Second user could not buy an item");

        // Are user balances correct after buying
        let result = client.get(format!("{URL}/api/user/info")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(user.balance_cents, 333 - 250, "User has unexpected balance");

        let result = client2.get(format!("{URL}/api/user/info")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(user.balance_cents, 250 - 111 * 2, "User has unexpected balance");

        Ok(())
    }
}
