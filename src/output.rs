use anyhow::Result;
use std::fs;
use std::path::Path;
use csv::Writer;

use crate::cookies::{CookieItem, to_simple_cookies};

/// 以JSON格式输出Cookies
pub fn format_as_json(cookies: &[CookieItem]) -> Result<String> {
    Ok(serde_json::to_string_pretty(cookies)?)
}

/// 以Netscape格式输出Cookies
pub fn format_as_netscape(cookies: &[CookieItem]) -> Result<String> {
    let mut output = String::from("# Netscape HTTP Cookie File\n# https://curl.se/docs/http-cookies.html\n");
    
    for cookie in cookies {
        // 格式: domain flag path secure expiry name value
        let secure = if cookie.secure { "TRUE" } else { "FALSE" };
        // http_only在Netscape格式中不使用，但在其他地方可能会用到
        let _http_only = if cookie.http_only { "TRUE" } else { "FALSE" };
        let expiry = match cookie.expires {
            Some(time) => time.timestamp().to_string(),
            None => "0".to_string(),
        };
        let domain = cookie.domain.clone();
        // Netscape格式要求域名开头有一个点
        let domain = if domain.starts_with('.') { domain } else { format!(".{}", domain) };
        
        let line = format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            domain,
            "FALSE", // host_only flag
            cookie.path,
            secure,
            expiry,
            cookie.name,
            cookie.value
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
        output.push_str(&format!("{}={}\n", cookie.key, cookie.value));
    }
    
    Ok(output)
}

/// 以TOML格式输出Cookies
pub fn format_as_toml(cookies: &[CookieItem]) -> Result<String> {
    let simple_cookies = to_simple_cookies(cookies);
    
    Ok(toml::to_string_pretty(&simple_cookies)?)
}

/// 以CSV格式输出Cookies
pub fn format_as_csv(cookies: &[CookieItem]) -> Result<String> {
    let mut writer = Writer::from_writer(vec![]);
    
    for cookie in cookies {
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