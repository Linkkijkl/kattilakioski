use actix_web::Result;
use log::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{self, Duration};

const CRON_FREQUENCY: usize = 10;

/// Spawns a new task which invokes cron() periodically until `stop` equals true
pub async fn start(stop: Arc<AtomicBool>) -> Result<(), ()> {
    let mut counter: usize = 0;
    let mut interval = time::interval(Duration::from_secs(1));
    while !stop.load(Ordering::Relaxed) {
        counter += 1;
        if counter >= CRON_FREQUENCY {
            if let Err(err) = cron().await {
                error!("Cron error: {err}");
            }
            counter = 0;
        }
        interval.tick().await;
    }

    Ok(())
}

/// The actual cron
async fn cron() -> Result<(), String> {
    println!("Hello world from cron");

    Ok(())
}
