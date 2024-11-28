use actix_files::Files;
use actix_session::config::{CookieContentSecurity, PersistentSession};
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::{middleware, web, App, HttpServer};
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use std::io::ErrorKind;

pub type BB8Pool = Pool<diesel_async::AsyncPgConnection>;

mod api;
mod models;
mod schema;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // TODO: More secure logging
    pretty_env_logger::init();

    // Cookie session middleware vars
    let secret_key_str = std::env::var("SESSION_SECRET")
        .unwrap_or("sessionsecretsecretsecretsecretsecret".to_string());
    let cookie_secret_key = Key::derive_from(secret_key_str.as_bytes());
    const COOKIE_TTL: Duration = Duration::days(7);

    // Initiate db connection pool
    let diesel_connection_manager =
        AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
            std::env::var("DATABASE_URL")
                .unwrap_or("postgres://postgres:mysecretpassword@postgres".to_string()),
        );
    let diesel_connection_pool: BB8Pool = Pool::builder()
        .build(diesel_connection_manager)
        .await
        .map_err(|_| {
            std::io::Error::new(
                ErrorKind::ConnectionRefused,
                "Could not connect to Postgres",
            )
        })?;

    HttpServer::new(move || {
        let cookie_middleware =
            SessionMiddleware::builder(CookieSessionStore::default(), cookie_secret_key.clone())
                .session_lifecycle(PersistentSession::default().session_ttl(COOKIE_TTL))
                .cookie_content_security(CookieContentSecurity::Private)
                // Don't use secure cookies in debug builds as they require ssl
                .cookie_secure(!cfg!(debug_assertions))
                .build();

        App::new()
            .app_data(web::Data::new(diesel_connection_pool.clone()))
            .wrap(
                middleware::DefaultHeaders::new().add(("content-type", "text/html; charset=UTF-8")),
            )
            .wrap(cookie_middleware)
            .wrap(middleware::Logger::new("%t %s %r %Dms"))
            .configure(api::config)
            .service(Files::new("/", "public").index_file("index.html"))
    })
    .bind(("0.0.0.0", 3030))?
    .run()
    .await
}
