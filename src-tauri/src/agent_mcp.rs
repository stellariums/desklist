use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    },
    ErrorData as McpError, ServerHandler,
};
use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::{
    agent_access::AgentAccessState,
    agent_api::record_action,
    db::DatabaseState,
    events::{self, EventInput, InboxInput},
};

const MCP_SOURCE: &str = "mcp-agent";
const VALID_FILTERS: [&str; 6] = ["today", "upcoming", "completed", "all", "inbox", "trash"];

#[derive(Clone)]
struct DesklistMcp {
    app: AppHandle,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListTasksParams {
    /// Filter: upcoming, completed, all, inbox, trash, or today.
    #[serde(default = "default_filter")]
    filter: String,
    /// RFC 3339 start boundary. Required only when filter is today.
    today_start: Option<String>,
    /// RFC 3339 end boundary. Required only when filter is today.
    today_end: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateTaskParams {
    /// Task title, at most 200 characters.
    title: String,
    /// Optional background, requirements, or next action.
    #[serde(default)]
    description: String,
    /// Scheduled start in RFC 3339 format, including a timezone offset.
    event_time: String,
    /// Optional scheduled end in RFC 3339 format.
    scheduled_end: Option<String>,
    /// Optional real deadline in RFC 3339 format. Do not invent one.
    due_time: Option<String>,
    /// Optional advance reminder time in RFC 3339 format.
    remind_at: Option<String>,
    /// Whether to remind at due_time or event_time: 0 or 1.
    #[serde(default = "default_remind_on_time")]
    remind_on_time: i64,
    /// Recurrence: none, daily, weekly, or monthly.
    #[serde(default = "default_recurrence")]
    recurrence: String,
    /// Optional recurrence end in RFC 3339 format.
    recurrence_end: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CaptureTaskParams {
    /// Task title, at most 200 characters.
    title: String,
    /// Optional background, requirements, or next action.
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateTaskParams {
    /// Existing Desklist task ID.
    id: String,
    /// Complete replacement title, at most 200 characters.
    title: String,
    /// Complete replacement description.
    #[serde(default)]
    description: String,
    /// Complete replacement scheduled start in RFC 3339 format.
    event_time: String,
    /// Complete replacement scheduled end, or null.
    scheduled_end: Option<String>,
    /// Complete replacement deadline, or null. Do not invent one.
    due_time: Option<String>,
    /// Complete replacement advance reminder time, or null.
    remind_at: Option<String>,
    /// Whether to remind on time: 0 or 1.
    #[serde(default = "default_remind_on_time")]
    remind_on_time: i64,
    /// Recurrence: none, daily, weekly, or monthly.
    #[serde(default = "default_recurrence")]
    recurrence: String,
    /// Complete replacement recurrence end, or null.
    recurrence_end: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TaskIdParams {
    /// Existing Desklist task ID.
    id: String,
}

#[tool_router]
impl DesklistMcp {
    #[tool(
        description = "List Desklist tasks. Defaults to incomplete scheduled tasks. Use filter=inbox for unscheduled captures and filter=trash for recoverable deletions.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn list_tasks(
        &self,
        Parameters(params): Parameters<ListTasksParams>,
    ) -> Result<CallToolResult, McpError> {
        if !VALID_FILTERS.contains(&params.filter.as_str()) {
            return tool_error("Unsupported task filter");
        }
        let pool = match self.app.state::<DatabaseState>().pool().await {
            Ok(pool) => pool,
            Err(error) => return tool_error(error),
        };
        match events::fetch_events_in_pool(
            &pool,
            &params.filter,
            params.today_start.as_deref(),
            params.today_end.as_deref(),
        )
        .await
        {
            Ok(events) => json_result(&events),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Get one Desklist task by ID without reading an entire task list.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_task(
        &self,
        Parameters(params): Parameters<TaskIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let pool = match self.app.state::<DatabaseState>().pool().await {
            Ok(pool) => pool,
            Err(error) => return tool_error(error),
        };
        match events::fetch_event_in_pool(&pool, &params.id).await {
            Ok(Some(event)) => json_result(&event),
            Ok(None) => tool_error("Task not found"),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Create a scheduled Desklist task. Use capture_task instead when the user has not chosen a date or time.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn create_task(
        &self,
        Parameters(params): Parameters<CreateTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let pool = match self.app.state::<DatabaseState>().pool().await {
            Ok(pool) => pool,
            Err(error) => return tool_error(error),
        };
        match events::create_event_in_pool(&pool, params.into()).await {
            Ok(id) => match record_action(&pool, Some(&id), "create", MCP_SOURCE).await {
                Ok(()) => json_result(&serde_json::json!({ "id": id })),
                Err(error) => tool_error(error),
            },
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Capture an unscheduled task in the Desklist inbox. Use this when the user has not chosen a date or time.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn capture_task(
        &self,
        Parameters(params): Parameters<CaptureTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let pool = match self.app.state::<DatabaseState>().pool().await {
            Ok(pool) => pool,
            Err(error) => return tool_error(error),
        };
        match events::create_inbox_event_in_pool(&pool, params.into()).await {
            Ok(id) => match record_action(&pool, Some(&id), "create_inbox", MCP_SOURCE).await {
                Ok(()) => json_result(&serde_json::json!({ "id": id })),
                Err(error) => tool_error(error),
            },
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Replace the editable fields of an existing Desklist task. First call list_tasks and preserve fields the user did not ask to change.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn update_task(
        &self,
        Parameters(params): Parameters<UpdateTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let pool = match self.app.state::<DatabaseState>().pool().await {
            Ok(pool) => pool,
            Err(error) => return tool_error(error),
        };
        let id = params.id.clone();
        match events::update_event_in_pool(&pool, &id, params.into()).await {
            Ok(()) => match record_action(&pool, Some(&id), "update", MCP_SOURCE).await {
                Ok(()) => ok_result(),
                Err(error) => tool_error(error),
            },
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Mark a scheduled Desklist task complete. Safe to retry.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn complete_task(
        &self,
        Parameters(params): Parameters<TaskIdParams>,
    ) -> Result<CallToolResult, McpError> {
        self.set_completion(params.id, true).await
    }

    #[tool(
        description = "Mark a completed Desklist task incomplete again. Safe to retry.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn reopen_task(
        &self,
        Parameters(params): Parameters<TaskIdParams>,
    ) -> Result<CallToolResult, McpError> {
        self.set_completion(params.id, false).await
    }

    #[tool(
        description = "Move a Desklist task to the recoverable recycle bin. This never permanently deletes data.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn trash_task(
        &self,
        Parameters(params): Parameters<TaskIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let pool = match self.app.state::<DatabaseState>().pool().await {
            Ok(pool) => pool,
            Err(error) => return tool_error(error),
        };
        match events::delete_event_in_pool(&pool, &params.id).await {
            Ok(()) => match record_action(&pool, Some(&params.id), "trash", MCP_SOURCE).await {
                Ok(()) => ok_result(),
                Err(error) => tool_error(error),
            },
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Restore a Desklist task from the recycle bin.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn restore_task(
        &self,
        Parameters(params): Parameters<TaskIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let pool = match self.app.state::<DatabaseState>().pool().await {
            Ok(pool) => pool,
            Err(error) => return tool_error(error),
        };
        match events::restore_event_in_pool(&pool, &params.id).await {
            Ok(()) => {
                match record_action(&pool, Some(&params.id), "restore_from_trash", MCP_SOURCE).await
                {
                    Ok(()) => ok_result(),
                    Err(error) => tool_error(error),
                }
            }
            Err(error) => tool_error(error),
        }
    }
}

impl DesklistMcp {
    async fn set_completion(
        &self,
        id: String,
        completed: bool,
    ) -> Result<CallToolResult, McpError> {
        let pool = match self.app.state::<DatabaseState>().pool().await {
            Ok(pool) => pool,
            Err(error) => return tool_error(error),
        };
        match events::set_completion_in_pool(&pool, &id, completed).await {
            Ok(()) => {
                let action = if completed { "complete" } else { "reopen" };
                match record_action(&pool, Some(&id), action, MCP_SOURCE).await {
                    Ok(()) => ok_result(),
                    Err(error) => tool_error(error),
                }
            }
            Err(error) => tool_error(error),
        }
    }
}

#[tool_handler(
    name = "desklist",
    version = "1.0.0",
    instructions = "Manage the user's local Desklist tasks. If no date or time is confirmed, use capture_task so the item stays in the inbox. Use create_task only for a real scheduled time in RFC 3339 with timezone. Never invent a deadline; due_time must be null unless the user supplied one. Before update_task, use get_task and preserve fields not requested to change. Deletion always uses trash_task and remains recoverable. complete_task and reopen_task are safe to retry."
)]
impl ServerHandler for DesklistMcp {}

impl From<CreateTaskParams> for EventInput {
    fn from(params: CreateTaskParams) -> Self {
        Self {
            title: params.title,
            description: params.description,
            event_time: params.event_time,
            scheduled_end: params.scheduled_end,
            due_time: params.due_time,
            remind_at: params.remind_at,
            remind_on_time: params.remind_on_time,
            recurrence: params.recurrence,
            recurrence_end: params.recurrence_end,
        }
    }
}

impl From<UpdateTaskParams> for EventInput {
    fn from(params: UpdateTaskParams) -> Self {
        Self {
            title: params.title,
            description: params.description,
            event_time: params.event_time,
            scheduled_end: params.scheduled_end,
            due_time: params.due_time,
            remind_at: params.remind_at,
            remind_on_time: params.remind_on_time,
            recurrence: params.recurrence,
            recurrence_end: params.recurrence_end,
        }
    }
}

impl From<CaptureTaskParams> for InboxInput {
    fn from(params: CaptureTaskParams) -> Self {
        Self {
            title: params.title,
            description: params.description,
        }
    }
}

pub fn router(app: AppHandle) -> Router<AppHandle> {
    let service_app = app.clone();
    let service = StreamableHttpService::new(
        move || {
            Ok(DesklistMcp {
                app: service_app.clone(),
            })
        },
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true),
    );

    Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn_with_state(app, require_token))
}

async fn require_token(State(app): State<AppHandle>, request: Request, next: Next) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    if token
        .map(|token| app.state::<AgentAccessState>().is_authorized(token))
        .unwrap_or(false)
    {
        next.run(request).await
    } else {
        (StatusCode::UNAUTHORIZED, "Invalid Desklist Agent token").into_response()
    }
}

fn default_filter() -> String {
    "upcoming".to_string()
}

fn default_remind_on_time() -> i64 {
    1
}

fn default_recurrence() -> String {
    "none".to_string()
}

fn ok_result() -> Result<CallToolResult, McpError> {
    json_result(&serde_json::json!({ "status": "ok" }))
}

fn json_result(value: &impl serde::Serialize) -> Result<CallToolResult, McpError> {
    match serde_json::to_string(value) {
        Ok(json) => Ok(CallToolResult::success(vec![ContentBlock::text(json)])),
        Err(error) => tool_error(format!("Unable to serialize Desklist response: {error}")),
    }
}

fn tool_error(message: impl Into<String>) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![ContentBlock::text(
        message.into(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::DesklistMcp;

    #[test]
    fn mcp_tools_are_discoverable_with_safety_annotations() {
        let tools = DesklistMcp::tool_router().list_all();
        let names: Vec<_> = tools.iter().map(|tool| tool.name.as_ref()).collect();
        assert_eq!(
            names,
            vec![
                "capture_task",
                "complete_task",
                "create_task",
                "get_task",
                "list_tasks",
                "reopen_task",
                "restore_task",
                "trash_task",
                "update_task",
            ]
        );

        let list_tasks = tools
            .iter()
            .find(|tool| tool.name == "list_tasks")
            .expect("list_tasks should be published");
        assert_eq!(
            list_tasks
                .annotations
                .as_ref()
                .and_then(|annotations| annotations.read_only_hint),
            Some(true)
        );

        let get_task = tools
            .iter()
            .find(|tool| tool.name == "get_task")
            .expect("get_task should be published");
        assert_eq!(
            get_task
                .annotations
                .as_ref()
                .and_then(|annotations| annotations.read_only_hint),
            Some(true)
        );

        let trash_task = tools
            .iter()
            .find(|tool| tool.name == "trash_task")
            .expect("trash_task should be published");
        assert_eq!(
            trash_task
                .annotations
                .as_ref()
                .and_then(|annotations| annotations.destructive_hint),
            Some(true)
        );
    }
}
