use std::sync::LazyLock;
use actix_session::Session;
use actix_web::{error, get, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use serde::Deserialize;

use crate::models::User;
use crate::BB8Pool;

const LOGGED_IN_KEY: &str = "logged_in";

#[derive(Deserialize)]
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
static SALT: LazyLock<String> = LazyLock::new(|| {
    match std::env::var("SALT") {
        Ok(val) => val,
        Err(_) => "defaultsalt".to_string(),
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
        return Err(error::ErrorBadRequest("No whitespace allowed in username!"));
    }

    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let password = hash(&query.password);

    // Insert into db
    let user = diesel::insert_into(crate::schema::users::table)
        .values((username.eq(&query.username), password_hash.eq(password)))
        .returning(User::as_returning())
        .get_result(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

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

    let uid = get_login_uid(&session)?.ok_or(error::ErrorUnauthorized("Not logged in"))?;
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let user = users
        .filter(id.eq(uid))
        .select(User::as_select())
        .get_result(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(user))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(hello_world)
            .service(login)
            .service(logout)
            .service(user_info)
            .service(new_user),
    );
}
