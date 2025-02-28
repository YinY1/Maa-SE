use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::ConfigValue;

pub type TaskQueue = Vec<(String, String)>;

/// 既可以作为gui的配置项，也可以作为执行任务的参数名
#[derive(Debug, Display)]
pub enum TaskType {
    Daily(DailyTaskType),
    Extra(ExtraTaskType),
    Custom(String),
}

impl FromStr for TaskType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if let Ok(t) = s.parse() => Ok(Self::Daily(t)),
            s if let Ok(t) = s.parse() => Ok(Self::Extra(t)),
            _ => Ok(Self::Custom(s.to_string())),
        }
    }
}

#[derive(Debug, Display, EnumString)]
pub enum DailyTaskType {
    StartUp,
    CloseDown,
    Fight,
    Recruit,
    Infrast,
    Mall,
    Award,
    Roguelike,
    Reclamation,
}

#[derive(Debug, Display, EnumString)]
pub enum ExtraTaskType {
    Custom,
    Copilot,
    SSSCopilot,
    Depot,
    OperBox,
    SingleStep,
    VideoRecognition,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Parameters {
    pub enable: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, ConfigValue>,
}
