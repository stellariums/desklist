use chrono::{Local, TimeZone, Utc};
use std::time::Duration;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;

use crate::db::DatabaseState;

pub fn start_reminder_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            if let Err(e) = check_reminders(&app).await {
                #[cfg(debug_assertions)]
                eprintln!("Scheduler error: {}", e);
                #[cfg(not(debug_assertions))]
                let _ = e;
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
    let state = match app.try_state::<DatabaseState>() {
        Some(state) => state,
        None => return Ok(()),
    };

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    let pool = match state.pool().await {
        Ok(pool) => pool,
        Err(_) => return Ok(()),
    };

    let reminders = sqlx::query_as::<_, ReminderRow>(
        "SELECT rq.id, rq.event_id, rq.fire_at, rq.type, e.title, e.event_time \
         FROM reminder_queue rq \
         JOIN events e ON rq.event_id = e.id \
         WHERE rq.fired = 0 AND rq.fire_at <= ? AND e.completed = 0",
    )
    .bind(&now)
    .fetch_all(&pool)
    .await?;

    for reminder in &reminders {
        let time_display = if let Ok(utc_time) = chrono::NaiveDateTime::parse_from_str(
            &reminder.event_time.replace("Z", ""),
            "%Y-%m-%dT%H:%M:%S%.f",
        ) {
            let utc_dt = Utc.from_utc_datetime(&utc_time);
            let local_dt = utc_dt.with_timezone(&Local);
            local_dt.format("%Y-%m-%d %H:%M").to_string()
        } else {
            reminder.event_time.clone()
        };

        let _ = app
            .notification()
            .builder()
            .title(&reminder.title)
            .body(&format!("Scheduled: {}", time_display))
            .sound("Default")
            .show();

        sqlx::query("UPDATE reminder_queue SET fired = 1 WHERE id = ?")
            .bind(reminder.id)
            .execute(&pool)
            .await?;
    }

    Ok(())
}
