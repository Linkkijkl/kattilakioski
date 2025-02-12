use actix_multipart::form::MultipartFormConfig;
use actix_web::{get, web, Error, HttpResponse};

pub mod user;
pub mod item;
pub mod attachment;
pub mod transactions;
pub mod admin;

#[get("/hello")]
pub async fn hello_world() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

/// Service config for the whole api
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(hello_world)
            .service(user::login)
            .service(user::logout)
            .service(user::user_info)
            .service(user::new_user)
            .service(admin::clear_db)
            .service(admin::give_balance)
            .service(item::get_items)
            .service(item::new_item)
            .service(item::buy_item)
            .service(transactions::get_transactions)
            .app_data(
                MultipartFormConfig::default()
                    .total_limit(10 * 1024 * 1024) // 10MiB maximum file upload size
                    .memory_limit(256), // Allow for almost no memory usage
            )
            .service(attachment::upload),
    );
}

#[cfg(test)]
mod tests {
    use reqwest::Result;
    const URL: &str = "http://backend:3030";

    // Test if api is available
    #[test]
    fn api_is_responsive() -> Result<()> {
        let status = reqwest::blocking::get(format!("{URL}/api/hello"))?.status();
        assert_eq!(status, 200, "Could not reach API. Make sure the backend is running and available in `{URL}` before running tests.");
        Ok(())
    }
}
