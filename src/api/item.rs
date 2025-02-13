use actix_session::Session;
use actix_web::post;
use actix_web::{error, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use futures::try_join;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use crate::api::user::get_login_uid;
use crate::models::{Attachment, Item, User};
use crate::BB8Pool;

#[derive(Serialize, Deserialize)]
struct ItemQuery {
    search_term: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
    get_items_without_stock: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct ItemResult {
    #[serde(flatten)]
    item: Item,
    attachments: Vec<Attachment>,
}

/// Lists items for sale. The endpoint can filter results by a search
/// term and get results regardless of if they are in stock. It can also
/// limit the amount of returned items and skip an amount of items from
/// being returned, which can be used to implement pages in the frontend.
#[post("/item/list")]
pub async fn get_items(
    pool: web::Data<BB8Pool>,
    query: Option<web::Json<ItemQuery>>,
) -> Result<HttpResponse, Error> {
    use crate::schema::items::dsl::*;

    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    // Default values
    let mut offset = 0;
    let mut limit = 20;
    let mut minimum_stock = 1;

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
        if let Some(true) = &query.get_items_without_stock {
            minimum_stock = 0;
        }
    }

    // Query db and return results
    let item_result = db_query
        .offset(offset)
        .limit(limit)
        .select(Item::as_select())
        .filter(amount.ge(minimum_stock))
        .load(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    let attachments_result = Attachment::belonging_to(&item_result)
        .select(Attachment::as_select())
        .load(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    let attachments_per_item = attachments_result
        .grouped_by(&item_result)
        .into_iter()
        .zip(item_result)
        .map(|(attachments, item)| ItemResult { item, attachments })
        .collect::<Vec<ItemResult>>();

    Ok(HttpResponse::Ok().json(attachments_per_item))
}

#[derive(Serialize, Deserialize)]
pub struct NewItemQuery {
    pub title: String,
    pub description: String,
    pub amount: usize,
    pub price: String,
    pub attachments: Vec<i32>,
}

/// Parses string of form 1.23 to number like 123, see tests
fn parse_decimal_to_cents(string: String) -> Result<u32, ()> {
    let split: Vec<&str> = string.split(['.', ',']).collect();
    let flattened = split.concat();
    let splits = split.len();
    if (splits >= 2 && split[1].len() > 2) || !flattened.chars().all(char::is_numeric) {
        return Err(());
    }
    let exp = if splits < 2 {
        2
    } else {
        2 - split[1].len() as u32
    };
    let mult = 10_u32.pow(exp);
    let cents = flattened.parse::<u32>().map_err(|_| ())? * mult;
    Ok(cents)
}

/// Enlists a new item for sale.
#[post("/item/new")]
pub async fn new_item(
    pool: web::Data<BB8Pool>,
    query: web::Json<NewItemQuery>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::{attachments, items};

    // Limits
    const MAX_TITLE_LENGTH: usize = 50;
    const MAX_DESCRIPTION_LENGTH: usize = 500;
    const MAX_ITEM_AMOUNT: usize = 50;
    const MAX_PRICE_CENTS: u32 = 15_00;
    const MIN_PRICE_CENTS: u32 = 1;
    const MAX_ATTACHMENTS: usize = 5;

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
    if !(MIN_PRICE_CENTS..=MAX_PRICE_CENTS).contains(&item_price_cents) {
        return Err(error::ErrorBadRequest(format!(
            "Price must be at least {MIN_PRICE_CENTS} cents and at most {MAX_PRICE_CENTS} cents"
        )));
    }
    let item_price_cents = item_price_cents as i32;

    // Deduplicate attachments
    let item_attachments: Vec<i32> = query.attachments.iter().unique().cloned().collect();

    if item_attachments.len() > MAX_ATTACHMENTS {
        return Err(error::ErrorBadRequest(format!(
            "Amount of attachments can be at most {MAX_ATTACHMENTS}"
        )));
    }

    // Aquire db connection hande only when needed, to avoid aquiring it for no use on bad user input
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    // Validate referenced attachments
    let referenced_attachments = attachments::table
        .select(Attachment::as_select())
        .filter(attachments::columns::id.eq_any(&item_attachments))
        .filter(attachments::columns::uploader_id.eq(&user_id)) // Validate ownership
        .filter(attachments::columns::item_id.is_null()) // Validate that attachment is not already bound to an item
        .load(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    if referenced_attachments.len() != item_attachments.len() {
        let missing_attachments = referenced_attachments
            .iter()
            .filter(|a| !item_attachments.contains(&a.id))
            .map(|a| &a.id)
            .join(", ");
        return Err(error::ErrorBadRequest(format!("Following attachments could not be used: {missing_attachments}. Try uploading them again.")));
    }

    // Insert item into db
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

    // Reference attachments to item
    let attachments = diesel::update(attachments::table)
        .filter(attachments::columns::id.eq_any(item_attachments))
        .set(attachments::columns::item_id.eq(item.id))
        .get_results(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(ItemResult { attachments, item }))
}

#[derive(Serialize, Deserialize)]
struct BuyQuery {
    item_id: i32,
    amount: Option<i32>,
}

/// Buys an item.
#[post("/item/buy")]
pub async fn buy_item(
    pool: web::Data<BB8Pool>,
    query: web::Json<BuyQuery>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::{items, transactions, users};

    // Gather and validate input
    let buyer_id =
        get_login_uid(&session)?.ok_or_else(|| error::ErrorUnauthorized("Not logged in"))?;
    let item_id = query.item_id;
    let item_amount = query.amount.unwrap_or(1);

    // Aquire db connection handle
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

                // Same for both parties of the transaction
                let users = users::table
                    .filter(users::columns::id.eq(buyer_id))
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
                let seller_id = item.seller_id; // Relation guarantees that the seller exists if the item referring to it does

                // All checks ok, make the transaction
                try_join!(
                    // Remove items from stock
                    diesel::update(items::table)
                        .filter(items::columns::id.eq(item_id))
                        .set(items::columns::amount.eq(items::columns::amount - item_amount))
                        .execute(con),
                    // Remove balance from the buyers account
                    diesel::update(users::table)
                        .filter(users::columns::id.eq(buyer_id))
                        .set(
                            users::columns::balance_cents
                                .eq(users::columns::balance_cents - total_price)
                        )
                        .execute(con),
                    // Append balance to the sellers account
                    diesel::update(users::table)
                        .filter(users::columns::id.eq(seller_id))
                        .set(
                            users::columns::balance_cents
                                .eq(users::columns::balance_cents + total_price)
                        )
                        .execute(con),
                    // Log transaction
                    diesel::insert_into(transactions::table)
                        .values((
                            transactions::columns::item_id.eq(item_id), // Seller id is deductible from this
                            transactions::columns::buyer_id.eq(buyer_id),
                            transactions::columns::item_amount.eq(item_amount),
                            transactions::columns::transacted_at.eq(chrono::offset::Utc::now()),
                        ))
                        .execute(con),
                )?;

                Ok(Ok(()))
            })
        })
        .await;

    // Propagate errors from transaction
    result
        .map_err(error::ErrorInternalServerError)? // Error executing the transaction
        .map_err(error::ErrorBadRequest)?; // Error from inside of the transaction

    // If we got here the transaction was a success
    Ok(HttpResponse::Ok().body("OK"))
}

#[cfg(test)]
mod tests {
    use reqwest::Result;
    use std::sync::Arc;

    use crate::api::{admin::AdminGiveQuery, user::UserQuery};

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

    // Test selling and buying
    #[test]
    fn item_operations() -> Result<()> {
        // Set things up for testing
        let cookie_provider = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_provider(cookie_provider.clone())
            .build()?;
        let cookie_provider2 = Arc::new(reqwest::cookie::Jar::default());
        let client2 = reqwest::blocking::ClientBuilder::new()
            .cookie_provider(cookie_provider2.clone())
            .build()?;

        // Clear database for testing
        let result = client.get(format!("{URL}/api/admin/db/clear")).send()?;
        assert_eq!(
            result.status(),
            200,
            "Could not clear db. Make sure the server is compiled in debug mode."
        );

        // Register test users and log them in to their clients
        let result = client
            .post(format!("{URL}/api/user/new"))
            .json(&UserQuery {
                username: "test".to_string(),
                password: "test".to_string(),
            })
            .send()?;
        assert_eq!(result.status(), 200, "Could not create a new user");

        let result = client2
            .post(format!("{URL}/api/user/new"))
            .json(&UserQuery {
                username: "test2".to_string(),
                password: "test".to_string(),
            })
            .send()?;
        assert_eq!(result.status(), 200, "Could not create a second user");

        // Test selling items
        let result = client
            .post(format!("{URL}/api/item/new"))
            .json(&NewItemQuery {
                title: "test item".to_string(),
                description: "test description".to_string(),
                amount: 3,
                price: "1.11".to_string(),
                attachments: Vec::new(),
            })
            .send()?;
        let item: Item = result.json()?;
        assert_eq!(item.price_cents, 111, "Could not create new item for sale");

        let result = client2
            .post(format!("{URL}/api/item/new"))
            .json(&NewItemQuery {
                title: "the best item".to_string(),
                description: "the best item description".to_string(),
                amount: 1,
                price: "2,5".to_string(),
                attachments: Vec::new(),
            })
            .send()?;
        let item2: Item = result.json()?;

        assert_eq!(
            item2.price_cents, 250,
            "Could not create new item for sale from second user"
        );

        #[derive(Deserialize)]
        struct TestUserQuery {
            balance_cents: u32,
        }

        // Test giving currency
        let result = client
            .post(format!("{URL}/api/admin/give"))
            .json(&AdminGiveQuery {
                user_id: Some(item.seller_id),
                amount_cents: 333,
            })
            .send()?;
        assert_eq!(result.status(), 200, "Could not add balance to user");

        let result = client2
            .post(format!("{URL}/api/admin/give"))
            .json(&AdminGiveQuery {
                user_id: Some(item2.seller_id),
                amount_cents: 250,
            })
            .send()?;
        assert_eq!(result.status(), 200, "Could not add balance to user");

        let result = client.post(format!("{URL}/api/user")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(user.balance_cents, 333, "User has unexpected balance");

        let result = client2.post(format!("{URL}/api/user")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(user.balance_cents, 250, "User has unexpected balance");

        // Item listing
        let result = client.post(format!("{URL}/api/item/list")).send()?;
        let items: Vec<Item> = result.json()?;
        assert_eq!(
            items.len(),
            2,
            "Item query resulted in incorrect amount of available items"
        );

        // Item searching
        let result = client
            .post(format!("{URL}/api/item/list"))
            .json(&ItemQuery {
                search_term: Some("best".to_string()),
                limit: None,
                offset: None,
                get_items_without_stock: None,
            })
            .send()?;
        let items: Vec<Item> = result.json()?;
        let got_item = &items[0];
        assert_eq!(*got_item, item2);

        // Item buying
        let result = client
            .post(format!("{URL}/api/item/buy"))
            .json(&BuyQuery {
                item_id: item2.id,
                amount: Some(1),
            })
            .send()?;
        assert_eq!(result.status(), 200, "Could not buy an item");

        let result = client2
            .post(format!("{URL}/api/item/buy"))
            .json(&BuyQuery {
                item_id: item.id,
                amount: Some(2),
            })
            .send()?;
        assert_eq!(result.status(), 200, "Second user could not buy an item");

        // Are user balances correct after buying
        let result = client.post(format!("{URL}/api/user")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(
            user.balance_cents,
            333 - 250 + 111 * 2,
            "User has unexpected balance"
        );

        let result = client2.post(format!("{URL}/api/user")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(
            user.balance_cents,
            250 - 111 * 2 + 250,
            "User has unexpected balance"
        );

        Ok(())
    }
}
