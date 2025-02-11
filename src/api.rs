use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::{MultipartForm, MultipartFormConfig};
use actix_session::Session;
use actix_web::post;
use actix_web::{error, get, web, Error, HttpResponse};
use async_fs::File;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use futures::{try_join, AsyncWriteExt};
use image::{ImageReader, Limits};
use itertools::Itertools;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use std::sync::LazyLock;

use crate::models::User;
use crate::models::{Attachment, Item};
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
    Err(_) => {
        // Allow use of development values only in debug builds
        if !cfg!(debug_assertions) {
            panic!("Environment variable SALT not set! Password salt is required for security.");
        }
        "defaultsalt".to_string()
    }
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

#[post("/user/new")]
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

#[post("/user/login")]
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

#[derive(Serialize, Deserialize)]
enum GetUserQuery {
    Username(String),
    UserId(i32),
}

#[post("/user")]
pub async fn user_info(
    pool: web::Data<BB8Pool>,
    query: Option<web::Json<GetUserQuery>>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::users::dsl::*;

    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    // Aquire a valid user id
    let uid: i32 = match query {
        Some(query) => {
            // Get valid user id for query
            let mut db_query = users.into_boxed();
            match query.0 {
                GetUserQuery::UserId(user_id) => {
                    // Validate user id if one was provided
                    db_query = db_query.filter(id.eq(user_id));
                }
                GetUserQuery::Username(uname) => {
                    // Get a valid user id for username if one was provided
                    db_query = db_query.filter(username.eq(uname));
                }
            };
            let result = db_query
                .select(User::as_select())
                .load(&mut con)
                .await
                .map_err(error::ErrorInternalServerError)?;
            match &result[..] {
                [user] => user.id,
                _ => return Err(error::ErrorBadRequest("User not found")),
            }
        }
        None => {
            // Get currently logged in users id if no query was provided
            get_login_uid(&session)?.ok_or_else(|| error::ErrorUnauthorized("Not logged in"))?
        }
    };

    let user = users
        .filter(id.eq(uid))
        .select(User::as_select())
        .get_result(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(user))
}

#[get("/admin/db/clear")]
pub async fn clear_db(pool: web::Data<BB8Pool>) -> Result<HttpResponse, Error> {
    use crate::schema::attachments::dsl::*;
    use crate::schema::items::dsl::*;
    use crate::schema::transactions::dsl::*;
    use crate::schema::users::dsl::*;

    // Prevent access when not running a debug build
    if !cfg!(debug_assertions) {
        return Err(error::ErrorNotFound(
            "Feature only available in debug builds",
        ));
    }

    // Aquire db connection handle
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    // Remove everything ( in correct order! )
    try_join!(
        diesel::delete(attachments).execute(&mut con),
        diesel::delete(transactions).execute(&mut con)
    )
    .map_err(error::ErrorInternalServerError)?;
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
    get_items_without_stock: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct ItemResult {
    #[serde(flatten)]
    item: Item,
    attachments: Vec<Attachment>,
}

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
struct NewItemQuery {
    title: String,
    description: String,
    amount: usize,
    price: String,
    attachments: Vec<i32>,
}

// Parses string of form 1.23 to number like 123, see tests
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
                let seller_id = item.seller_id; // The relation guarantees that seller exists if the item pointing to it does

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

#[derive(Debug, MultipartForm)]
struct UploadForm {
    file: TempFile,
}

#[post("/attachment/upload")]
pub async fn upload(
    pool: web::Data<BB8Pool>,
    session: Session,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<HttpResponse, Error> {
    use crate::schema::attachments;

    const PUBLIC_DIR: &str = "public"; // TODO: make configurable
    const EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];
    const RANDOM_FILE_NAME_LENGTH: usize = 20;
    const MAX_IMAGE_RESOLUTION: u32 = 10_000;
    const THUMBNAIL_SIZE: u32 = 320;
    const THUMBNAIL_QUALITY: f32 = 50.0;

    let temp_file = form.file;

    // Validate login
    let user_id =
        get_login_uid(&session)?.ok_or_else(|| error::ErrorUnauthorized("Not logged in"))?;

    // Parse file extension
    let file_name = temp_file
        .file_name
        .ok_or_else(|| error::ErrorBadRequest("No file name provided with file"))?;
    let extension = Path::new(&file_name)
        .extension()
        .ok_or_else(|| error::ErrorBadRequest("File name does not contain extension"))?
        .to_string_lossy()
        .into_owned();
    if !EXTENSIONS.contains(&extension.as_str()) {
        return Err(error::ErrorBadRequest(format!(
            "Bad file extension. Accepted extensions are: {:?}",
            EXTENSIONS
        )));
    }

    // Generate new file name with path
    let (file_path, thumbnail_path) = loop {
        let id = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(RANDOM_FILE_NAME_LENGTH)
            .map(char::from)
            .collect::<String>();
        let path = format!("{PUBLIC_DIR}/{id}.{extension}");
        if (File::open(&path).await).is_err() {
            break (path, format!("{PUBLIC_DIR}/{id}.thumb.webp"));
        }
    };

    // Prevent loading "zip bomb" images before the image is decoded in memory
    let mut decoder = ImageReader::open(temp_file.file.path())
        .map_err(error::ErrorInternalServerError)?
        .with_guessed_format()
        .map_err(error::ErrorInternalServerError)?;
    let mut limits = Limits::default();
    limits.max_alloc = Some(512 * 1024 * 1024); /* 512 MiB */
    limits.max_image_height = Some(MAX_IMAGE_RESOLUTION);
    limits.max_image_width = Some(MAX_IMAGE_RESOLUTION);
    decoder.limits(limits);

    // Generate thumbnail for image
    let img = decoder.decode().map_err(|_| {
        error::ErrorBadRequest(format!(
            "Could not decode {file_name}. Uploaded image might be too large or corrupted."
        ))
    })?;
    let thumbnail = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    let thumbnail_bytes = webp::Encoder::from_image(&thumbnail)
        .unwrap()
        .encode(THUMBNAIL_QUALITY);
    let mut file = File::create(&thumbnail_path)
        .await
        .map_err(error::ErrorInternalServerError)?;
    file.write_all(&thumbnail_bytes)
        .await
        .map_err(error::ErrorInternalServerError)?;
    file.flush()
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Persist provided image file when everything above has passed without errors
    async_fs::copy(temp_file.file.path(), &file_path)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Index image to db
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let attachment = diesel::insert_into(attachments::table)
        .values((
            attachments::columns::file_path.eq(file_path),
            attachments::columns::thumbnail_path.eq(thumbnail_path),
            attachments::columns::uploader_id.eq(user_id),
            attachments::columns::uploaded_at.eq(chrono::offset::Utc::now()),
        ))
        .returning(Attachment::as_returning())
        .get_result(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Return info of newly created attachment
    Ok(HttpResponse::Ok().json(attachment))
}

/// Service config for the whole api
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
            .service(buy_item)
            .app_data(
                MultipartFormConfig::default()
                    .total_limit(10 * 1024 * 1024) // 10MiB maximum file upload size
                    .memory_limit(256), // Allow for almost no memory usage
            )
            .service(upload),
    );
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use image::ImageBuffer;
    use rand::random;
    use reqwest::Result;
    use temp_dir::TempDir;

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
            .post(format!("{URL}/api/user/login"))
            .json(&user_query)
            .send()?;
        assert_ne!(
            result.status(),
            200,
            "Login returned ok status with bad user info"
        );

        // Create a new user
        let result = client
            .post(format!("{URL}/api/user/new"))
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

        // Get user info without a valid session
        let result = client.get(format!("{URL}/api/user/info")).send()?;
        assert_ne!(
            result.status(),
            200,
            "User info returned wrong status without valid session"
        );

        // Log out without a valid session
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
        let result = client.get(format!("{URL}/api/debug/db/clear")).send()?;
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

        // Test user balances
        let result = client.get(format!("{URL}/api/user/info")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(user.balance_cents, 333, "User has unexpected balance");

        let result = client2.get(format!("{URL}/api/user/info")).send()?;
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
        let result = client.get(format!("{URL}/api/user/info")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(user.balance_cents, 333 - 250, "User has unexpected balance");

        let result = client2.get(format!("{URL}/api/user/info")).send()?;
        let user: TestUserQuery = result.json()?;
        assert_eq!(
            user.balance_cents,
            250 - 111 * 2,
            "User has unexpected balance"
        );

        Ok(())
    }

    // Test attachment uploading
    #[test]
    fn attachment_operations() -> Result<()> {
        // Set things up for testing
        let cookie_provider = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_provider(cookie_provider.clone())
            .build()?;
        let cookie_provider2 = Arc::new(reqwest::cookie::Jar::default());
        let client2 = reqwest::blocking::ClientBuilder::new()
            .cookie_provider(cookie_provider2.clone())
            .build()?;
        let temp_dir = TempDir::new().unwrap();

        // Generate test images
        let random_image_generator = || {
            ImageBuffer::from_fn(2000, 1500, |_, _| {
                let a = || random::<u8>() % 255_u8;
                image::Rgb([a(), a(), a()])
            })
        };
        let image = random_image_generator();
        let attachment_path = temp_dir.child("test_image.png");
        image.save(&attachment_path).unwrap();
        let image2 = random_image_generator();
        let attachment_path2 = temp_dir.child("test_image_2.jpg");
        image2.save(&attachment_path2).unwrap();

        // Clear database for testing
        let result = client.get(format!("{URL}/api/debug/db/clear")).send()?;
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

        // Upload attachments
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", attachment_path)
            .unwrap();
        let result = client
            .post(format!("{URL}/api/attachment/upload"))
            .multipart(form)
            .send()?;
        assert_eq!(result.status(), 200, "Could not upload attachment");
        let attachment_id = result.json::<Attachment>().unwrap().id;

        let form2 = reqwest::blocking::multipart::Form::new()
            .file("file", attachment_path2)
            .unwrap();
        let result = client2
            .post(format!("{URL}/api/attachment/upload"))
            .multipart(form2)
            .send()?;
        assert_eq!(
            result.status(),
            200,
            "Could not upload attachment for second user"
        );
        let attachment_id2 = result.json::<Attachment>().unwrap().id;

        // Sell an item with uploaded attachment
        let result = client
            .post(format!("{URL}/api/item/new"))
            .json(&NewItemQuery {
                title: "test item".to_string(),
                description: "test description".to_string(),
                amount: 3,
                price: "1.11".to_string(),
                attachments: vec![attachment_id],
            })
            .send()?;
        assert_eq!(
            result.status(),
            200,
            "Could not create new item with attachment"
        );

        // Try to sell an item with attachment beloning to another user
        let result = client
            .post(format!("{URL}/api/item/new"))
            .json(&NewItemQuery {
                title: "test item".to_string(),
                description: "test description".to_string(),
                amount: 1,
                price: "1,00".to_string(),
                attachments: vec![attachment_id2],
            })
            .send()?;
        assert_ne!(
            result.status(),
            200,
            "Could use attachment which is not owned"
        );

        // Try to re-use attachment
        let result = client
            .post(format!("{URL}/api/item/new"))
            .json(&NewItemQuery {
                title: "test item".to_string(),
                description: "test description".to_string(),
                amount: 3,
                price: "1.11".to_string(),
                attachments: vec![attachment_id],
            })
            .send()?;
        assert_ne!(
            result.status(),
            200,
            "Could use the same attachment in 2 different items"
        );

        Ok(())
    }
}
