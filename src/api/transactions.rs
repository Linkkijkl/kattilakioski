use actix_session::Session;
use actix_web::post;
use actix_web::{error, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use futures::try_join;
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

/// Returns a transaction log for a given user. If no user is provided,
/// the endpoint will use the user currently logged in. It's possible
/// to list logs for everyone, but only in debug builds.
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

#[derive(Serialize, Deserialize)]
pub struct TransferQuery {
    amount_cents: i32,
    recipient: String,
}

/// Transfers money from the logged in user to another user
#[post("/transfer")]
pub async fn transfer(
    pool: web::Data<BB8Pool>,
    query: web::Json<TransferQuery>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::{transactions, users};

    // Gather and validate input
    let transactor_id =
        get_login_uid(&session)?.ok_or_else(|| error::ErrorUnauthorized("Not logged in"))?;

    // Aquire db connection handle
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let result: Result<Result<(), &str>, diesel::result::Error> = con
        .transaction(move |con| {
            Box::pin(async move {
                // Fetch both users from db and perform checks
                let users = users::table
                    .filter(users::columns::id.eq(transactor_id))
                    .select(User::as_select())
                    .load(con)
                    .await?;
                let transactor = match &users[..] {
                    [user] => user,
                    _ => return Ok(Err("Your user does not exist")),
                };
                if transactor.balance_cents < query.amount_cents {
                    return Ok(Err("Insufficient funds"));
                }
                let users = users::table
                    .filter(users::columns::username.eq(&query.recipient))
                    .select(User::as_select())
                    .load(con)
                    .await?;
                let recipient = match &users[..] {
                    [user] => user,
                    _ => return Ok(Err("Recipient does not exist")),
                };

                // All checks ok, make the transaction
                try_join!(
                    // Take balance from seller
                    diesel::update(users::table)
                        .filter(users::columns::id.eq(transactor_id))
                        .set(
                            users::columns::balance_cents
                                .eq(users::columns::balance_cents - query.amount_cents)
                        )
                        .execute(con),
                    // Append balance to recipient
                    diesel::update(users::table)
                        .filter(users::columns::id.eq(recipient.id))
                        .set(
                            users::columns::balance_cents
                                .eq(users::columns::balance_cents + query.amount_cents)
                        )
                        .execute(con),
                    // Log transaction
                    diesel::insert_into(transactions::table)
                        .values((
                            transactions::columns::payer_id.eq(transactor.id),
                            transactions::columns::receiver_id.eq(recipient.id),
                            transactions::columns::transacted_at.eq(chrono::offset::Utc::now()),
                            transactions::columns::amount_cents.eq(query.amount_cents)
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
