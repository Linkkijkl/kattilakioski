use actix_files::Files;
use actix_session::config::{CookieContentSecurity, PersistentSession};
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::{middleware, web, App, HttpServer};
use async_fs::DirBuilder;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use diesel::pg::PgConnection;
use futures_util::StreamExt;
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM, TERM_SIGNALS};
use signal_hook::flag;
use signal_hook_tokio::Signals;
use std::io::ErrorKind;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use diesel_migrations::MigrationHarness;
use diesel::Connection;

pub type BB8Pool = Pool<diesel_async::AsyncPgConnection>;

mod api;
mod cron;
mod models;
mod schema;

/// Run database migrations
fn run_migrations(db_url: &str) {
    // Running migrations async proved to be really hard, so lets just do it sync
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
    let mut connection = PgConnection::establish(db_url).expect("Could not connect to database");
    connection.run_pending_migrations(MIGRATIONS).expect("Could not run database migrations");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> { // TODO: Rework main function error handling
    // TODO: More secure logging
    pretty_env_logger::init();

    // Generate data directories if they don't exist
    let _ = DirBuilder::new().create("./public").await;

    // Cookie session middleware vars
    let secret_key_str = std::env::var("SESSION_SECRET")
        .unwrap_or("sessionsecretsecretsecretsecretsecret".to_string());
    let cookie_secret_key = Key::derive_from(secret_key_str.as_bytes());
    const COOKIE_TTL: Duration = Duration::days(7);

    // Run pending db migrations
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:mysecretpassword@postgres".to_string());
    run_migrations(&db_url);

    // Initiate db connection pool
    let diesel_connection_manager =
        AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(&db_url);
    let diesel_connection_pool: BB8Pool = Pool::builder()
        .build(diesel_connection_manager)
        .await
        .map_err(|_| {
            std::io::Error::new(
                ErrorKind::ConnectionRefused,
                "Could not connect to Postgres",
            )
        })?;

    // Bind terminating signals to set flag true
    let stop_flag = Arc::new(AtomicBool::new(false));
    for signal in TERM_SIGNALS {
        let _ = flag::register(*signal, Arc::clone(&stop_flag));
    }

    // Spawn cron task
    let cron = tokio::task::spawn(cron::start(Arc::clone(&stop_flag), diesel_connection_pool.clone()));

    // Spawn actix server task
    let actix = tokio::task::spawn(
        HttpServer::new(move || {
            let cookie_middleware = SessionMiddleware::builder(
                CookieSessionStore::default(),
                cookie_secret_key.clone(),
            )
            .session_lifecycle(PersistentSession::default().session_ttl(COOKIE_TTL))
            .cookie_content_security(CookieContentSecurity::Private)
            // Don't use secure cookies in debug builds as they require use of https, which would complicate testing
            .cookie_secure(!cfg!(debug_assertions))
            .build();
            let headers_middleware =
                middleware::DefaultHeaders::new().add(("content-type", "text/html; charset=UTF-8"));
            let logger_middleware = middleware::Logger::new("%t %s %r %Dms");

            App::new()
                .app_data(web::Data::new(diesel_connection_pool.clone()))
                .wrap(headers_middleware)
                .wrap(cookie_middleware)
                .wrap(logger_middleware)
                .configure(api::config)
                .service(Files::new("/", "public").index_file("index.html"))
        })
        .bind(("0.0.0.0", 3030))?
        .run(),
    );

    // Wait for terminating signals
    let mut signals = Signals::new(TERM_SIGNALS)?;
    while let Some(signal) = signals.next().await {
        match signal {
            SIGINT | SIGTERM | SIGQUIT => {
                break;
            }
            _ => continue,
        }
    }

    // Wait for all tasks to return
    let (a, b) = tokio::join!(actix, cron);
    a.unwrap().unwrap();
    b.unwrap().unwrap();
    Ok(())
}
