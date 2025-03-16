use std::ffi::{CStr, c_char, c_void};

use log::Level;
use maa_types::primitive::AsstMsgId;
use strum::{Display, FromRepr};

use crate::{
    callback_types::{ConnectionInfo, ConnectionInfoType},
    msg_handler,
};

/// default callback function
///
/// # Safety
///
/// This function is unsafe because it passes C pointer from DLL.
///
/// # Parameters
///
/// - `code`: message code, see `AsstMsgCode`
/// - `json_raw`: message details in JSON c_str pointer
pub unsafe extern "C" fn default_callback_log(
    code: AsstMsgId,
    json_raw: *const c_char,
    _: *mut c_void,
) {
    use std::str::FromStr;

    let json_str = unsafe { CStr::from_ptr(json_raw).to_str().unwrap() };
    let msg_type = AsstMsgCode::from_repr(code).unwrap_or_default();

    let level = if matches!(msg_type, AsstMsgCode::ConnectionInfo) {
        let info: ConnectionInfo = serde_json::from_str(json_str).unwrap();
        ConnectionInfoType::from_str(&info.what)
            .unwrap_or(ConnectionInfoType::Others)
            .level()
    } else {
        msg_type.level()
    };

    match level {
        Level::Error => log::error!("[{}] {}", msg_type, json_str),
        Level::Warn => log::warn!("[{}] {}", msg_type, json_str),
        Level::Info => log::info!("[{}] {}", msg_type, json_str),
        Level::Debug => log::debug!("[{}] {}", msg_type, json_str),
        Level::Trace => log::trace!("[{}] {}", msg_type, json_str),
    }

    // 简化log
    if let Err(e) = msg_handler::notify(msg_type, json_str) {
        log::error!("[{}] {}", msg_type, e)
    }
}

#[derive(Default, Debug, Display, Clone, Copy, FromRepr)]
#[repr(i32)]
pub enum AsstMsgCode {
    /* Global Info */
    /// 内部错误
    InternalError      = 0,
    /// 初始化失败
    InitFailed         = 1,
    /// 连接相关信息
    ConnectionInfo     = 2,
    /// 全部任务完成
    AllTasksCompleted  = 3,
    /// 外部异步调用信息
    AsyncCallInfo      = 4,
    /// 实例已销毁
    Destroyed          = 5,

    /* TaskChain Info */
    /// 任务链执行/识别错误
    TaskChainError     = 10000,
    /// 任务链开始
    TaskChainStart     = 10001,
    /// 任务链完成
    TaskChainCompleted = 10002,
    /// 任务链额外信息
    TaskChainExtraInfo = 10003,
    /// 任务链手动停止
    TaskChainStopped   = 10004,

    /* SubTask Info */
    /// 原子任务执行/识别错误
    SubTaskError       = 20000,
    /// 原子任务开始
    SubTaskStart       = 20001,
    /// 原子任务完成
    SubTaskCompleted   = 20002,
    /// 原子任务额外信息
    SubTaskExtraInfo   = 20003,
    /// 原子任务手动停止
    SubTaskStopped     = 20004,

    /// 未知状态
    #[default]
    Unknown            = -1,
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

    pub fn level(&self) -> Level {
        match self {
            AsstMsgCode::Unknown
            | AsstMsgCode::InitFailed
            | AsstMsgCode::InternalError
            | AsstMsgCode::TaskChainError
            | AsstMsgCode::SubTaskError => Level::Error,

            AsstMsgCode::Destroyed => Level::Warn,

            AsstMsgCode::TaskChainStart
            | AsstMsgCode::TaskChainCompleted
            | AsstMsgCode::TaskChainStopped
            | AsstMsgCode::SubTaskStart
            | AsstMsgCode::SubTaskCompleted
            | AsstMsgCode::SubTaskStopped
            | AsstMsgCode::TaskChainExtraInfo
            | AsstMsgCode::SubTaskExtraInfo
            | AsstMsgCode::AllTasksCompleted => Level::Info,

            AsstMsgCode::AsyncCallInfo | AsstMsgCode::ConnectionInfo => Level::Debug,
        }
    }
}
