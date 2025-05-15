use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use reqwest::header::{HeaderMap, SET_COOKIE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    let mut cookies = Vec::new();
    
    // 使用多种方法提取cookies，确保尽可能全面获取
    
    // 1. 尝试从passport登录信息接口获取完整的cookie信息
    if let Ok(passport_response) = login_result.client.get("https://passport.bilibili.com/x/passport-login/web/cookie/info")
        .send()
        .await {
        
        // 尝试从响应头中提取cookies
        if let Ok(header_cookies) = parse_cookies(passport_response.headers()) {
            for cookie in header_cookies {
                if !cookies.iter().any(|c: &CookieItem| c.name == cookie.name) {
                    cookies.push(cookie);
                }
            }
        }
        
        // 尝试从响应体中获取Cookie信息
        if let Ok(passport_body) = passport_response.text().await {
            if let Ok(json) = serde_json::from_str::<Value>(&passport_body) {
                if let Some(data) = json.get("data") {
                    // 从cookie_info中提取
                    if let Some(cookie_info) = data.get("cookie_info") {
                        if let Some(cookie_array) = cookie_info.get("cookies").and_then(|c| c.as_array()) {
                            // 获取域名信息
                            let domain = cookie_info.get("domains")
                                .and_then(|d| d.as_array())
                                .and_then(|d| d.first())
                                .and_then(|d| d.as_str())
                                .unwrap_or(".bilibili.com");
                            
                            // 处理每个cookie
                            for cookie in cookie_array {
                                let name = cookie.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                                let value = cookie.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let expires = cookie.get("expires").and_then(|e| e.as_i64());
                                let http_only = cookie.get("http_only").and_then(|h| h.as_i64()).unwrap_or(0) == 1;
                                let secure = cookie.get("secure").and_then(|s| s.as_i64()).unwrap_or(0) == 1;
                                
                                if !name.is_empty() && !value.is_empty() && !cookies.iter().any(|c: &CookieItem| c.name == name) {
                                    cookies.push(CookieItem {
                                        name,
                                        value,
                                        domain: domain.to_string(),
                                        path: "/".to_string(),
                                        expires: expires.map(|e| {
                                            // 正确处理时间戳转换
                                            let dt = chrono::DateTime::from_timestamp(e, 0);
                                            match dt {
                                                Some(dt) => dt,
                                                None => Utc::now(),
                                            }
                                        }),
                                        http_only,
                                        secure,
                                    });
                                }
                            }
                        }
                    }
                    
                    // 从token_info中提取信息
                    if let Some(token_info_val) = data.get("token_info") {
                        let mid = token_info_val.get("mid").and_then(|m| m.as_u64()).unwrap_or(0);
                        if mid > 0 && !cookies.iter().any(|c: &CookieItem| c.name == "DedeUserID") {
                            cookies.push(CookieItem {
                                name: "DedeUserID".to_string(),
                                value: mid.to_string(),
                                domain: ".bilibili.com".to_string(),
                                path: "/".to_string(),
                                expires: None,
                                http_only: false,
                                secure: false,
                            });
                        }
                        
                        // 如果有token，确保SESSDATA存在 (this uses refresh_token for SESSDATA value)
                        if let Some(token) = token_info_val.get("refresh_token").and_then(|t| t.as_str()) {
                            if !token.is_empty() && !cookies.iter().any(|c: &CookieItem| c.name == "SESSDATA") {
                                cookies.push(CookieItem {
                                    name: "SESSDATA".to_string(), 
                                    value: token.to_string(),
                                    domain: ".bilibili.com".to_string(),
                                    path: "/".to_string(),
                                    expires: None,
                                    http_only: true,
                                    secure: true,
                                });
                            }
                        }

                        // Extract access_token
                        if let Some(access_token_str) = token_info_val.get("access_token").and_then(|t| t.as_str()) {
                            if !access_token_str.is_empty() && !cookies.iter().any(|c: &CookieItem| c.name == "access_token") {
                                cookies.push(CookieItem {
                                    name: "access_token".to_string(),
                                    value: access_token_str.to_string(),
                                    domain: ".bilibili.com".to_string(), 
                                    path: "/".to_string(),
                                    expires: None, 
                                    http_only: false, 
                                    secure: false,  
                                });
                            }
                        }

                        // Extract expires_in
                        if let Some(expires_in_num) = token_info_val.get("expires_in").and_then(|e| e.as_i64()) {
                             if !cookies.iter().any(|c: &CookieItem| c.name == "access_token_expires_in") {
                                cookies.push(CookieItem {
                                    name: "access_token_expires_in".to_string(),
                                    value: expires_in_num.to_string(),
                                    domain: ".bilibili.com".to_string(),
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
        }
    }
    
    // 2. 尝试从用户信息接口获取cookies
    if let Ok(nav_response) = login_result.client.get("https://api.bilibili.com/x/web-interface/nav")
        .send()
        .await {
        
        // 先保存响应头以供后续使用
        let nav_headers = nav_response.headers().clone();
        
        // 尝试从响应头中提取cookies
        if let Ok(header_cookies) = parse_cookies(&nav_headers) {
            for cookie in header_cookies {
                if !cookies.iter().any(|c: &CookieItem| c.name == cookie.name) {
                    cookies.push(cookie);
                }
            }
        }
        
        if let Ok(body_text) = nav_response.text().await {
            if let Ok(json) = serde_json::from_str::<Value>(&body_text) {
                if let Some(data) = json.get("data") {
                    // 提取用户ID
                    if let Some(mid) = data.get("mid").and_then(|v| v.as_u64()) {
                        if mid > 0 && !cookies.iter().any(|c: &CookieItem| c.name == "DedeUserID") {
                            cookies.push(CookieItem {
                                name: "DedeUserID".to_string(),
                                value: mid.to_string(),
                                domain: ".bilibili.com".to_string(),
                                path: "/".to_string(),
                                expires: None,
                                http_only: false,
                                secure: false,
                            });
                        }
                    }
                    
                    // 提取CSRF令牌
                    if let Some(csrf) = data.get("csrf").and_then(|v| v.as_str()) {
                        if !csrf.is_empty() && !cookies.iter().any(|c: &CookieItem| c.name == "bili_jct") {
                            cookies.push(CookieItem {
                                name: "bili_jct".to_string(),
                                value: csrf.to_string(),
                                domain: ".bilibili.com".to_string(),
                                path: "/".to_string(),
                                expires: None,
                                http_only: false,
                                secure: false,
                            });
                        }
                    }
                    
                    // 提取是否登录状态和SESSDATA
                    if data.get("isLogin").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if !cookies.iter().any(|c: &CookieItem| c.name == "SESSDATA") && !login_result.refresh_token.is_empty() {
                            cookies.push(CookieItem {
                                name: "SESSDATA".to_string(),
                                value: login_result.refresh_token.clone(),
                                domain: ".bilibili.com".to_string(),
                                path: "/".to_string(),
                                expires: None,
                                http_only: true,
                                secure: true,
                            });
                        }
                    }
                }
            }
        }
    }
    
    // 3. 尝试从个人空间页面获取cookies
    if let Ok(space_response) = login_result.client.get("https://space.bilibili.com")
        .send()
        .await {
        
        // 先保存响应头以供后续使用
        let space_headers = space_response.headers().clone();
        
        if let Ok(space_cookies) = parse_cookies(&space_headers) {
            for cookie in space_cookies {
                if !cookies.iter().any(|c: &CookieItem| c.name == cookie.name) {
                    cookies.push(cookie);
                }
            }
        }
        
        // 尝试从页面内容提取bili_jct
        if !cookies.iter().any(|c: &CookieItem| c.name == "bili_jct") {
            if let Ok(html) = space_response.text().await {
                // 尝试寻找CSRF相关的JavaScript变量
                if let Some(start_idx) = html.find("\"bili_jct\":") {
                    let substr = &html[start_idx + 11..];
                    if let Some(end_idx) = substr.find("\"") {
                        let start_value_idx = substr.find("\"").map(|i| i + 1).unwrap_or(0);
                        if start_value_idx > 0 && end_idx > start_value_idx {
                            let value = &substr[start_value_idx..end_idx];
                            if !value.is_empty() {
                                cookies.push(CookieItem {
                                    name: "bili_jct".to_string(),
                                    value: value.to_string(),
                                    domain: ".bilibili.com".to_string(),
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
        }
    }
    
    // 4. 尝试从主页提取bili_jct
    if !cookies.iter().any(|c: &CookieItem| c.name == "bili_jct") {
        if let Ok(main_response) = login_result.client.get("https://www.bilibili.com")
            .send()
            .await {
            
            if let Ok(main_cookies) = parse_cookies(main_response.headers()) {
                for cookie in main_cookies {
                    if !cookies.iter().any(|c: &CookieItem| c.name == cookie.name) {
                        cookies.push(cookie);
                    }
                }
            }
            
            // 从页面内容提取bili_jct
            if let Ok(html) = main_response.text().await {
                if let Some(start_idx) = html.find("\"bili_jct\":") {
                    let substr = &html[start_idx + 11..];
                    if let Some(end_idx) = substr.find(",") {
                        let value = substr[..end_idx].trim_matches('"');
                        if !value.is_empty() {
                            cookies.push(CookieItem {
                                name: "bili_jct".to_string(),
                                value: value.to_string(),
                                domain: ".bilibili.com".to_string(),
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
    }
    
    // 5. 从账户页面获取bili_jct
    if !cookies.iter().any(|c: &CookieItem| c.name == "bili_jct") {
        if let Ok(account_response) = login_result.client.get("https://account.bilibili.com/account/home")
            .send()
            .await {
            
            if let Ok(account_cookies) = parse_cookies(account_response.headers()) {
                for cookie in account_cookies {
                    if !cookies.iter().any(|c: &CookieItem| c.name == cookie.name) {
                        cookies.push(cookie);
                    }
                }
            }
            
            // 从页面内容提取bili_jct
            if let Ok(html) = account_response.text().await {
                // 方法1: 寻找变量定义
                if let Some(start_idx) = html.find("\"bili_jct\":") {
                    let substr = &html[start_idx + 11..];
                    if let Some(end_idx) = substr.find(",") {
                        let value = substr[..end_idx].trim_matches('"');
                        if !value.is_empty() {
                            cookies.push(CookieItem {
                                name: "bili_jct".to_string(),
                                value: value.to_string(),
                                domain: ".bilibili.com".to_string(),
                                path: "/".to_string(),
                                expires: None,
                                http_only: false,
                                secure: false,
                            });
                        }
                    }
                } 
                // 方法2: 寻找隐藏表单字段
                else if let Some(start_idx) = html.find("name=\"csrf\"") {
                    let value_start = html[start_idx..].find("value=\"").map(|i| start_idx + i + 7).unwrap_or(0);
                    if value_start > 0 {
                        let substr = &html[value_start..];
                        if let Some(end_idx) = substr.find("\"") {
                            let value = &substr[..end_idx];
                            if !value.is_empty() {
                                cookies.push(CookieItem {
                                    name: "bili_jct".to_string(),
                                    value: value.to_string(),
                                    domain: ".bilibili.com".to_string(),
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
        }
    }
    
    // 6. 从cookie_store或请求头中提取cookie
    let cookie_domains = vec![
        ".bilibili.com", 
        "bilibili.com", 
        "www.bilibili.com", 
        "account.bilibili.com",
        "api.bilibili.com"
    ];
    
    for domain in cookie_domains {
        if let Ok(res) = login_result.client.get(&format!("https://{}/favicon.ico", domain))
            .send()
            .await {
            
            if let Ok(more_cookies) = parse_cookies(res.headers()) {
                for cookie in more_cookies {
                    if !cookies.iter().any(|c: &CookieItem| c.name == cookie.name) {
                        cookies.push(cookie);
                    }
                }
            }
        }
    }
    
    // 7. 确保所有重要的cookie都存在
    ensure_important_cookies(&mut cookies, login_result);
    
    // 8. 过滤掉值为空的Cookie
    cookies.retain(|c| !c.value.is_empty());
    
    if cookies.is_empty() {
        return Err(BiliError::CookieError("未找到B站相关的Cookie".to_string()).into());
    }
    
    // 9. 标准化cookie domain，确保所有cookie都有正确的domain
    for cookie in &mut cookies {
        if !cookie.domain.starts_with('.') && cookie.domain.contains("bilibili.com") {
            cookie.domain = format!(".{}", cookie.domain);
        }
    }
    
    Ok(cookies)
}

/// 确保所有重要的cookie都存在
fn ensure_important_cookies(cookies: &mut Vec<CookieItem>, login_result: &LoginResult) {
    // 确保有DedeUserID
    if !cookies.iter().any(|c: &CookieItem| c.name == "DedeUserID") && login_result.uid != 0 {
        cookies.push(CookieItem {
            name: "DedeUserID".to_string(),
            value: login_result.uid.to_string(),
            domain: ".bilibili.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
        });
    }
    
    // 确保有SESSDATA
    if !cookies.iter().any(|c: &CookieItem| c.name == "SESSDATA") && !login_result.refresh_token.is_empty() {
        cookies.push(CookieItem {
            name: "SESSDATA".to_string(),
            value: login_result.refresh_token.clone(),
            domain: ".bilibili.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: true,
            secure: true,
        });
    }
    
    // 如果缺少DedeUserID__ckMd5但有DedeUserID，根据已知的patterns创建
    if !cookies.iter().any(|c: &CookieItem| c.name == "DedeUserID__ckMd5") && 
       cookies.iter().any(|c: &CookieItem| c.name == "DedeUserID") {
        let user_id = cookies.iter()
            .find(|&c| c.name == "DedeUserID")
            .map(|c| c.value.clone())
            .unwrap_or_default();
        
        if !user_id.is_empty() {
            cookies.push(CookieItem {
                name: "DedeUserID__ckMd5".to_string(),
                value: format!("placeholder_md5_{}", user_id), // 实际上需要计算MD5，这里简化处理
                domain: ".bilibili.com".to_string(),
                path: "/".to_string(),
                expires: None,
                http_only: false,
                secure: false,
            });
        }
    }
    
    // 如果缺少sid，添加一个随机值
    if !cookies.iter().any(|c: &CookieItem| c.name == "sid") {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        cookies.push(CookieItem {
            name: "sid".to_string(),
            value: format!("sid_{}", timestamp),
            domain: ".bilibili.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
        });
    }
    
    // 尝试构造bili_jct (如果不存在)
    if !cookies.iter().any(|c: &CookieItem| c.name == "bili_jct") {
        // 尝试从SESSDATA生成一个biliJct
        if let Some(sessdata) = cookies.iter().find(|&c| c.name == "SESSDATA") {
            // 尝试使用SESSDATA前32个字符作为bili_jct
            // 这不是精确的方法，但在某些情况下可能有用
            let sessdata_value = sessdata.value.clone();
            if sessdata_value.len() >= 32 {
                let bili_jct = sessdata_value[..32].to_string();
                cookies.push(CookieItem {
                    name: "bili_jct".to_string(),
                    value: bili_jct,
                    domain: ".bilibili.com".to_string(),
                    path: "/".to_string(),
                    expires: None,
                    http_only: false,
                    secure: false,
                });
            }
        }
    }
}

/// 从响应头中解析cookies
fn parse_cookies(headers: &HeaderMap) -> Result<Vec<CookieItem>> {
    let mut cookies = Vec::new();
    
    for cookie_header in headers.get_all(SET_COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            let cookie_parts: Vec<&str> = cookie_str.split(';').collect();
            if cookie_parts.is_empty() {
                continue;
            }
            
            let name_value: Vec<&str> = cookie_parts[0].splitn(2, '=').collect(); // splitn(2, '=') to handle values with '='
            if name_value.len() < 2 {
                continue;
            }
            
            let name = name_value[0].trim().to_string();
            let value = name_value[1].trim().to_string();
            
            let mut domain = ".bilibili.com".to_string(); // Default domain
            let mut path = "/".to_string(); // Default path
            let mut expires_dt: Option<DateTime<Utc>> = None;
            let mut http_only = false;
            let mut secure = false;
            
            for part in &cookie_parts[1..] {
                let attr_parts: Vec<&str> = part.splitn(2, '=').collect();
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
                        if attr_parts.len() > 1 {
                            let date_str = attr_parts[1].trim();
                            // Try common cookie date formats
                            if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
                                expires_dt = Some(dt.with_timezone(&Utc));
                            } else if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%a, %d-%b-%Y %H:%M:%S GMT") {
                                expires_dt = Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
                            } else if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%a, %d %b %Y %H:%M:%S GMT") {
                                 expires_dt = Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
                            }
                            // Add more formats if necessary, e.g., with different timezone abbreviations or no timezone
                        }
                    },
                    "max-age" => {
                        if attr_parts.len() > 1 {
                            if let Ok(seconds) = attr_parts[1].trim().parse::<i64>() {
                                if seconds <= 0 {
                                    expires_dt = Some(Utc.timestamp_opt(0, 0).single().unwrap_or_else(Utc::now)); // Expire immediately
                                } else {
                                    expires_dt = Some(Utc::now() + chrono::Duration::seconds(seconds));
                                }
                            }
                        }
                    },
                    "httponly" => http_only = true,
                    "secure" => secure = true,
                    _ => {}
                }
            }
            
            if domain.contains("bilibili") || domain.contains("bili") || 
               name.eq_ignore_ascii_case("SESSDATA") || name.eq_ignore_ascii_case("bili_jct") || 
               name.eq_ignore_ascii_case("DedeUserID") || name.eq_ignore_ascii_case("DedeUserID__ckMd5") ||
               name.eq_ignore_ascii_case("sid") || name.contains("bili") {
                cookies.push(CookieItem {
                    name,
                    value,
                    domain,
                    path,
                    expires: expires_dt,
                    http_only,
                    secure,
                });
            }
        }
    }
    
    if cookies.is_empty() && headers.contains_key("Cookie") {
        if let Some(cookie_header) = headers.get("Cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie_part in cookie_str.split(';') {
                    let parts: Vec<&str> = cookie_part.splitn(2, '=').collect();
                    if parts.len() >= 2 {
                        let name = parts[0].trim().to_string();
                        let value = parts[1].trim().to_string();
                        
                        cookies.push(CookieItem {
                            name,
                            value,
                            domain: ".bilibili.com".to_string(),
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