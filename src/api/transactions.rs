use actix_session::Session;
use actix_web::post;
use actix_web::{error, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use serde::Serialize;

use crate::api::user::get_login_uid;
use crate::models::{Transaction, User};
use crate::BB8Pool;

#[derive(Serialize, Deserialize)]
enum LogQuery {
    UserId(i32),
    ForEveryone,
}

#[post("/log")]
pub async fn get_transactions(
    pool: web::Data<BB8Pool>,
    query: Option<web::Json<LogQuery>>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::transactions;
    use crate::schema::users;

    // Aquire db connection handle
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let mut db_query = transactions::table.into_boxed();

    match query {
        Some(query) => {
            // Prevent access when not running a debug build
            if !cfg!(debug_assertions) {
                return Err(error::ErrorNotFound(
                    "Feature only available in debug builds",
                ));
            };
            match query.0 {
                LogQuery::UserId(uid) => {
                    // Validate user id exists
                    if users::table
                        .filter(users::columns::id.eq(uid))
                        .select(User::as_select())
                        .load(&mut con)
                        .await
                        .map_err(error::ErrorInternalServerError)?
                        .is_empty()
                    {
                        return Err(error::ErrorBadRequest("User not found"));
                    }
                    db_query = db_query.filter(transactions::columns::id.eq(uid));
                }
                LogQuery::ForEveryone => {}
            }
        }
        None => {
            // Get currently logged in users id if no query was provided
            let uid = get_login_uid(&session)?
                .ok_or_else(|| error::ErrorUnauthorized("Not logged in"))?;
            db_query = db_query.filter(transactions::columns::id.eq(uid));
        }
    };

    let transactions_result = db_query
        .select(Transaction::as_select())
        .load(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(transactions_result))
}
