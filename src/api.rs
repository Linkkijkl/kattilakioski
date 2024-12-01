use actix_session::Session;
use actix_web::{error, get, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
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
pub async fn hello_world(_pool: web::Data<BB8Pool>) -> Result<HttpResponse, Error> {
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
    use crate::schema::users::dsl::*;

    // Prevent access when not running a debug build
    if !cfg!(debug_assertions) {
        return Err(error::ErrorNotFound(
            "Feature only available in debug builds",
        ));
    }

    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;
    diesel::delete(users)
        .execute(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("OK"))
}

#[derive(Deserialize)]
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
    let mut search_term = "".to_string();
    let mut offset = 0;
    let mut limit = 20;

    // Limits
    const SEARCH_MAX_LENGTH: usize = 50;
    const OFFSET_MIN: i64 = 0;
    const LIMIT_CONSTRAINTS: (i64, i64) = (1, 100);

    // Overwrite default values with ones provided in item query
    if let Some(query) = &query {
        if let Some(val) = &query.search_term {
            if val.len() > SEARCH_MAX_LENGTH {
                return Err(error::ErrorBadRequest("Search term too long"));
            }
            search_term = val.to_owned();
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
    let result = items
        .filter(title.ilike(search_term))
        .offset(offset + 1) // Translate to dns indexes which start from 1
        .limit(limit)
        .select(Item::as_select())
        .get_result(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize)]
struct NewItemQuery {
    title: String,
    description: String,
    amount: usize,
    price: String,
}

fn parse_decimal_to_cents(string: String) -> Result<usize, ()> {
    let split: Vec<&str> = string.split(['.', ',']).collect();
    let flattened = split.concat();
    if split.len() != 2
        || split[0].is_empty()
        || split[1].len() != 2
        || !flattened.chars().all(char::is_numeric)
    {
        return Err(());
    }
    let cents = flattened.parse::<usize>().map_err(|_| ())?;
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

    // Aquire db connection hande only when needed, to avoid aquiring it in vein on bad user input
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
        .set(
            users::columns::balance_cents
                .eq(users::columns::balance_cents + item_price_cents * item_amount),
        )
        .execute(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(item))
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
            .service(new_item),
    );
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use reqwest::Result;

    use super::*;
    const URL: &str = "http://backend:3030";

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
        // Set things up for testing
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
}
