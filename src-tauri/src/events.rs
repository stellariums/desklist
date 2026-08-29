use chrono::{DateTime, Datelike, Days, Months, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Sqlite, SqlitePool, Transaction};
use tauri::State;

use crate::db::DatabaseState;

const VALID_RECURRENCES: [&str; 4] = ["none", "daily", "weekly", "monthly"];

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DeskEvent {
    pub id: String,
    pub title: String,
    pub description: String,
    pub event_time: String,
    pub completed: i64,
    pub remind_at: Option<String>,
    pub remind_on_time: i64,
    pub recurrence: String,
    pub recurrence_end: Option<String>,
    pub generated_next_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct EventInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub event_time: String,
    pub remind_at: Option<String>,
    #[serde(default = "default_remind_on_time")]
    pub remind_on_time: i64,
    #[serde(default = "default_recurrence")]
    pub recurrence: String,
    pub recurrence_end: Option<String>,
}

fn default_remind_on_time() -> i64 {
    1
}

fn default_recurrence() -> String {
    "none".to_string()
}

#[tauri::command]
pub async fn fetch_events(
    state: State<'_, DatabaseState>,
    filter: String,
    today_start: Option<String>,
    today_end: Option<String>,
) -> Result<Vec<DeskEvent>, String> {
    let pool = state.pool().await?;
    let rows = match filter.as_str() {
        "today" => {
            let start = today_start.ok_or_else(|| "缺少今天的开始时间".to_string())?;
            let end = today_end.ok_or_else(|| "缺少今天的结束时间".to_string())?;
            sqlx::query_as::<_, DeskEvent>(
                "SELECT * FROM events WHERE event_time >= ? AND event_time <= ? AND completed = 0 ORDER BY event_time ASC",
            )
            .bind(start)
            .bind(end)
            .fetch_all(&pool)
            .await
        }
        "upcoming" => {
            sqlx::query_as::<_, DeskEvent>(
                "SELECT * FROM events WHERE completed = 0 ORDER BY event_time ASC",
            )
            .fetch_all(&pool)
            .await
        }
        "completed" => {
            sqlx::query_as::<_, DeskEvent>(
                "SELECT * FROM events WHERE completed = 1 ORDER BY updated_at DESC",
            )
            .fetch_all(&pool)
            .await
        }
        _ => {
            sqlx::query_as::<_, DeskEvent>("SELECT * FROM events ORDER BY event_time ASC")
                .fetch_all(&pool)
                .await
        }
    };
    rows.map_err(|error| format!("读取任务失败：{error}"))
}

#[tauri::command]
pub async fn fetch_month_events(
    state: State<'_, DatabaseState>,
    month_start: String,
    month_end: String,
) -> Result<Vec<DeskEvent>, String> {
    let pool = state.pool().await?;
    sqlx::query_as::<_, DeskEvent>(
        "SELECT * FROM events WHERE event_time >= ? AND event_time <= ? AND completed = 0 ORDER BY event_time ASC",
    )
    .bind(month_start)
    .bind(month_end)
    .fetch_all(&pool)
    .await
    .map_err(|error| format!("读取日历失败：{error}"))
}

#[tauri::command]
pub async fn create_event(
    state: State<'_, DatabaseState>,
    event: EventInput,
) -> Result<String, String> {
    let pool = state.pool().await?;
    create_event_in_pool(&pool, event).await
}

async fn create_event_in_pool(pool: &SqlitePool, event: EventInput) -> Result<String, String> {
    validate_input(&event)?;
    let mut transaction = pool.begin().await.map_err(db_error)?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_iso();
    let recurrence = safe_recurrence(&event.recurrence);

    sqlx::query(
        "INSERT INTO events (id, title, description, event_time, completed, remind_at, remind_on_time, recurrence, recurrence_end, generated_next_id, created_at, updated_at) VALUES (?, ?, ?, ?, 0, ?, ?, ?, ?, NULL, ?, ?)",
    )
    .bind(&id)
    .bind(event.title.trim())
    .bind(event.description.trim())
    .bind(&event.event_time)
    .bind(&event.remind_at)
    .bind(event.remind_on_time)
    .bind(recurrence)
    .bind(&event.recurrence_end)
    .bind(&now)
    .bind(&now)
    .execute(&mut *transaction)
    .await
    .map_err(db_error)?;

    generate_reminders(
        &mut transaction,
        &id,
        &event.event_time,
        event.remind_at.as_deref(),
        event.remind_on_time,
        None,
    )
    .await?;
    transaction.commit().await.map_err(db_error)?;
    Ok(id)
}

#[tauri::command]
pub async fn update_event(
    state: State<'_, DatabaseState>,
    id: String,
    event: EventInput,
) -> Result<(), String> {
    let pool = state.pool().await?;
    update_event_in_pool(&pool, &id, event).await
}

async fn update_event_in_pool(
    pool: &SqlitePool,
    id: &str,
    event: EventInput,
) -> Result<(), String> {
    validate_input(&event)?;
    let mut transaction = pool.begin().await.map_err(db_error)?;
    let now = now_iso();
    let recurrence = safe_recurrence(&event.recurrence);

    let result = sqlx::query(
        "UPDATE events SET title = ?, description = ?, event_time = ?, remind_at = ?, remind_on_time = ?, recurrence = ?, recurrence_end = ?, updated_at = ? WHERE id = ?",
    )
    .bind(event.title.trim())
    .bind(event.description.trim())
    .bind(&event.event_time)
    .bind(&event.remind_at)
    .bind(event.remind_on_time)
    .bind(recurrence)
    .bind(&event.recurrence_end)
    .bind(&now)
    .bind(id)
    .execute(&mut *transaction)
    .await
    .map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err("找不到要修改的任务".to_string());
    }

    sqlx::query("DELETE FROM reminder_queue WHERE event_id = ? AND fired = 0")
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    generate_reminders(
        &mut transaction,
        id,
        &event.event_time,
        event.remind_at.as_deref(),
        event.remind_on_time,
        None,
    )
    .await?;
    transaction.commit().await.map_err(db_error)
}

#[tauri::command]
pub async fn delete_event(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let pool = state.pool().await?;
    delete_event_in_pool(&pool, &id).await
}

async fn delete_event_in_pool(pool: &SqlitePool, id: &str) -> Result<(), String> {
    let mut transaction = pool.begin().await.map_err(db_error)?;
    sqlx::query("DELETE FROM reminder_queue WHERE event_id = ?")
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    sqlx::query("DELETE FROM events WHERE id = ?")
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)
}

#[tauri::command]
pub async fn toggle_complete(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let pool = state.pool().await?;
    toggle_complete_in_pool(&pool, &id).await
}

async fn toggle_complete_in_pool(pool: &SqlitePool, id: &str) -> Result<(), String> {
    let mut transaction = pool.begin().await.map_err(db_error)?;
    let event = sqlx::query_as::<_, DeskEvent>("SELECT * FROM events WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(db_error)?;
    let Some(event) = event else {
        return Ok(());
    };

    let now = now_iso();
    let new_completed = if event.completed == 0 { 1 } else { 0 };
    sqlx::query("UPDATE events SET completed = ?, updated_at = ? WHERE id = ?")
        .bind(new_completed)
        .bind(&now)
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    sqlx::query("DELETE FROM reminder_queue WHERE event_id = ? AND fired = 0")
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;

    if new_completed == 0 {
        generate_reminders(
            &mut transaction,
            id,
            &event.event_time,
            event.remind_at.as_deref(),
            event.remind_on_time,
            Some(&now),
        )
        .await?;
        transaction.commit().await.map_err(db_error)?;
        return Ok(());
    }

    if event.recurrence != "none" {
        generate_next_occurrence(&mut transaction, &event, &now).await?;
    }

    transaction.commit().await.map_err(db_error)
}

async fn generate_next_occurrence(
    transaction: &mut Transaction<'_, Sqlite>,
    event: &DeskEvent,
    now: &str,
) -> Result<(), String> {
    if let Some(next_id) = &event.generated_next_id {
        let exists = sqlx::query_scalar::<_, String>("SELECT id FROM events WHERE id = ? LIMIT 1")
            .bind(next_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(db_error)?
            .is_some();
        if exists {
            return Ok(());
        }
    }

    let Some(next_time) = calculate_next_occurrence(&event.event_time, &event.recurrence) else {
        return Ok(());
    };
    let within_end = event
        .recurrence_end
        .as_ref()
        .map(|end| next_time <= *end)
        .unwrap_or(true);
    if !within_end {
        return Ok(());
    }

    let new_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO events (id, title, description, event_time, completed, remind_at, remind_on_time, recurrence, recurrence_end, generated_next_id, created_at, updated_at) VALUES (?, ?, ?, ?, 0, ?, ?, ?, ?, NULL, ?, ?)",
    )
    .bind(&new_id)
    .bind(&event.title)
    .bind(&event.description)
    .bind(&next_time)
    .bind(&event.remind_at)
    .bind(event.remind_on_time)
    .bind(&event.recurrence)
    .bind(&event.recurrence_end)
    .bind(now)
    .bind(now)
    .execute(&mut **transaction)
    .await
    .map_err(db_error)?;
    generate_reminders(
        transaction,
        &new_id,
        &next_time,
        event.remind_at.as_deref(),
        event.remind_on_time,
        None,
    )
    .await?;
    sqlx::query("UPDATE events SET generated_next_id = ?, updated_at = ? WHERE id = ?")
        .bind(&new_id)
        .bind(now)
        .bind(&event.id)
        .execute(&mut **transaction)
        .await
        .map_err(db_error)?;
    Ok(())
}

async fn generate_reminders(
    transaction: &mut Transaction<'_, Sqlite>,
    event_id: &str,
    event_time: &str,
    remind_at: Option<&str>,
    remind_on_time: i64,
    only_future_after: Option<&str>,
) -> Result<(), String> {
    if remind_on_time != 0 && should_queue(event_time, only_future_after) {
        sqlx::query("INSERT INTO reminder_queue (event_id, fire_at, fired, type) VALUES (?, ?, 0, 'on_time')")
            .bind(event_id)
            .bind(event_time)
            .execute(&mut **transaction)
            .await
            .map_err(db_error)?;
    }
    if let Some(remind_at) = remind_at {
        if should_queue(remind_at, only_future_after) {
            sqlx::query("INSERT INTO reminder_queue (event_id, fire_at, fired, type) VALUES (?, ?, 0, 'advance')")
                .bind(event_id)
                .bind(remind_at)
                .execute(&mut **transaction)
                .await
                .map_err(db_error)?;
        }
    }
    Ok(())
}

fn validate_input(event: &EventInput) -> Result<(), String> {
    let title_len = event.title.trim().chars().count();
    if title_len == 0 || title_len > 200 {
        return Err("标题需要填写，并且不能超过 200 个字".to_string());
    }
    if event.description.chars().count() > 1000 {
        return Err("描述不能超过 1000 个字".to_string());
    }
    DateTime::parse_from_rfc3339(&event.event_time)
        .map_err(|_| "任务时间格式不正确".to_string())?;
    if let Some(value) = &event.remind_at {
        DateTime::parse_from_rfc3339(value).map_err(|_| "提醒时间格式不正确".to_string())?;
    }
    if let Some(value) = &event.recurrence_end {
        DateTime::parse_from_rfc3339(value).map_err(|_| "重复截止时间格式不正确".to_string())?;
    }
    Ok(())
}

fn should_queue(fire_at: &str, threshold: Option<&str>) -> bool {
    let Some(threshold) = threshold else {
        return true;
    };
    match (
        DateTime::parse_from_rfc3339(fire_at),
        DateTime::parse_from_rfc3339(threshold),
    ) {
        (Ok(fire_at), Ok(threshold)) => fire_at > threshold,
        _ => true,
    }
}

fn safe_recurrence(value: &str) -> &str {
    if VALID_RECURRENCES.contains(&value) {
        value
    } else {
        "none"
    }
}

fn calculate_next_occurrence(event_time: &str, recurrence: &str) -> Option<String> {
    let date = DateTime::parse_from_rfc3339(event_time)
        .ok()?
        .with_timezone(&Utc);
    let next = match recurrence {
        "daily" => date.checked_add_days(Days::new(1))?,
        "weekly" => date.checked_add_days(Days::new(7))?,
        "monthly" => {
            let source_day = date.day();
            let first_of_target = date.with_day(1)?.checked_add_months(Months::new(1))?;
            let following_month = first_of_target.checked_add_months(Months::new(1))?;
            let last_day = following_month.checked_sub_days(Days::new(1))?.day();
            first_of_target.with_day(source_day.min(last_day))?
        }
        _ => return None,
    };
    Some(next.to_rfc3339_opts(SecondsFormat::Millis, true))
}

fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn db_error(error: sqlx::Error) -> String {
    format!("数据库操作失败：{error}")
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_next_occurrence, create_event_in_pool, delete_event_in_pool,
        toggle_complete_in_pool, update_event_in_pool, EventInput,
    };
    use crate::db::create_test_pool;

    #[test]
    fn monthly_recurrence_clamps_to_last_day() {
        assert_eq!(
            calculate_next_occurrence("2026-01-31T01:00:00.000Z", "monthly").as_deref(),
            Some("2026-02-28T01:00:00.000Z")
        );
    }

    #[test]
    fn event_lifecycle_and_reminders_work_on_temporary_data() {
        tauri::async_runtime::block_on(async {
            let pool = create_test_pool().await;
            let event_id = create_event_in_pool(
                &pool,
                EventInput {
                    title: "临时测试任务".to_string(),
                    description: "只存在于自动测试中".to_string(),
                    event_time: "2099-01-02T10:00:00.000Z".to_string(),
                    remind_at: Some("2099-01-02T09:45:00.000Z".to_string()),
                    remind_on_time: 1,
                    recurrence: "none".to_string(),
                    recurrence_end: None,
                },
            )
            .await
            .expect("event should be created");

            let reminder_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM reminder_queue WHERE event_id = ?")
                    .bind(&event_id)
                    .fetch_one(&pool)
                    .await
                    .expect("reminders should be readable");
            assert_eq!(reminder_count, 2);

            update_event_in_pool(
                &pool,
                &event_id,
                EventInput {
                    title: "已修改的临时任务".to_string(),
                    description: "修改成功".to_string(),
                    event_time: "2099-01-03T10:00:00.000Z".to_string(),
                    remind_at: None,
                    remind_on_time: 0,
                    recurrence: "none".to_string(),
                    recurrence_end: None,
                },
            )
            .await
            .expect("event should be updated");

            let updated_title: String = sqlx::query_scalar("SELECT title FROM events WHERE id = ?")
                .bind(&event_id)
                .fetch_one(&pool)
                .await
                .expect("updated event should be readable");
            let reminder_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM reminder_queue WHERE event_id = ?")
                    .bind(&event_id)
                    .fetch_one(&pool)
                    .await
                    .expect("reminders should be readable");
            assert_eq!(updated_title, "已修改的临时任务");
            assert_eq!(reminder_count, 0);

            toggle_complete_in_pool(&pool, &event_id)
                .await
                .expect("event should be completed");
            let completed: i64 = sqlx::query_scalar("SELECT completed FROM events WHERE id = ?")
                .bind(&event_id)
                .fetch_one(&pool)
                .await
                .expect("completion state should be readable");
            assert_eq!(completed, 1);

            toggle_complete_in_pool(&pool, &event_id)
                .await
                .expect("event should be reopened");
            delete_event_in_pool(&pool, &event_id)
                .await
                .expect("event should be deleted");
            let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE id = ?")
                .bind(&event_id)
                .fetch_one(&pool)
                .await
                .expect("event count should be readable");
            assert_eq!(remaining, 0);
            pool.close().await;
        });
    }
}
