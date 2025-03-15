use anyhow::Context;
use log::{error, info};
use serde::Deserialize;
use serde_json::Value;
use strum::AsRefStr;

use crate::callback::AsstMsgCode;

const PROCESS_TASK_NAME: &str = "ProcessTask";

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
        AsstMsgCode::TaskChainError => {
            let task: TaskChainInfo =
                serde_json::from_str(msg).context("parse task chain error")?;
            let task_name = task.get_task_chain_name();
            error!("任务失败：{}", task_name);
        }
        AsstMsgCode::SubTaskStart => {
            let sub_task: SubTask = serde_json::from_str(msg).context("parse sub task")?;
            let info = sub_task
                .get_task_info()
                .context("get sub task Chinese info")?;
            if let Some(info) = info {
                info!("{}", info)
            }
        }
        AsstMsgCode::Unknown => error!("未知错误！"),
        // TODO
        AsstMsgCode::SubTaskError => {}
        AsstMsgCode::SubTaskCompleted => {}
        AsstMsgCode::ConnectionInfo => {}
        AsstMsgCode::SubTaskExtraInfo => {}
        AsstMsgCode::SubTaskStopped => {}
        AsstMsgCode::AsyncCallInfo => {}
        AsstMsgCode::TaskChainExtraInfo => {}
        AsstMsgCode::TaskChainStopped => {}
        AsstMsgCode::Destroyed => {}
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct TaskChainInfo {
    taskchain: TaskChain,
}

impl TaskChainInfo {
    pub fn get_task_chain_name(&self) -> &str {
        self.taskchain.as_ref()
    }
}

#[derive(Debug, Deserialize, AsRefStr)]
pub enum TaskChain {
    #[strum(serialize = "开始唤醒")]
    StartUp,
    #[strum(serialize = "关闭游戏")]
    CloseDown,
    #[strum(serialize = "刷理智")]
    Fight,
    #[strum(serialize = "信用点及购物")]
    Mall,
    #[strum(serialize = "自动公招")]
    Recruit,
    #[strum(serialize = "基建换班")]
    Infrast,
    #[strum(serialize = "领取日常奖励")]
    Award,
    #[strum(serialize = "无限刷肉鸽")]
    Roguelike,
    #[strum(serialize = "自动抄作业")]
    Copilot,
    #[strum(serialize = "自动抄保全作业")]
    SSSCopilot,
    #[strum(serialize = "仓库识别")]
    Depot,
    #[strum(serialize = "干员 box 识别")]
    OperBox,
    #[strum(serialize = "生息演算")]
    ReclamationAlgorithm,
    #[strum(serialize = "自定义任务")]
    Custom,
    #[strum(serialize = "单步任务")]
    SingleStep,
    #[strum(serialize = "视频识别任务")]
    VideoRecognition,
    #[strum(serialize = "调试")]
    Debug,
}

#[derive(Debug, Deserialize)]
pub struct SubTask {
    subtask: String, // 子任务名
    details: Value,  // 详情
}

impl SubTask {
    pub fn get_task_info(&self) -> anyhow::Result<Option<&str>> {
        if self.subtask != PROCESS_TASK_NAME {
            return Ok(None);
        }
        self.details["task"]
            .as_str()
            .ok_or(anyhow::anyhow!("process task not exists"))
            .map(Self::match_task_type)
    }

    pub fn match_task_type(task_type: &str) -> Option<&'static str> {
        match task_type {
            "StartButton2" => Some("开始战斗"),
            "MedicineConfirm" => Some("使用理智药"),
            "ExpiringMedicineConfirm" => Some("使用 48 小时内过期的理智药"),
            "StoneConfirm" => Some("碎石"),
            "RecruitRefreshConfirm" => Some("公招刷新标签"),
            "RecruitConfirm" => Some("公招确认招募"),
            "RecruitNowConfirm" => Some("公招使用加急许可"),
            "ReportToPenguinStats" => Some("汇报到企鹅数据统计"),
            "ReportToYituliu" => Some("汇报到一图流大数据"),
            "InfrastDormDoubleConfirmButton" => Some("宿舍二次确认"),
            "StartExplore" => Some("肉鸽开始探索"),
            "StageTraderInvestConfirm" => Some("肉鸽投资了源石锭"),
            "StageTraderInvestSystemFull" => Some("肉鸽投资达到了游戏上限"),
            "ExitThenAbandon" => Some("肉鸽放弃了本次探索"),
            "MissionCompletedFlag" => Some("肉鸽战斗完成"),
            "MissionFailedFlag" => Some("肉鸽战斗失败"),
            "StageTraderEnter" => Some("肉鸽关卡：诡异行商"),
            "StageSafeHouseEnter" => Some("肉鸽关卡：安全的角落"),
            "StageEncounterEnter" => Some("肉鸽关卡：不期而遇/古堡馈赠"),
            "StageCombatDpsEnter" => Some("肉鸽关卡：普通作战"),
            "StageEmergencyDps" => Some("肉鸽关卡：紧急作战"),
            "StageDreadfulFoe" => Some("肉鸽关卡：险路恶敌"),
            "StartGameTask" => Some("打开客户端"),
            _ => None,
        }
    }
}
