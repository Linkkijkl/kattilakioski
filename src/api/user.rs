use actix_session::Session;
use actix_web::post;
use actix_web::{error, get, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use serde::Serialize;
use std::sync::LazyLock;

use crate::models::User;
use crate::BB8Pool;

const LOGGED_IN_KEY: &str = "logged_in";

#[derive(Serialize, Deserialize)]
pub struct UserQuery {
    pub username: String,
    pub password: String,
}

pub fn get_login_uid(session: &Session) -> Result<Option<i32>, Error> {
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

#[cfg(test)]
mod tests {
    use reqwest::Result;
    use std::sync::Arc;

    use super::*;
    const URL: &str = "http://backend:3030";

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
        let result = client.get(format!("{URL}/api/admin/db/clear")).send()?;
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
        let result = client.post(format!("{URL}/api/user")).send()?;
        assert_eq!(result.status(), 200, "Could not get user info");

        // Log user out
        let result = client.get(format!("{URL}/api/user/logout")).send()?;
        assert_eq!(result.status(), 200, "Could not log out");

        // Get user info without a valid session
        let result = client.post(format!("{URL}/api/user")).send()?;
        assert_ne!(
            result.status(),
            200,
            "User info returned wrong status without a valid session"
        );

        // Log out without a valid session
        let result = client.get(format!("{URL}/api/user/logout")).send()?;
        assert_ne!(
            result.status(),
            200,
            "Logout returned wrong status without a valid session"
        );

        Ok(())
    }
}
