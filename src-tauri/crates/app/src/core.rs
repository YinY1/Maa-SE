use anyhow::Context;
use maa_cfg::Parameters;
use maa_core::ConfigState;
use tauri::{async_runtime::spawn_blocking, State};

use crate::CommandResult;

#[tauri::command]
pub(crate) async fn run_daily(configs: State<'_, ConfigState>) -> CommandResult {
    let configs = configs.inner().clone();
    let tasks = configs.lock().unwrap().available_daily_tasks();
    // TODO: 验证blocking的合理性
    spawn_blocking(move || maa_core::run_core_tauri(tasks))
        .await
        .unwrap()
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub(crate) fn stop_core() {
    maa_core::stop_core();
}

/// update config json with given  name and params.
///
/// see `maa_cfg::ConfigType`.
#[tauri::command]
pub(crate) fn update_config(
    name: String,
    params: Parameters,
    configs: State<'_, ConfigState>,
) -> CommandResult {
    let cfg_type = name.parse().unwrap();
    configs
        .lock()
        .unwrap()
        .set_and_write(cfg_type, params)
        .context("update config")
        .map_err(|e| format!("{e:?}"))
}
