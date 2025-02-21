use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use anyhow::Context as _;
use maa_sys::Assistant;
use std::ffi::c_void;

pub mod callback;

pub type TaskList = Mutex<HashMap<String, String>>;

static STOP_SIGN: AtomicBool = AtomicBool::new(false);

pub const RES_PATH: &str = "D:\\tool\\MAA";
pub const ADB_PATH: &str = "D:\\MuMuPlayer-12.0\\shell\\adb.exe";
pub const ADB_ADDRESS: &str = "127.0.0.1:16384";

pub fn run_core(
    task_list: HashMap<String, String>,
    callback: maa_sys::binding::AsstApiCallback,
    arg: Option<*mut c_void>,
) -> anyhow::Result<()> {
    maa_sys::binding::load("MaaCore.dll")
        .map_err(|e| anyhow::anyhow!(e))
        .context("load core")?;
    Assistant::load_resource(RES_PATH).context("load resource")?;

    let assistant = Assistant::new(callback, arg);

    assistant
        .async_connect(ADB_PATH, ADB_ADDRESS, "", true)
        .context("connect")?;

    for (name, params) in task_list.into_iter() {
        assistant
            .append_task(name.as_str(), params.as_str())
            .with_context(|| format! {"append task: {name}"})?;
    }

    assistant.start().context("start")?;
    while assistant.running() && !STOP_SIGN.load(Ordering::Acquire) {
        sleep(Duration::from_millis(300)); // TODO: 优化sleep
    }
    STOP_SIGN.store(false, Ordering::Release);
    assistant.stop().context("stop")?;
    Ok(())
}

#[cfg(feature = "tauri-handle")]
pub fn run_core_tauri(
    app: tauri::AppHandle,
    task_list: HashMap<String, String>,
) -> anyhow::Result<()> {
    let ptr = Box::into_raw(Box::new(app));
    let res = run_core(
        task_list,
        Some(callback::default_callback_tauri),
        Some(ptr as *mut c_void),
    );
    let _app = unsafe { Box::from_raw(ptr) };
    res
}

pub fn stop_core() {
    STOP_SIGN.store(true, Ordering::Release);
}
