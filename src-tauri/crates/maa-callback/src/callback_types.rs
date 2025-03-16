use std::fmt::Write;

use anyhow::Context;
pub use depot_types::*;
pub use facility_types::*;
use log::Level;
pub use oper_box_types::*;
pub use recruit_types::*;
use serde::Deserialize;
use serde_json::Value;
pub use stage_types::*;
use strum::{AsRefStr, Display, EnumString};

const PROCESS_TASK_NAME: &str = "ProcessTask";

#[derive(Debug, Display, EnumString)]
pub enum ConnectionInfoType {
    /// 已连接，注意此时的 uuid 字段值为空（下一步才是获取）
    Connected,
    /// 已获取到设备唯一码
    UuidGot,
    /// 分辨率不被支持
    UnsupportedResolution,
    /// 分辨率获取错误
    ResolutionError,
    /// 分辨率获取成功
    ResolutionGot,
    /// 连接断开（adb / 模拟器 炸了），正在重连
    Reconnecting,
    /// 连接断开（adb / 模拟器 炸了），重连成功
    Reconnected,
    /// 连接断开（adb / 模拟器 炸了），并重试失败
    Disconnect,
    /// 截图失败（adb / 模拟器 炸了），并重试失败
    ScreencapFailed,
    /// 不支持的触控模式
    TouchModeNotAvailable,
    /// 其他
    Others,
}

impl ConnectionInfoType {
    pub fn level(&self) -> Level {
        match self {
            ConnectionInfoType::UnsupportedResolution
            | ConnectionInfoType::ResolutionError
            | ConnectionInfoType::ScreencapFailed
            | ConnectionInfoType::TouchModeNotAvailable => Level::Error,

            ConnectionInfoType::Reconnecting => Level::Warn,

            ConnectionInfoType::Connected
            | ConnectionInfoType::Reconnected
            | ConnectionInfoType::Disconnect => Level::Info,

            ConnectionInfoType::ResolutionGot
            | ConnectionInfoType::UuidGot
            | ConnectionInfoType::Others => Level::Debug,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConnectionInfo {
    pub what: String,         // 信息类型
    pub why: Option<String>,  // 信息原因
    pub uuid: Option<String>, // 设备唯一码（连接失败时为空）
    pub details: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct TaskChainInfo {
    taskchain: TaskChainType,
}

impl TaskChainInfo {
    pub fn get_task_chain_name(&self) -> &str {
        self.taskchain.as_ref()
    }
}

#[derive(Debug, Deserialize, AsRefStr)]
pub enum TaskChainType {
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
            .map(Self::as_task_type_cn)
    }

    pub fn as_task_type_cn(task_type: &str) -> Option<&'static str> {
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

#[derive(Deserialize, Debug)]
pub struct SubTaskExtraInfo {
    what: String,   // 信息类型
    details: Value, // 信息详情
}

impl SubTaskExtraInfo {
    pub fn to_exact_info(self) -> anyhow::Result<String> {
        match self.what.as_str() {
            "StageDrops" => self.to_drops_info(),
            "RecruitResult" => self.to_recruit_info(),
            // TODO 其他结果
            _ => Ok(String::new()),
        }
    }

    pub fn to_drops_info(self) -> anyhow::Result<String> {
        let stage_drops: StageDrops =
            serde_json::from_value(self.details).context("parse stage drops")?;

        let mut s = String::new();
        writeln!(
            s,
            "{}: {}星通过\n材料掉落:",
            stage_drops.stage.stage_code, stage_drops.stars
        )?;

        stage_drops
            .stats
            .into_iter()
            .try_for_each(|stat| writeln!(s, "{}", stat))?;

        Ok(s)
    }

    pub fn to_recruit_info(self) -> anyhow::Result<String> {
        let recuit_result: RecruitResult =
            serde_json::from_value(self.details).context("parse recruit result")?;

        let mut s = String::new();
        writeln!(
            s,
            "公招标签（{}星）:{:?}",
            recuit_result.level, recuit_result.tags
        )?;
        Ok(s)
    }
}

/// 关卡掉落相关json
#[allow(unused)]
pub mod stage_types {
    use std::fmt::{Debug, Display};

    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct StageDrops {
        pub stage: Stage,
        pub stars: u8,
        pub stats: Vec<Stat>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Stage {
        pub stage_code: String,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Stat {
        item_name: String,
        quantity: u32,
        add_quantity: u32,
    }

    impl Display for Stat {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(
                f,
                "'{}'*{} (+{})",
                self.item_name, self.quantity, self.add_quantity
            )
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct PenguinId {
        id: String,
    }

    /// 自动作战关卡的信息
    #[derive(Deserialize, Debug)]
    pub struct StageInfo {
        name: String, // 关卡名
    }
}

/// 公招相关json
#[allow(unused)] //TODO: 公招识别模块相关暂未实现
pub mod recruit_types {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct RecruitTagsDetected {
        tags: Vec<String>,
    }

    #[derive(Deserialize, Debug)]
    pub struct RecruitSpecialTag {
        tag: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Oper {
        name: String,
        level: u32,
    }

    #[derive(Deserialize, Debug)]
    pub struct ResultEntry {
        tags: Vec<String>,
        level: u8,
        opers: Vec<Oper>,
    }

    #[derive(Deserialize, Debug)]
    pub struct RecruitResult {
        pub tags: Vec<String>,
        pub level: u8,
        pub result: Vec<ResultEntry>,
    }

    #[derive(Deserialize, Debug)]
    pub struct RecruitTagsRefreshed {
        count: u8,
        refresh_limit: u8,
    }

    #[derive(Deserialize, Debug)]
    pub struct RecruitNoPermit {
        #[serde(rename = "continue")]
        continue_: bool,
    }

    pub type RecruitTagsSelected = RecruitTagsDetected;
}

/// 基建相关json
#[allow(unused)]
pub mod facility_types {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct EnterFacility {
        facility: String, // 设施名
        index: u16,       // 设施序号
    }

    #[derive(Deserialize, Debug)]
    pub struct NotEnoughStaff {
        facility: String, // 设施名
        index: u16,       // 设施序号
    }

    #[derive(Deserialize, Debug)]
    pub struct ProductOfFacility {
        product: String,  // 产物名
        facility: String, // 设施名
        index: u8,        // 设施序号
    }
}

/// 仓库识别相关json
#[allow(unused)]
pub mod depot_types {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Depot {
        done: bool,
        arkplanner: ArkPlanner,
        lolicon: Lolicon,
    }

    #[derive(Deserialize, Debug)]
    pub struct Item {
        id: String,
        have: u32,
        name: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct ArkPlannerObject {
        items: Vec<Item>,
        #[serde(rename = "@type")]
        type_: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct ArkPlanner {
        object: ArkPlannerObject,
        data: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct LoliconObject {
        #[serde(flatten)]
        items: std::collections::HashMap<String, u32>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Lolicon {
        object: LoliconObject,
        data: String,
    }
}

/// 干员识别相关json
#[allow(unused)]
pub mod oper_box_types {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct OperBox {
        done: bool,
        all_oper: Vec<OperInfo>,
        own_opers: Vec<OwnOperInfo>,
    }

    #[derive(Deserialize, Debug)]
    pub struct OperInfo {
        id: String,
        name: String,
        own: bool,
        rarity: u8,
    }

    #[derive(Deserialize, Debug)]
    pub struct OwnOperInfo {
        id: String,
        name: String,
        own: bool,
        elite: u8,
        level: u8,
        potential: u8,
        rarity: u8,
    }
}
