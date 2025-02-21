use anyhow::Context;
use maa_core::TaskList;
use tauri::{async_runtime::spawn_blocking, AppHandle, State};

use crate::{config::update_config, CommandResult};

#[tauri::command]
pub(crate) async fn start_core(app: AppHandle, task_list: State<'_, TaskList>) -> CommandResult {
    let task_list = task_list.lock().unwrap().clone();
    // TODO: 验证blocking的合理性
    spawn_blocking(move || maa_core::run_core_tauri(app, task_list))
        .await
        .unwrap()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn stop_core() {
    maa_core::stop_core();
}

/// enable a task with given task name and params.
///
/// see task names in `maa_types::TaskType`.
#[tauri::command]
pub(crate) fn update_task(
    name: &str,
    params: &str,
    task_list: State<'_, TaskList>,
) -> CommandResult {
    task_list
        .lock()
        .unwrap()
        .insert(name.to_string(), params.to_string());

    let cfg_type = name.parse()?;
    update_config(cfg_type, params)
        .context("update config")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn disable_task(name: &str, task_list: State<'_, TaskList>) {
    task_list.lock().unwrap().remove(name);
}
