mod core;
mod config;

use core::{disable_task, update_task, start_core, stop_core};

use maa_core::TaskList;

pub(crate) type CommandResult = Result<(),String>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_core,
            stop_core,
            update_task,
            disable_task,
        ])
        .manage(TaskList::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
