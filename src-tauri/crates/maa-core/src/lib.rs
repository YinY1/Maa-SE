#![feature(inherent_str_constructors)]

use std::{
    ffi::c_void,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use anyhow::Context as _;
use dashmap::DashMap;
use log::trace;
use maa_sys::Assistant;

pub mod callback;
#[cfg(feature = "tauri-handle")]
pub mod tauri_logger;

/// # key-value
/// { name : (is_enable, json_params) }
pub type TaskList = Arc<DashMap<String, (bool, String)>>;

static STOP_SIGN: AtomicBool = AtomicBool::new(false);

pub const ADB_PATH: &str = "D:\\MuMuPlayer-12.0\\shell\\adb.exe";
pub const DEFAULT_ADB_ADDRESS: &str = "127.0.0.1:16384";

pub fn run_core(
    task_list: TaskList,
    callback: maa_sys::binding::AsstApiCallback,
    arg: Option<*mut c_void>,
) -> anyhow::Result<()> {
    trace!("load MaaCore");
    maa_sys::binding::load("MaaCore.dll")
        .map_err(|e| anyhow::anyhow!(e))
        .context("load core")?;

    trace!("load resource");
    Assistant::load_resource(".").context("load resource")?;

    let assistant = Assistant::new(callback, arg);

    trace!("connect adb");
    assistant
        .async_connect(ADB_PATH, DEFAULT_ADB_ADDRESS, "", true)
        .context("connect")?;

    trace!("append task");
    for kv in task_list.iter().filter(|kv| kv.0) {
        let (name, (_, params)) = kv.pair();
        assistant
            .append_task(name.as_str(), params.as_str())
            .with_context(|| format! {"append task: {name}"})?;
    }

    trace!("run tasks");
    assistant.start().context("start")?;
    while assistant.running() && !STOP_SIGN.load(Ordering::Acquire) {
        sleep(Duration::from_millis(300)); // TODO: 优化sleep
    }
    STOP_SIGN.store(false, Ordering::Release);
    trace!("stop asst");
    assistant.stop().context("stop")?;
    Ok(())
}

#[cfg(feature = "tauri-handle")]
pub fn run_core_tauri(task_list: TaskList) -> anyhow::Result<()> {
    run_core(task_list, Some(callback::default_callback_log), None)
}

pub fn stop_core() {
    trace!("user stop manually");
    STOP_SIGN.store(true, Ordering::Release);
}
