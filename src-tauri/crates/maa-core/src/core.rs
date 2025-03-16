use std::{
    env::{
        consts::{DLL_PREFIX, DLL_SUFFIX},
        current_exe,
    },
    ffi::c_void,
    sync::OnceLock,
    time::Duration,
};

use anyhow::{Context, anyhow, bail};
use crossbeam_channel::select;
use log::{debug, trace};
use maa_callback::callback::STOP_CHAN;
use maa_cfg::task::TaskQueue;
use maa_sys::Assistant;

use crate::{ADB_PATH, DEFAULT_ADB_ADDRESS};

const MAA_CORE: &str = constcat::concat!(DLL_PREFIX, "MaaCore", DLL_SUFFIX);
static LOAD_CORE: OnceLock<()> = OnceLock::new();

/// run all tasks with given queue and callback
pub fn run_core(
    tasks: TaskQueue,
    callback: maa_sys::binding::AsstApiCallback,
    arg: Option<*mut c_void>,
) -> anyhow::Result<()> {
    LOAD_CORE
        .get_or_try_init(|| {
            trace!("load MaaCore");
            maa_sys::binding::load(MAA_CORE)
                .map_err(|e| anyhow!(e))
                .context("load core")
        })
        .context("once load core")?;

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
    loop {
        select! {
            recv(STOP_CHAN.rx) -> msg => {
                if let Err(e) = msg {
                    bail!(e);
                }
                break;
            }
            default(Duration::from_secs(1)) => if !assistant.running() {
                break;
            }
        }
    }
    trace!("stop asst");
    assistant.stop().context("stop")
}

pub fn reload_core() -> anyhow::Result<()> {
    trace!("unload MaaCore");
    maa_sys::binding::unload();
    trace!("load MaaCore");
    maa_sys::binding::load(MAA_CORE)
        .map_err(|e| anyhow!(e))
        .context("load core")
}

#[cfg(feature = "tauri-handle")]
pub fn run_core_tauri(tasks: TaskQueue) -> anyhow::Result<()> {
    use maa_callback::callback::default_callback_log;

    run_core(tasks, Some(default_callback_log), None)
}
