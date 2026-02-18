use std::time::Duration;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_sql::{DbInstances, DbPool};

pub fn start_reminder_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            if let Err(e) = check_reminders(&app).await {
                eprintln!("Scheduler error: {}", e);
            }
        }
    });
}

#[derive(Debug, sqlx::FromRow)]
struct ReminderRow {
    id: i64,
    #[allow(dead_code)]
    event_id: String,
    #[allow(dead_code)]
    fire_at: String,
    #[sqlx(rename = "type")]
    #[allow(dead_code)]
    reminder_type: String,
    title: String,
    event_time: String,
}

async fn check_reminders(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let db_instances = match app.try_state::<DbInstances>() {
        Some(instances) => instances,
        None => return Ok(()),
    };

    let now = chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    let instances = db_instances.0.read().await;
    let db_pool = match instances.get("sqlite:desklist.db") {
        Some(pool) => pool,
        None => return Ok(()),
    };

    let pool = match db_pool {
        DbPool::Sqlite(pool) => pool,
    };

    let reminders = sqlx::query_as::<_, ReminderRow>(
        "SELECT rq.id, rq.event_id, rq.fire_at, rq.type, e.title, e.event_time \
         FROM reminder_queue rq \
         JOIN events e ON rq.event_id = e.id \
         WHERE rq.fired = 0 AND rq.fire_at <= ? AND e.completed = 0",
    )
    .bind(&now)
    .fetch_all(pool)
    .await?;

    for reminder in &reminders {
        let time_display = if reminder.event_time.len() >= 16 {
            reminder.event_time[..16].replace("T", " ")
        } else {
            reminder.event_time.clone()
        };

        let _ = app
            .notification()
            .builder()
            .title(&reminder.title)
            .body(&format!("Scheduled: {}", time_display))
            .show();

        sqlx::query("UPDATE reminder_queue SET fired = 1 WHERE id = ?")
            .bind(reminder.id)
            .execute(pool)
            .await?;
    }

    Ok(())
}
