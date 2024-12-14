use actix_web::Result;
use diesel::prelude::*;
use log::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{self, Duration};

use crate::models::Attachment;
use crate::BB8Pool;

const CRON_FREQUENCY: usize = 10;

/// Spawns a new task which invokes cron() periodically until `stop` equals true
pub async fn start(stop_flag: Arc<AtomicBool>, pool: BB8Pool) -> Result<(), ()> {
    let mut counter: usize = 0;
    let mut interval = time::interval(Duration::from_secs(1));
    while !stop_flag.load(Ordering::Relaxed) {
        counter += 1;
        if counter >= CRON_FREQUENCY {
            if let Err(err) = cron(pool.clone()).await {
                error!("Cron error: {err}");
            }
            counter = 0;
        }
        interval.tick().await;
    }

    Ok(())
}

/// The actual cron
async fn cron(pool: BB8Pool) -> Result<(), String> {
    use crate::schema::attachments::dsl::*;

    // Aquire connection to db
    let mut con = pool.get().await.map_err(|a| a.to_string())?;

    // Remove old enough attachments not bound to any item
    const DANGLING_ATTACHMENT_TIMEOUT: u64 = 60 * 10;
    let now = chrono::offset::Utc::now();
    let oldest_accepted_timestamp = now - Duration::from_secs(DANGLING_ATTACHMENT_TIMEOUT);

    let removed_db_rows: Vec<Attachment> = diesel_async::RunQueryDsl::get_results(
        diesel::delete(attachments)
            .filter(item_id.is_null())
            .filter(uploaded_at.lt(oldest_accepted_timestamp)),
        &mut con,
    )
    .await
    .map_err(|s| s.to_string())?;

    // Remove associated files
    for row in &removed_db_rows {
        for attachment_file in [&row.file_path, &row.thumbnail_path] {
            async_fs::remove_file(attachment_file)
                .await
                .map_err(|s| s.to_string())?;
        }
    }

    let removed_amount = removed_db_rows.len();
    if removed_amount > 0 {
        info!("Cleaned {removed_amount} attachments without associated items");
    }

    Ok(())
}
