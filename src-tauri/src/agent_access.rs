use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::RwLock,
};
use tauri::{AppHandle, Manager, State};

const ACCESS_FILE: &str = "agent-access.json";
pub const AGENT_API_URL: &str = "http://127.0.0.1:47831/api/agent/v1";
pub const AGENT_MCP_URL: &str = "http://127.0.0.1:47831/mcp";

#[derive(Default)]
pub struct AgentAccessState {
    token: RwLock<String>,
}

impl AgentAccessState {
    pub fn is_authorized(&self, candidate: &str) -> bool {
        let Ok(token) = self.token.read() else {
            return false;
        };
        constant_time_eq(token.as_bytes(), candidate.as_bytes())
    }

    fn token(&self) -> Result<String, String> {
        self.token
            .read()
            .map(|token| token.clone())
            .map_err(|_| "Agent 令牌状态不可用".to_string())
    }

    fn replace_token(&self, token: String) -> Result<(), String> {
        *self
            .token
            .write()
            .map_err(|_| "Agent 令牌状态不可用".to_string())? = token;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentAccessConfig {
    token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAccessStatus {
    endpoint: &'static str,
    mcp_endpoint: &'static str,
    token: String,
}

pub fn initialize(app: &AppHandle) -> Result<(), String> {
    let config_path = access_path(app)?;
    let token = if config_path.exists() {
        let raw = fs::read_to_string(&config_path)
            .map_err(|error| format!("无法读取 Agent 访问设置：{error}"))?;
        match serde_json::from_str::<AgentAccessConfig>(&raw) {
            Ok(config) if validate_token(&config.token).is_ok() => config.token,
            _ => {
                let token = generate_token();
                save_token(&config_path, &token)?;
                token
            }
        }
    } else {
        let token = generate_token();
        save_token(&config_path, &token)?;
        token
    };
    app.state::<AgentAccessState>().replace_token(token)
}

#[tauri::command]
pub fn get_agent_access(state: State<'_, AgentAccessState>) -> Result<AgentAccessStatus, String> {
    Ok(AgentAccessStatus {
        endpoint: AGENT_API_URL,
        mcp_endpoint: AGENT_MCP_URL,
        token: state.token()?,
    })
}

#[tauri::command]
pub fn regenerate_agent_token(
    app: AppHandle,
    state: State<'_, AgentAccessState>,
) -> Result<AgentAccessStatus, String> {
    let token = generate_token();
    save_token(&access_path(&app)?, &token)?;
    state.replace_token(token.clone())?;
    Ok(AgentAccessStatus {
        endpoint: AGENT_API_URL,
        mcp_endpoint: AGENT_MCP_URL,
        token,
    })
}

fn access_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法读取程序配置目录：{error}"))?;
    Ok(config_dir.join(ACCESS_FILE))
}

fn save_token(path: &Path, token: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Agent 访问设置路径无效".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建程序配置目录：{error}"))?;
    let temporary_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(&AgentAccessConfig {
        token: token.to_string(),
    })
    .map_err(|error| format!("无法保存 Agent 访问设置：{error}"))?;
    if path.exists() {
        fs::write(path, content).map_err(|error| format!("无法保存 Agent 访问设置：{error}"))?;
    } else {
        fs::write(&temporary_path, content)
            .map_err(|error| format!("无法保存 Agent 访问设置：{error}"))?;
        fs::rename(&temporary_path, path)
            .map_err(|error| format!("无法启用 Agent 访问设置：{error}"))?;
    }
    Ok(())
}

fn generate_token() -> String {
    format!(
        "dl_{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn validate_token(token: &str) -> Result<(), String> {
    if token.starts_with("dl_") && token.len() == 67 {
        Ok(())
    } else {
        Err("Agent 访问设置中的令牌格式无效".to_string())
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

#[cfg(test)]
mod tests {
    use super::{constant_time_eq, generate_token, validate_token};

    #[test]
    fn generated_agent_token_has_expected_shape() {
        let token = generate_token();
        assert!(validate_token(&token).is_ok());
        assert_eq!(token.len(), 67);
    }

    #[test]
    fn token_comparison_rejects_wrong_values() {
        assert!(constant_time_eq(b"same", b"same"));
        assert!(!constant_time_eq(b"same", b"diff"));
        assert!(!constant_time_eq(b"same", b"short"));
    }
}
