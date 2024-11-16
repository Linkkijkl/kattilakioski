use crate::BB8Pool;
use actix_web::{get, web, Error, HttpResponse};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(hello_world)
    );
}

#[get("/hello")]
pub async fn hello_world(_pool: web::Data<BB8Pool>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello World!"))
}
