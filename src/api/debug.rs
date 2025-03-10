use actix_web::{error, get, web, Error, HttpResponse};
use diesel_async::RunQueryDsl;
use futures::try_join;

use crate::BB8Pool;

/// Clears the whole database. This endpoint is only accessible in debug builds.
#[get("/debug/db/clear")]
pub async fn clear_db(pool: web::Data<BB8Pool>) -> Result<HttpResponse, Error> {
    use crate::schema::attachments::dsl::*;
    use crate::schema::items::dsl::*;
    use crate::schema::transactions::dsl::*;
    use crate::schema::users::dsl::*;

    // Require debug build
    if !cfg!(debug_assertions) {
        return Err(error::ErrorForbidden(
            "Feature available only in debug builds",
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
