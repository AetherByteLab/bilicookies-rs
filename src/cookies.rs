use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, SET_COOKIE};
use serde::{Deserialize, Serialize};

use crate::{auth::LoginResult, error::BiliError};

/// 存储Cookie信息的结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieItem {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<DateTime<Utc>>,
    pub http_only: bool,
    pub secure: bool,
}

/// 从登录结果中提取cookies
pub async fn extract_cookies(login_result: &LoginResult) -> Result<Vec<CookieItem>> {
    // 先尝试从用户信息接口获取cookies
    let response = login_result.client.get("https://api.bilibili.com/x/web-interface/nav")
        .send()
        .await?;
    
    // 尝试从响应体中提取用户信息和可能的登录状态
    let body_text = response.text().await?;
    
    // 尝试解析JSON响应
    let mut cookies = Vec::new();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_text) {
        // 从响应体中提取用户ID和登录状态
        if let Some(data) = json.get("data") {
            // 提取用户ID
            if let Some(mid) = data.get("mid").and_then(|v| v.as_u64()) {
                if mid > 0 {
                    cookies.push(CookieItem {
                        name: "DedeUserID".to_string(),
                        value: mid.to_string(),
                        domain: "bilibili.com".to_string(),
                        path: "/".to_string(),
                        expires: None,
                        http_only: false,
                        secure: false,
                    });
                }
            }
            
            // 如果有csrf令牌
            if let Some(csrf) = data.get("csrf").and_then(|v| v.as_str()) {
                cookies.push(CookieItem {
                    name: "bili_jct".to_string(),
                    value: csrf.to_string(),
                    domain: "bilibili.com".to_string(),
                    path: "/".to_string(),
                    expires: None,
                    http_only: false,
                    secure: false,
                });
            }
        }
    }
    
    // 尝试调用其他API
    // 1. 通过passport接口获取完整cookies
    let passport_response = login_result.client.get("https://passport.bilibili.com/x/passport-login/web/cookie/info")
        .send()
        .await?;
    
    if let Ok(more_cookies) = parse_cookies(passport_response.headers()) {
        for cookie in more_cookies {
            if !cookies.iter().any(|c| c.name == cookie.name) {
                cookies.push(cookie);
            }
        }
    }
    
    // 2. 通过个人中心接口
    let space_response = login_result.client.get("https://space.bilibili.com")
        .send()
        .await?;
    
    if let Ok(more_cookies) = parse_cookies(space_response.headers()) {
        for cookie in more_cookies {
            if !cookies.iter().any(|c| c.name == cookie.name) {
                cookies.push(cookie);
            }
        }
    }
    
    // 手动从login_result中提取特定信息生成cookie
    if cookies.iter().find(|c| c.name == "DedeUserID").is_none() && login_result.uid != 0 {
        cookies.push(CookieItem {
            name: "DedeUserID".to_string(),
            value: login_result.uid.to_string(),
            domain: "bilibili.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
        });
    }
    
    if cookies.iter().find(|c| c.name == "SESSDATA").is_none() && !login_result.refresh_token.is_empty() {
        // SESSDATA通常和refresh_token关联
        cookies.push(CookieItem {
            name: "SESSDATA".to_string(),
            value: login_result.refresh_token.clone(),
            domain: "bilibili.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: true,
            secure: true,
        });
    }
    
    if cookies.is_empty() {
        return Err(BiliError::CookieError("未找到B站相关的Cookie".to_string()).into());
    }
    
    Ok(cookies)
}

/// 从响应头中解析cookies
fn parse_cookies(headers: &HeaderMap) -> Result<Vec<CookieItem>> {
    let mut cookies = Vec::new();
    
    // 获取所有的Set-Cookie头，而不仅仅是第一个
    for cookie_header in headers.get_all(SET_COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            // 解析每个cookie
            let cookie_parts: Vec<&str> = cookie_str.split(';').collect();
            if cookie_parts.is_empty() {
                continue; // 跳过无效的cookie
            }
            
            // 解析name=value部分
            let name_value: Vec<&str> = cookie_parts[0].split('=').collect();
            if name_value.len() < 2 {
                continue; // 跳过无效的cookie
            }
            
            let name = name_value[0].trim().to_string();
            // 将剩余部分作为值（可能包含等号）
            let value = name_value[1..].join("=").trim().to_string();
            
            // 解析其他属性
            let mut domain = "bilibili.com".to_string();
            let mut path = "/".to_string();
            let expires = None;
            let mut http_only = false;
            let mut secure = false;
            
            for part in &cookie_parts[1..] {
                let attr_parts: Vec<&str> = part.split('=').collect();
                if attr_parts.is_empty() {
                    continue;
                }
                let attr_name = attr_parts[0].trim().to_lowercase();
                
                match attr_name.as_str() {
                    "domain" => {
                        if attr_parts.len() > 1 {
                            domain = attr_parts[1].trim().to_string();
                        }
                    },
                    "path" => {
                        if attr_parts.len() > 1 {
                            path = attr_parts[1].trim().to_string();
                        }
                    },
                    "expires" => {
                        // 简化处理，不解析过期时间
                    },
                    "httponly" => http_only = true,
                    "secure" => secure = true,
                    _ => {}
                }
            }
            
            // 只添加B站相关的Cookie或重要的Cookie
            if domain.contains("bilibili") || domain.contains("bili") || 
               name.eq_ignore_ascii_case("SESSDATA") || name.eq_ignore_ascii_case("bili_jct") || 
               name.eq_ignore_ascii_case("DedeUserID") || name.contains("bili") {
                cookies.push(CookieItem {
                    name,
                    value,
                    domain,
                    path,
                    expires,
                    http_only,
                    secure,
                });
            }
        }
    }
    
    // 如果仍然找不到cookies，则从Cookie头中尝试提取
    if cookies.is_empty() && headers.contains_key("Cookie") {
        if let Some(cookie_header) = headers.get("Cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie_part in cookie_str.split(';') {
                    let parts: Vec<&str> = cookie_part.split('=').collect();
                    if parts.len() >= 2 {
                        let name = parts[0].trim().to_string();
                        let value = parts[1..].join("=").trim().to_string();
                        
                        cookies.push(CookieItem {
                            name,
                            value,
                            domain: "bilibili.com".to_string(),
                            path: "/".to_string(),
                            expires: None,
                            http_only: false,
                            secure: false,
                        });
                    }
                }
            }
        }
    }
    
    // 对于解析来的结果，我们不需要检查是否为空，让调用者决定如何处理
    Ok(cookies)
}

/// 获取重要的Cookie
#[allow(dead_code)]
pub fn get_important_cookies(cookies: &[CookieItem]) -> Vec<CookieItem> {
    let important_names = ["SESSDATA", "bili_jct", "DedeUserID", "DedeUserID__ckMd5", "sid"];
    
    cookies.iter()
        .filter(|c| important_names.contains(&c.name.as_str()))
        .cloned()
        .collect()
}

/// 将CookieItem转换为Cookie字符串
#[allow(dead_code)]
pub fn cookie_to_string(cookie: &CookieItem) -> String {
    format!("{}={}", cookie.name, cookie.value)
}

/// 将多个CookieItem合并为单个Cookie字符串
#[allow(dead_code)]
pub fn cookies_to_header_string(cookies: &[CookieItem]) -> String {
    cookies.iter()
        .map(cookie_to_string)
        .collect::<Vec<String>>()
        .join("; ")
}

/// 简化的cookie对象 (键值对形式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleCookie {
    pub key: String,
    pub value: String,
}

/// 转换为简化的cookie列表
pub fn to_simple_cookies(cookies: &[CookieItem]) -> Vec<SimpleCookie> {
    cookies.iter().map(|c| {
        SimpleCookie {
            key: c.name.clone(),
            value: c.value.clone(),
        }
    }).collect()
} 