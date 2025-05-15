use anyhow::Result;
use std::fs;
use std::path::Path;
use csv::Writer;
use serde::Serialize;

use crate::cookies::{CookieItem, to_simple_cookies, SimpleCookie};

/// 以JSON格式输出Cookies
pub fn format_as_json(cookies: &[CookieItem]) -> Result<String> {
    Ok(serde_json::to_string_pretty(cookies)?)
}

/// 以Netscape格式输出Cookies
pub fn format_as_netscape(cookies: &[CookieItem]) -> Result<String> {
    let mut output = String::from("# Netscape HTTP Cookie File\n# https://curl.se/docs/http-cookies.html\n");
    
    for cookie in cookies {
        // 只检查cookie name是否为空。允许value为空。
        if cookie.name.is_empty() {
            continue; // 跳过没有名称的cookie项
        }

        let secure_str = if cookie.secure { "TRUE" } else { "FALSE" };
        let expiry_str = match cookie.expires {
            Some(time) => time.timestamp().to_string(),
            None => "0".to_string(), 
        };
        
        let domain_str = if cookie.domain.starts_with('.') || cookie.domain.matches('.').count() < 1 {
            cookie.domain.clone()
        } else {
            format!(".{}", cookie.domain)
        };
        
        let line = format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            domain_str, 
            "FALSE",    
            cookie.path,      
            secure_str,       
            expiry_str,       
            cookie.name,      
            cookie.value      // value 可能为空字符串
        );
        
        output.push_str(&line);
    }
    
    Ok(output)
}

/// 以键值对格式输出Cookies
pub fn format_as_key_value(cookies: &[CookieItem]) -> Result<String> {
    let simple_cookies = to_simple_cookies(cookies);
    let mut output = String::new();
    
    for cookie in simple_cookies {
        if !cookie.key.is_empty() { // 确保key不为空
            output.push_str(&format!("{}={}\n", cookie.key, cookie.value));
        }
    }
    
    Ok(output)
}

/// 为TOML序列化定义的包装结构体
#[derive(Serialize)]
struct TomlCookieListInternalWrapper<'a> {
    // 使用一个不容易与用户toml键冲突的名称
    bilicookies_rs_export: &'a [SimpleCookie],
}

/// 以TOML格式输出Cookies
pub fn format_as_toml(cookies: &[CookieItem]) -> Result<String> {
    let simple_cookies = to_simple_cookies(cookies)
        .into_iter()
        // 只过滤key为空的情况。允许value为空。
        .filter(|sc| !sc.key.is_empty()) 
        .collect::<Vec<SimpleCookie>>();

    if simple_cookies.is_empty() {
        return Ok("# No valid cookies (with non-empty names) to export in TOML format.\n".to_string());
    }

    let wrapper = TomlCookieListInternalWrapper { bilicookies_rs_export: &simple_cookies };
    
    match toml::to_string_pretty(&wrapper) {
        Ok(s) => Ok(s),
        Err(e) => {
            Err(anyhow::Error::new(e).context("Failed to serialize cookies to TOML"))
        }
    }
}

/// 以CSV格式输出Cookies
pub fn format_as_csv(cookies: &[CookieItem]) -> Result<String> {
    let mut writer = Writer::from_writer(vec![]);
    
    // 写入CSV头部
    writer.write_record(&["name", "value", "domain", "path", "expires_rfc3339", "http_only", "secure"])?;

    for cookie in cookies {
        if cookie.name.is_empty() { // 跳过没有名称的cookie
            continue;
        }
        let expires = match cookie.expires {
            Some(time) => time.to_rfc3339(),
            None => String::new(),
        };
        
        writer.write_record(&[
            &cookie.name,
            &cookie.value,
            &cookie.domain,
            &cookie.path,
            &expires,
            &cookie.http_only.to_string(),
            &cookie.secure.to_string(),
        ])?;
    }
    
    let csv_bytes = writer.into_inner()?;
    Ok(String::from_utf8(csv_bytes)?)
}

/// 保存内容到文件
pub fn save_to_file(content: &str, path: &str) -> Result<()> {
    // 确保父目录存在
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    
    fs::write(path, content)?;
    Ok(())
} 