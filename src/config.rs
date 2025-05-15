use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
};

use crate::cookies::CookieItem;
use crate::error::BiliError;

/// 应用配置
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub user_id: Option<u64>,
    pub username: Option<String>,
    pub refresh_token: Option<String>,
    pub cookies: Option<Vec<CookieItem>>,
    pub last_login: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_id: None,
            username: None,
            refresh_token: None,
            cookies: None,
            last_login: None,
        }
    }
}

/// 获取项目目录
#[allow(dead_code)]
pub fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("rs", "bilicookies", "bilicookies-rs")
        .ok_or_else(|| BiliError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "无法创建项目目录"
        )).into())
}

/// 获取配置文件路径
#[allow(dead_code)]
pub fn get_config_path() -> Result<PathBuf> {
    let project_dirs = get_project_dirs()?;
    let config_dir = project_dirs.config_dir();
    
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }
    
    Ok(config_dir.join("config.json"))
}

/// 读取配置
#[allow(dead_code)]
pub fn read_config() -> Result<Config> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        return Ok(Config::default());
    }
    
    let config_str = fs::read_to_string(config_path)?;
    Ok(serde_json::from_str(&config_str)?)
}

/// 保存配置
#[allow(dead_code)]
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    let config_str = serde_json::to_string_pretty(config)?;
    fs::write(config_path, config_str)?;
    Ok(())
}

/// 保存cookies到配置
#[allow(dead_code)]
pub fn save_cookies(cookies: &[CookieItem], user_id: u64, username: &str, refresh_token: &str) -> Result<()> {
    let mut config = read_config()?;
    
    config.user_id = Some(user_id);
    config.username = Some(username.to_string());
    config.refresh_token = Some(refresh_token.to_string());
    config.cookies = Some(cookies.to_vec());
    config.last_login = Some(chrono::Local::now().to_string());
    
    save_config(&config)
} 