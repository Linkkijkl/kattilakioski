use actix_session::Session;
use actix_web::post;
use actix_web::{error, get, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use futures::try_join;
use serde::Deserialize;
use serde::Serialize;

use crate::api::user::get_login_uid;
use crate::models::User;
use crate::BB8Pool;

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
pub struct AdminGiveQuery {
    pub user_id: Option<i32>,
    pub amount_cents: i32,
}

#[post("/admin/give")]
pub async fn give_balance(
    pool: web::Data<BB8Pool>,
    query: web::Json<AdminGiveQuery>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::users::dsl::*;

    // Prevent access when not running a debug build
    if !cfg!(debug_assertions) {
        return Err(error::ErrorNotFound(
            "Feature only available in debug builds",
        ));
    }

    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let uid: i32 = match query.user_id {
        Some(user_id) => {
            // Validate provided user id
            let result = users
                .filter(id.eq(user_id))
                .select(User::as_select())
                .load(&mut con)
                .await
                .map_err(error::ErrorInternalServerError)?;
            if let [_] = &result[..] {
                user_id
            } else {
                return Err(error::ErrorBadRequest("User not found"));
            }
        }
        None => {
            // Get currently logged in users id if no query was provided
            get_login_uid(&session)?.ok_or_else(|| error::ErrorUnauthorized("Not logged in"))?
        }
    };

    diesel::update(users)
        .filter(id.eq(uid))
        .set(balance_cents.eq(balance_cents + query.amount_cents))
        .execute(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("OK"))
}
