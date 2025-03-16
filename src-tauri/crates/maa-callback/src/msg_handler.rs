use anyhow::Context;
use log::{error, info};

use crate::{
    callback::AsstMsgCode,
    callback_types::{SubTask, SubTaskExtraInfo, TaskChainInfo},
};

pub fn notify(code: AsstMsgCode, msg: &str) -> anyhow::Result<()> {
    match code {
        AsstMsgCode::InternalError => error!("内部错误"),
        AsstMsgCode::InitFailed => error!("初始化失败"),
        AsstMsgCode::AllTasksCompleted => info!("全部任务完成"),
        AsstMsgCode::TaskChainStart => {
            let task: TaskChainInfo =
                serde_json::from_str(msg).context("parse task chain start")?;
            let task_name = task.get_task_chain_name();
            info!("开始任务：{}", task_name);
        }
        AsstMsgCode::TaskChainCompleted => {
            let task: TaskChainInfo =
                serde_json::from_str(msg).context("parse task chain completed")?;
            let task_name = task.get_task_chain_name();
            info!("任务完成：{}", task_name);
        }
        AsstMsgCode::TaskChainStopped => {
            info!("已停止");
        }
        AsstMsgCode::TaskChainError => {
            let task: TaskChainInfo =
                serde_json::from_str(msg).context("parse task chain error")?;
            let task_name = task.get_task_chain_name();
            error!("任务失败：{}", task_name);
        }
        AsstMsgCode::SubTaskStart => {
            let sub_task: SubTask = serde_json::from_str(msg).context("parse sub task")?;
            sub_task
                .get_task_info()
                .context("get sub task Chinese info")?
                .inspect(|i| info!("{i}"));
        }
        AsstMsgCode::ConnectionInfo => {} // TODO: 截图时间 adb相关
        AsstMsgCode::SubTaskExtraInfo => {
            let sub_task_ex: SubTaskExtraInfo =
                serde_json::from_str(msg).context("parse sub task ex")?;
            sub_task_ex
                .to_exact_info()
                .context("get sub task Chinese ex info")?
                .inspect(|i| info!("{i}"));
        }
        AsstMsgCode::Unknown => error!("未知错误！"),
        _ => {}
    }
    Ok(())
}
