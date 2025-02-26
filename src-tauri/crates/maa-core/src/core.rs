use std::{
    env::current_exe,
    ffi::c_void,
    sync::atomic::{AtomicBool, Ordering},
    thread::sleep,
    time::Duration,
};

use anyhow::{Context, anyhow};
use log::{debug, trace};
use maa_cfg::task::TaskQueue;
use maa_sys::Assistant;

use crate::{ADB_PATH, DEFAULT_ADB_ADDRESS};

static STOP_SIGN: AtomicBool = AtomicBool::new(false);

/// run all tasks with given queue and callback
pub fn run_core(
    tasks: TaskQueue,
    callback: maa_sys::binding::AsstApiCallback,
    arg: Option<*mut c_void>,
) -> anyhow::Result<()> {
    trace!("load MaaCore");
    maa_sys::binding::load("MaaCore.dll")
        .map_err(|e| anyhow!(e))
        .context("load core")?;

    trace!("load resource");
    let exe_path = current_exe().context("get exe path")?;
    Assistant::load_resource(exe_path.parent().unwrap()).context("load resource")?;

    let assistant = Assistant::new(callback, arg);

    trace!("connect adb");
    assistant
        .async_connect(ADB_PATH, DEFAULT_ADB_ADDRESS, "", true)
        .context("connect")?;

    trace!("append tasks");

    for (name, params) in tasks {
        let id = assistant
            .append_task(name.as_str(), params.as_str())
            .with_context(|| format!("append task {name}"))?;
        debug!("append task '{}' (id: {})", name, id);
    }

    trace!("run tasks");
    assistant.start().context("start")?;
    while assistant.running() && !STOP_SIGN.load(Ordering::Acquire) {
        sleep(Duration::from_millis(300)); // TODO: 优化sleep
    }
    STOP_SIGN.store(false, Ordering::Release);
    trace!("stop asst");
    assistant.stop().context("stop")
}

#[cfg(feature = "tauri-handle")]
pub fn run_core_tauri(tasks: TaskQueue) -> anyhow::Result<()> {
    run_core(tasks, Some(crate::callback::default_callback_log), None)
}

pub fn stop_core() {
    trace!("user stop manually");
    STOP_SIGN.store(true, Ordering::Release);
}
