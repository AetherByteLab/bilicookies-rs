use anyhow::Result;
use colored::Colorize;
use qrcode::QrCode;
use reqwest::{Client, cookie::Jar};
use std::{sync::Arc, time::Duration};
use tokio::time;

use crate::{api, error::BiliError};

// 二维码轮询间隔
const POLL_INTERVAL_MS: u64 = 1000;
// 二维码过期时间（秒）
const QR_CODE_EXPIRE_SECONDS: u64 = 180;

// 二维码状态码
const QR_CODE_STATUS_SCANNED: i32 = 86038; // 已扫码
const QR_CODE_STATUS_CONFIRMED: i32 = 0;   // 已确认登录
const QR_CODE_STATUS_EXPIRED: i32 = 86039; // 已过期

/// 登录响应
#[derive(Debug)]
pub struct LoginResult {
    pub client: Client,
    #[allow(dead_code)]
    pub cookie_jar: Arc<Jar>,
    #[allow(dead_code)]
    pub refresh_token: String,
    #[allow(dead_code)]
    pub uid: u64,
    #[allow(dead_code)]
    pub username: String,
}

/// 使用二维码登录B站账号
pub async fn login_with_qrcode() -> Result<LoginResult> {
    // 创建客户端
    let (client, cookie_jar) = api::create_client()?;
    
    // 获取二维码
    let qr_data = api::generate_qrcode(&client).await?;
    
    // 显示二维码
    display_qrcode(&qr_data.url)?;
    
    println!("{}", "请使用B站手机APP扫描上方二维码并确认登录...".yellow());
    
    // 轮询二维码状态
    let mut interval = time::interval(Duration::from_millis(POLL_INTERVAL_MS));
    let start = std::time::Instant::now();
    
    loop {
        interval.tick().await;
        
        // 检查是否超时
        if start.elapsed().as_secs() > QR_CODE_EXPIRE_SECONDS {
            return Err(BiliError::LoginError("二维码已过期，请重新运行程序".to_string()).into());
        }
        
        let poll_result = api::poll_qrcode(&client, &qr_data.qrcode_key).await;
        
        match poll_result {
            Ok(data) => {
                match data.code {
                    QR_CODE_STATUS_SCANNED => {
                        println!("{}", "已扫描二维码，请在手机上确认登录...".yellow());
                    },
                    QR_CODE_STATUS_CONFIRMED => {
                        println!("{}", "扫码成功，正在获取用户信息...".green());
                        
                        // 获取用户信息
                        let user_info = api::get_user_info(&client).await?;
                        
                        if user_info.is_login {
                            return Ok(LoginResult {
                                client,
                                cookie_jar,
                                refresh_token: data.refresh_token,
                                uid: user_info.mid,
                                username: user_info.uname,
                            });
                        } else {
                            return Err(BiliError::LoginError("登录状态校验失败".to_string()).into());
                        }
                    },
                    QR_CODE_STATUS_EXPIRED => {
                        return Err(BiliError::LoginError("二维码已过期，请重新运行程序".to_string()).into());
                    },
                    _ => {
                        // 继续轮询
                    }
                }
            },
            Err(e) => {
                // 轮询出错，继续尝试
                eprintln!("轮询出错: {}, 将继续尝试...", e);
            },
        }
    }
}

/// 在终端显示二维码
fn display_qrcode(url: &str) -> Result<()> {
    // 创建二维码
    let qr = QrCode::new(url.as_bytes())?;
    
    // 转为字符串显示
    let qr_string = qr.render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    
    println!("{}", qr_string);
    
    Ok(())
} 