use anyhow::Context;
use maa_core::TaskList;
use tauri::{async_runtime::spawn_blocking, State};

use crate::{config::update_config, CommandResult};

#[tauri::command]
pub(crate) async fn start_core(task_list: State<'_, TaskList>) -> CommandResult {
    let task_list = task_list.inner().clone();
    // TODO: 验证blocking的合理性
    spawn_blocking(move || maa_core::run_core_tauri(task_list))
        .await
        .unwrap()
        .map_err(|e| format!("{e:?}"))
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
    enable: bool,
    name: &str,
    params: &str,
    task_list: State<'_, TaskList>,
) -> CommandResult {
    task_list.insert(name.to_string(), (enable, params.to_string()));

    let cfg_type = name.parse()?;
    update_config(cfg_type, params)
        .context("update config")
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub(crate) fn disable_task(name: &str, task_list: State<'_, TaskList>) {
    if let Some(mut kv) = task_list.get_mut(name) {
        kv.0 = false;
    }
}
