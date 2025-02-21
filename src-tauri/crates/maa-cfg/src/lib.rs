#![feature(if_let_guard)]

use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, anyhow};
use maa_types::TaskType;

pub const CFG_DIR: &str = "config";
pub const DEFAULT_CFG_PATH: &str = "default";
pub const TASK_CFG: &str = "task.json";
pub const TOOLS_CFG: &str = "tools.json";
pub const SETTINGS_CFG: &str = "settings.json";

//TODO: tools等有没有嵌套结构
#[derive(Debug)]
pub enum ConfigType {
    Task(TaskType),
    Tools,
    Settings,
}

impl FromStr for ConfigType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Tools" => Ok(Self::Tools),
            "Settings" => Ok(Self::Settings),
            s if let Ok(t) = s.parse::<TaskType>() => Ok(Self::Task(t)),
            s => Err(format!("unsupported type '{s}'")),
        }
    }
}

pub struct Config {
    path: PathBuf,
    task: serde_json::Value,
    tools: serde_json::Value,
    settings: serde_json::Value,
}

impl Config {
    pub fn load(cfg_group: Option<String>) -> anyhow::Result<Self> {
        let path = Path::new(CFG_DIR).join(cfg_group.unwrap_or(DEFAULT_CFG_PATH.to_string()));

        let task = Self::load_from_file(path.join(TASK_CFG)).context("load task.json")?;
        let tools = Self::load_from_file(path.join(TOOLS_CFG)).context("load tools.json")?;
        let settings =
            Self::load_from_file(path.join(SETTINGS_CFG)).context("load settings.json")?;

        Ok(Self {
            path,
            task,
            tools,
            settings,
        })
    }

    fn load_from_file(path: PathBuf) -> anyhow::Result<serde_json::Value> {
        if !fs::exists(&path).context("judge cfg exists")? {
            return Ok(serde_json::Value::Object(Default::default()));
        }

        let content = fs::read_to_string(path).context("read cfg")?;
        match serde_json::from_str(&content) {
            Ok(v) => Ok(v),
            Err(e) if e.is_eof() => Ok(serde_json::Value::Object(Default::default())),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub fn set(&mut self, cfg_type: ConfigType, content: &str) -> anyhow::Result<()> {
        match cfg_type {
            ConfigType::Task(t) => {
                self.task
                    .as_object_mut()
                    .expect("must be object")
                    .insert(t.to_string(), serde_json::from_str(content)?);

                let path = self.path.join(TASK_CFG);
                fs::write(path, self.task.to_string()).context("write task.json")
            }
            ConfigType::Tools => {
                self.tools = serde_json::from_str(content)?;

                let path = self.path.join(TOOLS_CFG);
                fs::write(path, self.tools.to_string()).context("write tools.json")
            }
            ConfigType::Settings => {
                self.settings = serde_json::from_str(content)?;
                let path = self.path.join(SETTINGS_CFG);
                fs::write(path, self.settings.to_string()).context("write settings.json")
            }
        }
    }
}
