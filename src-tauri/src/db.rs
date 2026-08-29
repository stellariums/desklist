use serde::{Deserialize, Serialize};
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    Connection, SqliteConnection, SqlitePool,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager, State};
use tokio::sync::{Mutex, RwLock};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
const DATABASE_FILE: &str = "desklist.db";
const LOCATION_FILE: &str = "data-location.json";

#[derive(Default)]
pub struct DatabaseState {
    pool: RwLock<Option<SqlitePool>>,
    data_dir: RwLock<Option<PathBuf>>,
    last_error: RwLock<Option<String>>,
    configure_lock: Mutex<()>,
}

impl DatabaseState {
    pub async fn pool(&self) -> Result<SqlitePool, String> {
        self.pool
            .read()
            .await
            .clone()
            .ok_or_else(|| "请先选择任务数据的保存位置".to_string())
    }

    pub async fn data_dir(&self) -> Result<PathBuf, String> {
        self.data_dir
            .read()
            .await
            .clone()
            .ok_or_else(|| "请先选择任务数据的保存位置".to_string())
    }

    async fn set_ready(&self, pool: SqlitePool, data_dir: PathBuf) {
        *self.pool.write().await = Some(pool);
        *self.data_dir.write().await = Some(data_dir);
        *self.last_error.write().await = None;
    }

    async fn set_error(&self, error: String) {
        *self.last_error.write().await = Some(error);
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataStatus {
    pub configured: bool,
    pub data_dir: Option<String>,
    pub legacy_database_found: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DataLocationConfig {
    data_dir: String,
}

pub async fn initialize(app: &AppHandle) {
    let state = app.state::<DatabaseState>();
    let config_dir = match app.path().app_config_dir() {
        Ok(path) => path,
        Err(error) => {
            state
                .set_error(format!("无法读取程序配置目录：{error}"))
                .await;
            return;
        }
    };

    let location_path = config_dir.join(LOCATION_FILE);
    if !location_path.exists() {
        return;
    }

    let result = async {
        let raw = fs::read_to_string(&location_path)
            .map_err(|error| format!("无法读取数据位置设置：{error}"))?;
        let config: DataLocationConfig =
            serde_json::from_str(&raw).map_err(|error| format!("数据位置设置已损坏：{error}"))?;
        let data_dir = PathBuf::from(config.data_dir);
        let database_path = data_dir.join(DATABASE_FILE);
        if !database_path.exists() {
            return Err(format!("找不到任务数据库：{}", database_path.display()));
        }

        let pool = open_pool(&database_path).await?;
        MIGRATOR
            .run(&pool)
            .await
            .map_err(|error| format!("数据库升级失败：{error}"))?;
        validate_database(&pool).await?;
        Ok((pool, data_dir))
    }
    .await;

    match result {
        Ok((pool, data_dir)) => state.set_ready(pool, data_dir).await,
        Err(error) => state.set_error(error).await,
    }
}

#[tauri::command]
pub async fn get_data_status(
    app: AppHandle,
    state: State<'_, DatabaseState>,
) -> Result<DataStatus, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法读取程序配置目录：{error}"))?;
    Ok(status(&state, &config_dir).await)
}

#[tauri::command]
pub async fn configure_data_directory(
    app: AppHandle,
    state: State<'_, DatabaseState>,
    data_dir: String,
) -> Result<DataStatus, String> {
    let _configure_guard = state.configure_lock.lock().await;
    let selected_dir = PathBuf::from(data_dir.trim());
    if !selected_dir.is_absolute() {
        return Err("请选择一个完整的文件夹路径".to_string());
    }

    if let Some(current_dir) = state.data_dir.read().await.clone() {
        if current_dir == selected_dir {
            let config_dir = app
                .path()
                .app_config_dir()
                .map_err(|error| format!("无法读取程序配置目录：{error}"))?;
            return Ok(status(&state, &config_dir).await);
        }
        return Err("当前版本暂不支持再次移动数据位置".to_string());
    }

    fs::create_dir_all(&selected_dir).map_err(|error| format!("无法创建所选文件夹：{error}"))?;

    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法读取程序配置目录：{error}"))?;
    let legacy_database = config_dir.join(DATABASE_FILE);
    let target_database = selected_dir.join(DATABASE_FILE);

    if target_database.exists() {
        return Err(format!(
            "所选文件夹中已经存在 {}，请换一个空文件夹",
            DATABASE_FILE
        ));
    }

    let pool = if legacy_database.exists() {
        migrate_legacy_database(&legacy_database, &target_database).await?
    } else {
        let pool = open_pool(&target_database).await?;
        MIGRATOR
            .run(&pool)
            .await
            .map_err(|error| format!("创建任务数据库失败：{error}"))?;
        validate_database(&pool).await?;
        pool
    };

    save_location(&config_dir, &selected_dir)?;
    state.set_ready(pool, selected_dir).await;
    Ok(status(&state, &config_dir).await)
}

#[tauri::command]
pub async fn open_data_directory(state: State<'_, DatabaseState>) -> Result<(), String> {
    let data_dir = state.data_dir().await?;
    tauri_plugin_opener::open_path(data_dir, None::<&str>)
        .map_err(|error| format!("无法打开数据文件夹：{error}"))
}

async fn status(state: &DatabaseState, config_dir: &Path) -> DataStatus {
    let data_dir = state.data_dir.read().await.clone();
    DataStatus {
        configured: data_dir.is_some(),
        data_dir: data_dir.map(|path| path.to_string_lossy().into_owned()),
        legacy_database_found: config_dir.join(DATABASE_FILE).exists(),
        error: state.last_error.read().await.clone(),
    }
}

async fn migrate_legacy_database(source: &Path, target: &Path) -> Result<SqlitePool, String> {
    let temporary = target.with_extension(format!("migrating-{}.db", uuid::Uuid::new_v4()));
    let result = async {
        backup_database(source, &temporary).await?;

        let temporary_pool = open_pool(&temporary).await?;
        validate_database(&temporary_pool).await?;
        compare_core_counts(source, &temporary_pool).await?;
        temporary_pool.close().await;

        fs::rename(&temporary, target).map_err(|error| format!("无法完成数据库迁移：{error}"))?;

        let pool = open_pool(target).await?;
        MIGRATOR
            .run(&pool)
            .await
            .map_err(|error| format!("数据库升级失败：{error}"))?;
        validate_database(&pool).await?;
        Ok(pool)
    }
    .await;

    if result.is_err() && temporary.exists() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

async fn backup_database(source: &Path, target: &Path) -> Result<(), String> {
    let options = SqliteConnectOptions::new()
        .filename(source)
        .create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options)
        .await
        .map_err(|error| format!("无法打开原数据库：{error}"))?;

    let escaped_target = target.to_string_lossy().replace('\'', "''");
    sqlx::query(&format!("VACUUM INTO '{escaped_target}'"))
        .execute(&mut connection)
        .await
        .map_err(|error| format!("复制原数据库失败：{error}"))?;
    connection
        .close()
        .await
        .map_err(|error| format!("关闭原数据库失败：{error}"))?;
    Ok(())
}

async fn compare_core_counts(source: &Path, target: &SqlitePool) -> Result<(), String> {
    let source_options = SqliteConnectOptions::new()
        .filename(source)
        .create_if_missing(false);
    let source_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(source_options)
        .await
        .map_err(|error| format!("无法校验原数据库：{error}"))?;

    let source_events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(&source_pool)
        .await
        .map_err(|error| format!("无法统计原任务：{error}"))?;
    let target_events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(target)
        .await
        .map_err(|error| format!("无法统计迁移后的任务：{error}"))?;
    let source_reminders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reminder_queue")
        .fetch_one(&source_pool)
        .await
        .map_err(|error| format!("无法统计原提醒：{error}"))?;
    let target_reminders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reminder_queue")
        .fetch_one(target)
        .await
        .map_err(|error| format!("无法统计迁移后的提醒：{error}"))?;
    source_pool.close().await;

    if source_events != target_events || source_reminders != target_reminders {
        return Err(format!(
            "迁移前后数据数量不一致：任务 {source_events}/{target_events}，提醒 {source_reminders}/{target_reminders}"
        ));
    }
    Ok(())
}

async fn open_pool(database_path: &Path) -> Result<SqlitePool, String> {
    let options = SqliteConnectOptions::new()
        .filename(database_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|error| format!("无法打开任务数据库：{error}"))
}

async fn validate_database(pool: &SqlitePool) -> Result<(), String> {
    let result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(pool)
        .await
        .map_err(|error| format!("无法检查数据库：{error}"))?;
    if result != "ok" {
        return Err(format!("数据库检查未通过：{result}"));
    }
    Ok(())
}

fn save_location(config_dir: &Path, data_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(config_dir).map_err(|error| format!("无法创建程序配置目录：{error}"))?;
    let location_path = config_dir.join(LOCATION_FILE);
    let temporary_path = config_dir.join(format!("{LOCATION_FILE}.tmp"));
    let config = DataLocationConfig {
        data_dir: data_dir.to_string_lossy().into_owned(),
    };
    let content = serde_json::to_string_pretty(&config)
        .map_err(|error| format!("无法保存数据位置：{error}"))?;
    fs::write(&temporary_path, content).map_err(|error| format!("无法保存数据位置：{error}"))?;
    fs::rename(&temporary_path, &location_path)
        .map_err(|error| format!("无法启用数据位置：{error}"))?;
    Ok(())
}

#[cfg(test)]
pub async fn create_test_pool() -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .in_memory(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("test database should open");
    MIGRATOR
        .run(&pool)
        .await
        .expect("test migrations should run");
    pool
}
