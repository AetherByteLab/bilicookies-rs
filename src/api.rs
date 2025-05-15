use anyhow::Result;
use reqwest::{Client, Response};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::BiliError;

// API 路径
const QR_CODE_GENERATE_URL: &str = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
const QR_CODE_POLL_URL: &str = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";
const USER_INFO_URL: &str = "https://api.bilibili.com/x/web-interface/nav";

/// 创建HTTP客户端，带有cookie jar，并启用cookie存储
pub fn create_client() -> Result<Client> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .cookie_store(true)
        .build()?;
    
    Ok(client)
}

/// 获取当前时间戳（毫秒）
pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("获取时间失败")
        .as_millis() as u64
}

/// 二维码生成响应
#[derive(Debug, Deserialize)]
pub struct QrCodeGenerateResponse {
    pub code: i32,
    pub message: String,
    #[allow(dead_code)]
    pub ttl: i32,
    pub data: Option<QrCodeGenerateData>,
}

#[derive(Debug, Deserialize)]
pub struct QrCodeGenerateData {
    pub url: String,
    pub qrcode_key: String,
}

/// 二维码轮询响应
#[derive(Debug, Deserialize)]
pub struct QrCodePollResponse {
    pub code: i32,
    pub message: String,
    #[allow(dead_code)]
    pub ttl: i32,
    pub data: Option<QrCodePollData>,
}

#[derive(Debug, Deserialize)]
pub struct QrCodePollData {
    #[allow(dead_code)]
    pub url: String,
    pub refresh_token: String,
    #[allow(dead_code)]
    pub timestamp: u64,
    pub code: i32,
    #[allow(dead_code)]
    pub message: String,
}

/// 用户信息响应
#[derive(Debug, Deserialize)]
pub struct UserInfoResponse {
    pub code: i32,
    pub message: String,
    #[allow(dead_code)]
    pub ttl: i32,
    pub data: Option<UserInfoData>,
}

#[derive(Debug, Deserialize)]
pub struct UserInfoData {
    #[serde(rename = "isLogin")]
    pub is_login: bool,
    pub mid: u64,
    pub uname: String,
}

/// 处理B站API响应的通用逻辑
async fn handle_api_response<T, D>(response: Response, error_msg: &str) -> Result<D> 
where 
    T: for<'de> Deserialize<'de>,
    T: std::fmt::Debug,
    T: ApiResponse<Data = D>
{
    check_response(&response).await?;
    
    let res_data: T = response.json().await?;
    
    match res_data.get_code() {
        0 => {
            if let Some(data) = res_data.get_data() {
                Ok(data)
            } else {
                Err(BiliError::ApiError(res_data.get_code(), error_msg.to_string()).into())
            }
        },
        _ => Err(BiliError::ApiError(res_data.get_code(), res_data.get_message()).into()),
    }
}

/// API响应公共特性，用于处理不同响应结构体的共同特性
trait ApiResponse {
    type Data;
    
    fn get_code(&self) -> i32;
    fn get_message(&self) -> String;
    fn get_data(&self) -> Option<Self::Data>;
}

impl ApiResponse for QrCodeGenerateResponse {
    type Data = QrCodeGenerateData;
    
    fn get_code(&self) -> i32 {
        self.code
    }
    
    fn get_message(&self) -> String {
        self.message.clone()
    }
    
    fn get_data(&self) -> Option<Self::Data> {
        self.data.clone()
    }
}

impl ApiResponse for QrCodePollResponse {
    type Data = QrCodePollData;
    
    fn get_code(&self) -> i32 {
        self.code
    }
    
    fn get_message(&self) -> String {
        self.message.clone()
    }
    
    fn get_data(&self) -> Option<Self::Data> {
        self.data.clone()
    }
}

impl ApiResponse for UserInfoResponse {
    type Data = UserInfoData;
    
    fn get_code(&self) -> i32 {
        self.code
    }
    
    fn get_message(&self) -> String {
        self.message.clone()
    }
    
    fn get_data(&self) -> Option<Self::Data> {
        self.data.clone()
    }
}

// 添加Clone特性用于ApiResponse特性实现
impl Clone for QrCodeGenerateData {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            qrcode_key: self.qrcode_key.clone(),
        }
    }
}

impl Clone for QrCodePollData {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            refresh_token: self.refresh_token.clone(),
            timestamp: self.timestamp,
            code: self.code,
            message: self.message.clone(),
        }
    }
}

impl Clone for UserInfoData {
    fn clone(&self) -> Self {
        Self {
            is_login: self.is_login,
            mid: self.mid,
            uname: self.uname.clone(),
        }
    }
}

/// 生成登录二维码
pub async fn generate_qrcode(client: &Client) -> Result<QrCodeGenerateData> {
    let timestamp = get_timestamp();
    let url = format!("{}?source=main-fe-header&t={}", QR_CODE_GENERATE_URL, timestamp);
    
    let response = client.get(url).send().await?;
    handle_api_response::<QrCodeGenerateResponse, _>(response, "无返回数据").await
}

/// 轮询二维码状态
pub async fn poll_qrcode(client: &Client, qrcode_key: &str) -> Result<QrCodePollData> {
    let timestamp = get_timestamp();
    let url = format!("{}?qrcode_key={}&t={}", QR_CODE_POLL_URL, qrcode_key, timestamp);
    
    let response = client.get(url).send().await?;
    handle_api_response::<QrCodePollResponse, _>(response, "无返回数据").await
}

/// 获取用户信息
pub async fn get_user_info(client: &Client) -> Result<UserInfoData> {
    let response = client.get(USER_INFO_URL).send().await?;
    handle_api_response::<UserInfoResponse, _>(response, "无返回数据").await
}

/// 检查响应状态
async fn check_response(response: &Response) -> Result<()> {
    if response.status().is_success() {
        Ok(())
    } else {
        Err(BiliError::ApiError(
            response.status().as_u16() as i32,
            format!("请求失败: {}", response.status())
        ).into())
    }
} 