#![feature(if_let_guard)]
#![feature(let_chains)]
#![deny(warnings)]

pub mod task;

use std::{
    env::current_exe,
    fs::{self, create_dir_all},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Context, anyhow};
use log::trace;
use strum::{Display, EnumString};
pub use task::*;

pub const CFG_DIR: &str = "config";
pub const DEFAULT_CFG_PATH: &str = "default";
pub const DAILY_CFG: &str = "daily.json";
pub const EXTRA_TASK_CFG: &str = "extra-task.json";
pub const SETTINGS_CFG: &str = "settings.json";
pub const TOOL_STORAGE: &str = "tool-storage.json";
pub const CUSTOMS_CFG: &str = "customs.json";
pub const CUSTOMS_STORAGE: &str = "custom-storage.json";

/// 存放数据，而非gui配置本身
#[derive(Debug, Display, EnumString)]
pub enum Storage {
    /// 小工具识别结果，包括干员识别和仓库识别等
    Tool,
    /// 用户自定义任务名的数据存储
    Custom(String),
}

#[derive(Debug, Display)]
pub enum ConfigType {
    Task(TaskType),
    Storage(Storage),
    Settings,
}

impl FromStr for ConfigType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Settings" => Ok(Self::Settings),
            s if let Ok(t) = s.parse() => Ok(Self::Task(t)),
            s if let Ok(t) = s.parse() => Ok(Self::Storage(t)),
            _ => Err(s.to_string()),
        }
    }
}

pub type ConfigValue = serde_json::Value;

pub struct Config {
    path: PathBuf,
    daily_task: ConfigValue,
    extra_task: ConfigValue,
    custom_task: ConfigValue,
    custom_storage: ConfigValue,
    tools: ConfigValue,
    settings: ConfigValue,
}

impl Config {
    pub fn load(cfg_group: Option<String>) -> anyhow::Result<Self> {
        let path = current_exe()
            .context("get exe path")?
            .parent()
            .unwrap()
            .join(CFG_DIR)
            .join(cfg_group.unwrap_or(DEFAULT_CFG_PATH.to_string()));

        if let Err(e) = create_dir_all(&path)
            && !matches!(e.kind(), std::io::ErrorKind::AlreadyExists)
        {
            anyhow::bail!(e)
        }

        Ok(Self {
            daily_task: load_json_obj(path.join(DAILY_CFG))
                .with_context(|| format!("load {DAILY_CFG}"))?,
            tools: load_json_obj(path.join(TOOL_STORAGE))
                .with_context(|| format!("load {TOOL_STORAGE}"))?,
            settings: load_json_obj(path.join(SETTINGS_CFG))
                .with_context(|| format!("load {SETTINGS_CFG}"))?,
            custom_task: load_json_obj(path.join(CUSTOMS_CFG))
                .with_context(|| format!("load {CUSTOMS_CFG}"))?,
            custom_storage: load_json_obj(path.join(CUSTOMS_STORAGE))
                .with_context(|| format!("load {CUSTOMS_CFG}"))?,
            extra_task: load_json_obj(path.join(EXTRA_TASK_CFG))
                .with_context(|| format!("load {EXTRA_TASK_CFG}"))?,
            path,
        })
    }

    /// 更新缓存并写入本地配置文件
    ///
    /// # Parameter
    ///
    /// `cfg_type`: {name}.json
    ///
    /// `contents`: json string
    pub fn set_and_write(
        &mut self,
        cfg_type: ConfigType,
        params: Parameters,
    ) -> anyhow::Result<()> {
        trace!("cache config and write");
        match cfg_type {
            ConfigType::Task(TaskType::Daily(t)) => {
                let path = self.path.join(DAILY_CFG);
                Self::set_and_write_impl(&mut self.daily_task, path, t.to_string(), params)
                    .context("write daily.json")
            }
            ConfigType::Task(TaskType::Extra(t)) => {
                let path = self.path.join(EXTRA_TASK_CFG);
                Self::set_and_write_impl(&mut self.extra_task, path, t.to_string(), params)
                    .context("write extra_task.json")
            }
            ConfigType::Task(TaskType::Custom(t)) => {
                let path = self.path.join(CUSTOMS_CFG);
                Self::set_and_write_impl(&mut self.custom_task, path, t, params)
                    .context("write customs.json")
            }
            ConfigType::Storage(Storage::Tool) => {
                self.tools = serde_json::to_value(params)?;
                let path = self.path.join(TOOL_STORAGE);
                fs::write(path, self.tools.to_string()).context("write tool-storage.json")
            }
            ConfigType::Storage(Storage::Custom(t)) => {
                let path = self.path.join(CUSTOMS_STORAGE);
                Self::set_and_write_impl(&mut self.custom_storage, path, t, params)
                    .context("write custom-storage.json")
            }
            ConfigType::Settings => {
                self.settings = serde_json::to_value(params)?;
                let path = self.path.join(SETTINGS_CFG);
                fs::write(path, self.settings.to_string()).context("write settings.json")
            }
        }
    }

    fn set_and_write_impl(
        target_cache: &mut ConfigValue,
        path: PathBuf,
        key: String,
        params: Parameters,
    ) -> anyhow::Result<()> {
        let value = serde_json::to_value(params).context("serde params to json value")?;
        target_cache
            .as_object_mut()
            .expect("must be object")
            .insert(key, value);

        fs::write(path, serde_json::to_string_pretty(target_cache).unwrap()).context("write config")
    }

    pub fn available_daily_tasks(&self) -> TaskQueue {
        Self::available_tasks_impl(&self.daily_task)
    }

    fn available_tasks_impl(cache: &ConfigValue) -> TaskQueue {
        cache
            .as_object()
            .unwrap()
            .iter()
            .filter(|(_, params)| params["enable"].as_bool().unwrap())
            .map(|(name, params)| (name.to_string(), params.to_string()))
            .collect()
    }
}

pub fn load_json_obj(path: PathBuf) -> anyhow::Result<serde_json::Value> {
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
