use std::ffi::{CStr, c_char, c_void};

use maa_types::primitive::AsstMsgId;

#[cfg(feature = "tauri-handle")]
/// default callback function
///
/// # Safety
///
/// This function is unsafe because it passes C pointer from DLL and
/// dereferences the `app` pointer.
///
/// # Parameters
///
/// - `code`: message code, see `AsstMsgCode`
/// - `json_raw`: message details in JSON c_str pointer
/// - `app`: pointer to `tauri::AppHandle`
pub unsafe extern "C" fn default_callback_tauri(
    code: AsstMsgId,
    json_raw: *const c_char,
    app: *mut c_void,
) {
    use tauri::{AppHandle, Emitter as _};
    unsafe {
        let json_str = CStr::from_ptr(json_raw).to_str().unwrap();
        if let Some(handle) = (app as *mut AppHandle).as_ref() {
            handle
                .emit(
                    "callback-log",
                    format!("code: {}, details {}", code, json_str),
                )
                .unwrap();
        }
    }
}

pub enum AsstMsgCode {
    /* Global Info */
    InternalError = 0,     // 内部错误
    InitFailed = 1,        // 初始化失败
    ConnectionInfo = 2,    // 连接相关信息
    AllTasksCompleted = 3, // 全部任务完成
    AsyncCallInfo = 4,     // 外部异步调用信息
    Destroyed = 5,         // 实例已销毁

    /* TaskChain Info */
    TaskChainError = 10000,     // 任务链执行/识别错误
    TaskChainStart = 10001,     // 任务链开始
    TaskChainCompleted = 10002, // 任务链完成
    TaskChainExtraInfo = 10003, // 任务链额外信息
    TaskChainStopped = 10004,   // 任务链手动停止

    /* SubTask Info */
    SubTaskError = 20000,     // 原子任务执行/识别错误
    SubTaskStart = 20001,     // 原子任务开始
    SubTaskCompleted = 20002, // 原子任务完成
    SubTaskExtraInfo = 20003, // 原子任务额外信息
    SubTaskStopped = 20004,   // 原子任务手动停止
}

impl From<AsstMsgId> for AsstMsgCode {
    fn from(id: AsstMsgId) -> Self {
        match id {
            0 => AsstMsgCode::InternalError,
            1 => AsstMsgCode::InitFailed,
            2 => AsstMsgCode::ConnectionInfo,
            3 => AsstMsgCode::AllTasksCompleted,
            4 => AsstMsgCode::AsyncCallInfo,
            5 => AsstMsgCode::Destroyed,
            10000 => AsstMsgCode::TaskChainError,
            10001 => AsstMsgCode::TaskChainStart,
            10002 => AsstMsgCode::TaskChainCompleted,
            10003 => AsstMsgCode::TaskChainExtraInfo,
            10004 => AsstMsgCode::TaskChainStopped,
            20000 => AsstMsgCode::SubTaskError,
            20001 => AsstMsgCode::SubTaskStart,
            20002 => AsstMsgCode::SubTaskCompleted,
            20003 => AsstMsgCode::SubTaskExtraInfo,
            20004 => AsstMsgCode::SubTaskStopped,
            _ => unreachable!("Invalid AsstMsgId: {}", id),
        }
    }
}

impl AsstMsgCode {
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            AsstMsgCode::AllTasksCompleted
                | AsstMsgCode::TaskChainCompleted
                | AsstMsgCode::SubTaskCompleted
        )
    }
}
