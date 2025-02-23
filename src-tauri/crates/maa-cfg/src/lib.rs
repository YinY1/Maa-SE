#![feature(if_let_guard)]
#![feature(let_chains)]
#![deny(warnings)]

use std::{
    env::current_exe,
    fs::{self, create_dir_all},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Context, anyhow};
use log::trace;
use maa_types::TaskType;
use strum::{Display, EnumString};

pub const CFG_DIR: &str = "config";
pub const DEFAULT_CFG_PATH: &str = "default";
pub const TASK_CFG: &str = "task.json";
pub const SETTINGS_CFG: &str = "settings.json";
pub const TOOLS_CFG: &str = "tools.json";
pub const CUSTOMS_CFG: &str = "customs.json";

/// 用于存放小工具识别后得到的内容类别，并非任务参数
#[derive(Debug, Display, EnumString, PartialEq)]
pub enum ToolType {
    /// 干员识别结果
    Operator,
    /// 仓库识别结果
    Material,
}

//TODO: tools等有没有嵌套结构
#[derive(Debug)]
pub enum ConfigType {
    Task(TaskType),
    Tools(ToolType),
    Settings,
    Custom(String),
}

impl FromStr for ConfigType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Settings" => Ok(Self::Settings),
            s if let Ok(t) = s.parse::<TaskType>() => Ok(Self::Task(t)),
            s if let Ok(t) = s.parse::<ToolType>() => Ok(Self::Tools(t)),
            s => Ok(Self::Custom(s.to_string())),
        }
    }
}

pub type ConfigCacheType = serde_json::Value;

pub struct Config {
    path: PathBuf,
    task: ConfigCacheType,
    tools: ConfigCacheType,
    settings: ConfigCacheType,
    customs: ConfigCacheType,
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
            task: load_json_obj(path.join(TASK_CFG)).with_context(|| format!("load {TASK_CFG}"))?,
            tools: load_json_obj(path.join(TOOLS_CFG))
                .with_context(|| format!("load {TOOLS_CFG}"))?,
            settings: load_json_obj(path.join(SETTINGS_CFG))
                .with_context(|| format!("load {SETTINGS_CFG}"))?,
            customs: load_json_obj(path.join(CUSTOMS_CFG))
                .with_context(|| format!("load {CUSTOMS_CFG}"))?,
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
    pub fn set_and_write(&mut self, cfg_type: ConfigType, contents: &str) -> anyhow::Result<()> {
        trace!("cache config and write");
        match cfg_type {
            ConfigType::Task(t) => {
                let path = self.path.join(TASK_CFG);
                Self::set_and_write_impl(&mut self.task, path, t.to_string(), contents)
                    .context("write task.json")
            }
            ConfigType::Tools(t) => {
                let path = self.path.join(TOOLS_CFG);
                Self::set_and_write_impl(&mut self.tools, path, t.to_string(), contents)
                    .context("write tools.json")
            }
            ConfigType::Settings => {
                self.settings = serde_json::from_str(contents)?;
                let path = self.path.join(SETTINGS_CFG);
                fs::write(path, self.settings.to_string()).context("write settings.json")
            }
            ConfigType::Custom(name) => {
                let path = self.path.join(CUSTOMS_CFG);
                Self::set_and_write_impl(&mut self.customs, path, name, contents)
            }
        }
    }

    fn set_and_write_impl(
        target_cache: &mut ConfigCacheType,
        path: PathBuf,
        key: String,
        contents: &str,
    ) -> anyhow::Result<()> {
        target_cache
            .as_object_mut()
            .expect("must be object")
            .insert(key, serde_json::from_str(contents)?);

        fs::write(path, target_cache.to_string()).context("write config")
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
