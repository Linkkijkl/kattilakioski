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

/// Returns Ok(true) if session user is admin, Ok(false) if not
pub async fn session_is_admin(session: &Session, pool: web::Data<BB8Pool>) -> Result<bool, Error> {
    use crate::schema::users::dsl::*;

    let uid = if let Ok(Some(uid)) = get_login_uid(session) {
        uid
    } else {
        return Err(error::ErrorBadRequest("Not logged in"));
    };

    // Aquire connection to db
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let result = users
        .filter(id.eq(uid))
        .select(User::as_select())
        .load(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;
    if let [user] = &result[..] {
        Ok(user.is_admin)
    } else {
        Err(error::ErrorBadRequest("User not found"))
    }
}

/// Clears the whole database. This endpoint is only accessible in debug builds.
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

/// Appends given amount of cents to users balance. If the user is not
/// specified, currently logged in user is used. Note that this endpoint
/// can also be used to reduct balance by providing negative values.
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

    // Aquire db connection handle
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

#[derive(Serialize, Deserialize)]
pub struct AdminPromoteQuery {
    pub user_id: i32,
}

/// Promotes given user to admin status. Requires a session with admin
/// level privileges.
#[post("/admin/promote")]
pub async fn promote(
    pool: web::Data<BB8Pool>,
    query: web::Json<AdminPromoteQuery>,
    session: Session,
) -> Result<HttpResponse, Error> {
    use crate::schema::users::dsl::*;

    if !session_is_admin(&session, pool.clone()).await? {
        return Err(error::ErrorForbidden("Insufficent privileges"));
    }

    // Aquire db connection handle
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;

    diesel::update(users)
        .filter(id.eq(query.user_id))
        .set(is_admin.eq(true))
        .execute(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("OK"))
}

#[cfg(test)]
mod tests {
    use reqwest::Result;
    use std::sync::Arc;

    use crate::api::user::UserQuery;

    use super::*;
    const URL: &str = "http://backend:3030";

    // Test admin operations
    #[test]
    fn admin_operations() -> Result<()> {
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

        // Create a new user
        let result = client
            .post(format!("{URL}/api/user/new"))
            .json(&user_query)
            .send()?;
        assert_eq!(result.status(), 200, "Could not create a new user");

        // Admin give currency
        let result = client
            .post(format!("{URL}/api/admin/give"))
            .json(&AdminGiveQuery{amount_cents: 111, user_id: None})
            .send()?;
        assert_eq!(result.status(), 200, "Could not give currency to user via admin give query");

        // Validate that user has the correct amount of currency
        let result = client.post(format!("{URL}/api/user")).send()?;
        let user: User = result.json()?;
        assert_eq!(user.balance_cents, 111, "User didn't get given currency");

        // Promote admin
        let result = client
            .post(format!("{URL}/api/admin/give"))
            .json(&AdminGiveQuery{amount_cents: 111, user_id: None})
            .send()?;
        assert_eq!(result.status(), 200, "Could not promote user to admin status");

        // Validate that user has the correct amount of currency
        let result = client.post(format!("{URL}/api/user")).send()?;
        let user: User = result.json()?;
        assert!(user.is_admin, "User didn't aquire admin status");

        Ok(())
    }
}
