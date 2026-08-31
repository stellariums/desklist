use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, Response, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::TcpListener as StdTcpListener;
use tauri::{AppHandle, Manager};

use crate::{agent_api, agent_mcp, db::DatabaseState, events};

pub const WEB_ADDRESS: &str = "127.0.0.1:47831";
pub const WEB_URL: &str = "http://127.0.0.1:47831";

#[derive(Debug, Deserialize)]
struct EventQuery {
    filter: Option<String>,
    today_start: Option<String>,
    today_end: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct CreatedResponse {
    id: String,
}

#[derive(Serialize)]
struct OkResponse {
    status: &'static str,
}

pub fn start(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let listener = StdTcpListener::bind(WEB_ADDRESS)?;
    listener.set_nonblocking(true)?;

    tauri::async_runtime::spawn(async move {
        let listener = match tokio::net::TcpListener::from_std(listener) {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("无法启动浏览器工作台：{error}");
                return;
            }
        };
        let router = Router::new()
            .merge(agent_mcp::router(app.clone()))
            .nest("/api/agent/v1", agent_api::router())
            .route("/api/health", get(health))
            .route("/api/events", get(list_events).post(create_event))
            .route("/api/inbox", post(create_inbox_event))
            .route(
                "/api/events/{id}",
                put(update_event).delete(permanently_delete_event),
            )
            .route("/api/events/{id}/toggle", post(toggle_event))
            .route("/api/events/{id}/trash", post(trash_event))
            .route("/api/events/{id}/restore", post(restore_event))
            .fallback(get(serve_asset))
            .with_state(app);

        if let Err(error) = axum::serve(listener, router).await {
            eprintln!("浏览器工作台已停止：{error}");
        }
    });

    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn list_events(
    State(app): State<AppHandle>,
    Query(query): Query<EventQuery>,
) -> impl IntoResponse {
    let pool = match app.state::<DatabaseState>().pool().await {
        Ok(pool) => pool,
        Err(error) => return api_error(error),
    };
    let filter = query.filter.as_deref().unwrap_or("today");
    match events::fetch_events_in_pool(
        &pool,
        filter,
        query.today_start.as_deref(),
        query.today_end.as_deref(),
    )
    .await
    {
        Ok(items) => Json(items).into_response(),
        Err(error) => api_error(error),
    }
}

async fn create_event(
    State(app): State<AppHandle>,
    Json(event): Json<events::EventInput>,
) -> impl IntoResponse {
    let pool = match app.state::<DatabaseState>().pool().await {
        Ok(pool) => pool,
        Err(error) => return api_error(error),
    };
    match events::create_event_in_pool(&pool, event).await {
        Ok(id) => (StatusCode::CREATED, Json(CreatedResponse { id })).into_response(),
        Err(error) => api_bad_request(error),
    }
}

async fn create_inbox_event(
    State(app): State<AppHandle>,
    Json(event): Json<events::InboxInput>,
) -> impl IntoResponse {
    let pool = match app.state::<DatabaseState>().pool().await {
        Ok(pool) => pool,
        Err(error) => return api_error(error),
    };
    match events::create_inbox_event_in_pool(&pool, event).await {
        Ok(id) => (StatusCode::CREATED, Json(CreatedResponse { id })).into_response(),
        Err(error) => api_bad_request(error),
    }
}

async fn update_event(
    State(app): State<AppHandle>,
    Path(id): Path<String>,
    Json(event): Json<events::EventInput>,
) -> impl IntoResponse {
    let pool = match app.state::<DatabaseState>().pool().await {
        Ok(pool) => pool,
        Err(error) => return api_error(error),
    };
    match events::update_event_in_pool(&pool, &id, event).await {
        Ok(()) => Json(OkResponse { status: "ok" }).into_response(),
        Err(error) => api_bad_request(error),
    }
}

async fn toggle_event(State(app): State<AppHandle>, Path(id): Path<String>) -> impl IntoResponse {
    let pool = match app.state::<DatabaseState>().pool().await {
        Ok(pool) => pool,
        Err(error) => return api_error(error),
    };
    match events::toggle_complete_in_pool(&pool, &id).await {
        Ok(()) => Json(OkResponse { status: "ok" }).into_response(),
        Err(error) => api_bad_request(error),
    }
}

async fn trash_event(State(app): State<AppHandle>, Path(id): Path<String>) -> impl IntoResponse {
    let pool = match app.state::<DatabaseState>().pool().await {
        Ok(pool) => pool,
        Err(error) => return api_error(error),
    };
    match events::delete_event_in_pool(&pool, &id).await {
        Ok(()) => Json(OkResponse { status: "ok" }).into_response(),
        Err(error) => api_bad_request(error),
    }
}

async fn restore_event(State(app): State<AppHandle>, Path(id): Path<String>) -> impl IntoResponse {
    let pool = match app.state::<DatabaseState>().pool().await {
        Ok(pool) => pool,
        Err(error) => return api_error(error),
    };
    match events::restore_event_in_pool(&pool, &id).await {
        Ok(()) => Json(OkResponse { status: "ok" }).into_response(),
        Err(error) => api_bad_request(error),
    }
}

async fn permanently_delete_event(
    State(app): State<AppHandle>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let pool = match app.state::<DatabaseState>().pool().await {
        Ok(pool) => pool,
        Err(error) => return api_error(error),
    };
    match events::permanently_delete_event_in_pool(&pool, &id).await {
        Ok(()) => Json(OkResponse { status: "ok" }).into_response(),
        Err(error) => api_bad_request(error),
    }
}

fn api_error(error: String) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error }),
    )
        .into_response()
}

fn api_bad_request(error: String) -> axum::response::Response {
    (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })).into_response()
}

async fn serve_asset(State(app): State<AppHandle>, uri: Uri) -> Response<Body> {
    let Some(path) = normalize_asset_path(uri.path()) else {
        return not_found();
    };
    let resolver = app.asset_resolver();
    let asset = resolver.get(path.clone()).or_else(|| {
        if path.contains('.') {
            None
        } else {
            resolver.get("index.html".to_string())
        }
    });
    let Some(asset) = asset else {
        return not_found();
    };

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, asset.mime_type())
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(header::CACHE_CONTROL, "no-cache");
    if path == "index.html" || !path.contains('.') {
        response = response.header(
            header::CONTENT_SECURITY_POLICY,
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'",
        );
    }
    response
        .body(Body::from(asset.bytes))
        .unwrap_or_else(|_| not_found())
}

fn normalize_asset_path(path: &str) -> Option<String> {
    if path.contains("..") || path.contains('\\') {
        return None;
    }
    let path = path.trim_start_matches('/');
    Some(if path.is_empty() {
        "index.html".to_string()
    } else {
        path.to_string()
    })
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from("Not found"))
        .expect("static response should build")
}

#[cfg(test)]
mod tests {
    use super::normalize_asset_path;

    #[test]
    fn static_paths_stay_inside_embedded_assets() {
        assert_eq!(normalize_asset_path("/").as_deref(), Some("index.html"));
        assert_eq!(
            normalize_asset_path("/assets/app.js").as_deref(),
            Some("assets/app.js")
        );
        assert_eq!(normalize_asset_path("/../desklist.db"), None);
        assert_eq!(normalize_asset_path("/assets\\app.js"), None);
    }
}
