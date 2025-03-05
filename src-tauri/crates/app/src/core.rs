use anyhow::Context;
use maa_cfg::{Config, Parameters};
use maa_core::tauri_logger::{log_config, LogHandleState};
use tauri::{async_runtime::spawn_blocking, AppHandle, State};

use crate::CommandResult;

#[tauri::command]
pub async fn run_daily(configs: State<'_, Config>) -> CommandResult<()> {
    let tasks = configs.available_daily_tasks();
    spawn_blocking(move || maa_core::run_core_tauri(tasks))
        .await
        .unwrap()
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub fn stop_core() {
    maa_core::stop_core();
}

/// update config json with given  name and params.
///
/// see `maa_cfg::ConfigType`.
#[tauri::command]
pub async fn update_config(
    name: String,
    params: Parameters,
    configs: State<'_, Config>,
) -> CommandResult<()> {
    let cfg_type = name.parse().unwrap();
    configs
        .set_and_write(cfg_type, params)
        .await
        .context("update config")
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub async fn get_config(configs: State<'_, Config>) -> CommandResult<String> {
    serde_json::to_string(configs.inner())
        .context("serialize configs to string")
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub async fn set_log_level(
    level: &str,
    app_handle: AppHandle,
    log_handle: State<'_, LogHandleState>,
) -> CommandResult<()> {
    let level = level
        .parse()
        .map_err(|e: log::ParseLevelError| e.to_string())?;
    let config = log_config(app_handle, level).map_err(|e| format!("{e:?}"))?;
    log_handle.get().unwrap().set_config(config);
    Ok(())
}
