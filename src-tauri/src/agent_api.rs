use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

use crate::{agent_access::AgentAccessState, db::DatabaseState, events};

const AGENT_SOURCE_HEADER: &str = "x-desklist-agent";
const VALID_FILTERS: [&str; 6] = ["today", "upcoming", "completed", "all", "inbox", "trash"];

#[derive(Debug, Deserialize)]
struct EventQuery {
    filter: Option<String>,
    today_start: Option<String>,
    today_end: Option<String>,
}

#[derive(Serialize)]
struct CreatedResponse {
    id: String,
}

#[derive(Serialize)]
struct OkResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn router() -> Router<AppHandle> {
    Router::new()
        .route("/openapi.json", get(openapi))
        .route("/events", get(list_events).post(create_event))
        .route("/inbox", post(create_inbox_event))
        .route("/events/{id}", get(get_event).put(update_event))
        .route("/events/{id}/complete", post(complete_event))
        .route("/events/{id}/reopen", post(reopen_event))
        .route("/events/{id}/trash", post(trash_event))
        .route("/events/{id}/restore", post(restore_event))
}

async fn openapi() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        include_str!("../../docs/agent-api-openapi.json"),
    )
}

async fn list_events(
    State(app): State<AppHandle>,
    headers: HeaderMap,
    Query(query): Query<EventQuery>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&app, &headers) {
        return response;
    }
    let pool = match pool(&app).await {
        Ok(pool) => pool,
        Err(response) => return response,
    };
    let filter = query.filter.as_deref().unwrap_or("upcoming");
    if !VALID_FILTERS.contains(&filter) {
        return bad_request("不支持的任务筛选条件".to_string());
    }
    match events::fetch_events_in_pool(
        &pool,
        filter,
        query.today_start.as_deref(),
        query.today_end.as_deref(),
    )
    .await
    {
        Ok(items) => Json(items).into_response(),
        Err(error) => bad_request(error),
    }
}

async fn get_event(
    State(app): State<AppHandle>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&app, &headers) {
        return response;
    }
    let pool = match pool(&app).await {
        Ok(pool) => pool,
        Err(response) => return response,
    };
    match events::fetch_event_in_pool(&pool, &id).await {
        Ok(Some(event)) => Json(event).into_response(),
        Ok(None) => not_found("找不到任务"),
        Err(error) => internal_error(error),
    }
}

async fn create_event(
    State(app): State<AppHandle>,
    headers: HeaderMap,
    Json(event): Json<events::EventInput>,
) -> impl IntoResponse {
    let source = match authorize(&app, &headers) {
        Ok(source) => source,
        Err(response) => return response,
    };
    let pool = match pool(&app).await {
        Ok(pool) => pool,
        Err(response) => return response,
    };
    match events::create_event_in_pool(&pool, event).await {
        Ok(id) => match record_action(&pool, Some(&id), "create", &source).await {
            Ok(()) => (StatusCode::CREATED, Json(CreatedResponse { id })).into_response(),
            Err(error) => internal_error(error),
        },
        Err(error) => bad_request(error),
    }
}

async fn create_inbox_event(
    State(app): State<AppHandle>,
    headers: HeaderMap,
    Json(event): Json<events::InboxInput>,
) -> impl IntoResponse {
    let source = match authorize(&app, &headers) {
        Ok(source) => source,
        Err(response) => return response,
    };
    let pool = match pool(&app).await {
        Ok(pool) => pool,
        Err(response) => return response,
    };
    match events::create_inbox_event_in_pool(&pool, event).await {
        Ok(id) => match record_action(&pool, Some(&id), "create_inbox", &source).await {
            Ok(()) => (StatusCode::CREATED, Json(CreatedResponse { id })).into_response(),
            Err(error) => internal_error(error),
        },
        Err(error) => bad_request(error),
    }
}

async fn update_event(
    State(app): State<AppHandle>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(event): Json<events::EventInput>,
) -> impl IntoResponse {
    let source = match authorize(&app, &headers) {
        Ok(source) => source,
        Err(response) => return response,
    };
    let pool = match pool(&app).await {
        Ok(pool) => pool,
        Err(response) => return response,
    };
    match events::update_event_in_pool(&pool, &id, event).await {
        Ok(()) => match record_action(&pool, Some(&id), "update", &source).await {
            Ok(()) => ok(),
            Err(error) => internal_error(error),
        },
        Err(error) => bad_request(error),
    }
}

async fn complete_event(
    State(app): State<AppHandle>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    set_completion(app, headers, id, true).await
}

async fn reopen_event(
    State(app): State<AppHandle>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    set_completion(app, headers, id, false).await
}

async fn set_completion(
    app: AppHandle,
    headers: HeaderMap,
    id: String,
    completed: bool,
) -> axum::response::Response {
    let source = match authorize(&app, &headers) {
        Ok(source) => source,
        Err(response) => return response,
    };
    let pool = match pool(&app).await {
        Ok(pool) => pool,
        Err(response) => return response,
    };
    match events::set_completion_in_pool(&pool, &id, completed).await {
        Ok(()) => {
            let action = if completed { "complete" } else { "reopen" };
            match record_action(&pool, Some(&id), action, &source).await {
                Ok(()) => ok(),
                Err(error) => internal_error(error),
            }
        }
        Err(error) => bad_request(error),
    }
}

async fn trash_event(
    State(app): State<AppHandle>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let source = match authorize(&app, &headers) {
        Ok(source) => source,
        Err(response) => return response,
    };
    let pool = match pool(&app).await {
        Ok(pool) => pool,
        Err(response) => return response,
    };
    match events::delete_event_in_pool(&pool, &id).await {
        Ok(()) => match record_action(&pool, Some(&id), "trash", &source).await {
            Ok(()) => ok(),
            Err(error) => internal_error(error),
        },
        Err(error) => bad_request(error),
    }
}

async fn restore_event(
    State(app): State<AppHandle>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let source = match authorize(&app, &headers) {
        Ok(source) => source,
        Err(response) => return response,
    };
    let pool = match pool(&app).await {
        Ok(pool) => pool,
        Err(response) => return response,
    };
    match events::restore_event_in_pool(&pool, &id).await {
        Ok(()) => match record_action(&pool, Some(&id), "restore_from_trash", &source).await {
            Ok(()) => ok(),
            Err(error) => internal_error(error),
        },
        Err(error) => bad_request(error),
    }
}

fn authorize(app: &AppHandle, headers: &HeaderMap) -> Result<String, axum::response::Response> {
    let Some(token) = bearer_token(headers) else {
        return Err(unauthorized());
    };
    if !app.state::<AgentAccessState>().is_authorized(token) {
        return Err(unauthorized());
    }
    Ok(agent_source(headers))
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}

fn agent_source(headers: &HeaderMap) -> String {
    headers
        .get(AGENT_SOURCE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(100).collect())
        .unwrap_or_else(|| "unknown-agent".to_string())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, axum::response::Response> {
    app.state::<DatabaseState>()
        .pool()
        .await
        .map_err(internal_error)
}

pub(crate) async fn record_action(
    pool: &SqlitePool,
    event_id: Option<&str>,
    action: &str,
    source: &str,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    sqlx::query(
        "INSERT INTO agent_audit_log (event_id, action, source, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(event_id)
    .bind(action)
    .bind(source)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|error| format!("记录 Agent 操作失败：{error}"))?;
    Ok(())
}

fn ok() -> axum::response::Response {
    Json(OkResponse { status: "ok" }).into_response()
}

fn unauthorized() -> axum::response::Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "缺少或使用了无效的 Agent 令牌".to_string(),
        }),
    )
        .into_response()
}

fn not_found(message: &str) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
        .into_response()
}

fn bad_request(error: String) -> axum::response::Response {
    (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })).into_response()
}

fn internal_error(error: String) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::{agent_source, bearer_token, record_action};
    use crate::db::create_test_pool;
    use axum::http::{header, HeaderMap, HeaderValue};

    #[test]
    fn bearer_token_and_agent_source_are_parsed_safely() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer dl_test"),
        );
        headers.insert("x-desklist-agent", HeaderValue::from_static("codex-local"));
        assert_eq!(bearer_token(&headers), Some("dl_test"));
        assert_eq!(agent_source(&headers), "codex-local");
    }

    #[test]
    fn agent_actions_are_written_to_the_audit_log() {
        tauri::async_runtime::block_on(async {
            let pool = create_test_pool().await;
            record_action(&pool, Some("event-1"), "create", "codex-test")
                .await
                .expect("audit entry should be written");
            let row: (String, String, String) = sqlx::query_as(
                "SELECT event_id, action, source FROM agent_audit_log WHERE event_id = ?",
            )
            .bind("event-1")
            .fetch_one(&pool)
            .await
            .expect("audit entry should be readable");
            assert_eq!(row.0, "event-1");
            assert_eq!(row.1, "create");
            assert_eq!(row.2, "codex-test");
        });
    }
}
