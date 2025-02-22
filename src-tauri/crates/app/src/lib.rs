mod config;
mod core;

use core::{disable_task, start_core, stop_core, update_task};

use maa_core::{tauri_logger::Logger, TaskList};
use tauri::AppHandle;

pub(crate) type CommandResult = Result<(), String>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            build_logger(app.handle().clone());
            Ok(())
        })
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

fn build_logger(handle: AppHandle) {
    let logger = Logger::new(handle);
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(logger)))
        .init();
}
