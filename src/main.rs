use std::io::ErrorKind;
use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;

pub type BB8Pool = Pool<diesel_async::AsyncPgConnection>;

mod api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // TODO: More secure logging
    pretty_env_logger::init();

    // Initiate db connection pool
    let diesel_connection_manager =
        AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
            std::env::var("DATABASE_URL").unwrap_or("postgres://postgres".to_string()),
        );
    let diesel_connection_pool: BB8Pool = Pool::builder()
        .build(diesel_connection_manager)
        .await
        .map_err(|_| {
            std::io::Error::new(ErrorKind::ConnectionRefused, "Can't connect to Postgres")
        })?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(diesel_connection_pool.clone()))
            .wrap(
                middleware::DefaultHeaders::new().add(("content-type", "text/html; charset=UTF-8")),
            )
            .wrap(middleware::Logger::new("%t %s %r %Dms"))
            .configure(api::config)
            .service(Files::new("/", "public").index_file("index.html"))
    })
    .bind(("0.0.0.0", 3030))?
    .run()
    .await
}
