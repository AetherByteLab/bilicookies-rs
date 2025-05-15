use anyhow::Result;
use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::fs;

mod api;
mod auth;
mod config;
mod cookies;
mod error;
mod output;

#[derive(Parser, Debug)]
#[command(author, version, about = "B站扫码登录获取cookies工具")]
struct Cli {
    /// 输出格式
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Netscape)]
    format: OutputFormat,

    /// 保存到文件
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    /// JSON格式
    Json,
    /// Netscape cookies.txt格式
    Netscape,
    /// 键值对格式
    KeyValue,
    /// TOML格式
    Toml,
    /// CSV格式
    Csv,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    println!("{}", "欢迎使用B站扫码登录工具!".green().bold());
    println!("即将生成二维码，请使用B站手机客户端扫描以登录...");
    
    let login_result = auth::login_with_qrcode().await?;
    let cookies = cookies::extract_cookies(&login_result).await?;
    
    // ---- 临时调试代码 开始 ----
     println!("\nDEBUG: 全部提取到的Cookies:");
     for cookie in &cookies {
         println!("  Name: {}, Value: {}, Domain: {}, Path: {}, Expires: {:?}, HTTPOnly: {}, Secure: {}", 
                  cookie.name, cookie.value, cookie.domain, cookie.path, cookie.expires, cookie.http_only, cookie.secure);
     }
     println!("---- 临时调试代码 结束 ----\n");
    // ---- 临时调试代码 结束 ----
    
    let important_cookies = cookies::get_important_cookies(&cookies);
    let has_sessdata = important_cookies.iter().any(|c| c.name == "SESSDATA");
    let has_bili_jct = important_cookies.iter().any(|c| c.name == "bili_jct");
    let has_dedeuserid = important_cookies.iter().any(|c| c.name == "DedeUserID");
    let has_key_cookies = has_sessdata && has_dedeuserid; // bili_jct is important but sometimes not easily available initially
    
    if has_key_cookies {
        println!("{}", "✓ 已成功获取关键Cookie".green().bold());
        if let Some(uid_cookie) = cookies.iter().find(|c| c.name == "DedeUserID") {
            println!("{} {}", "用户ID:".cyan(), uid_cookie.value);
        }
        if !login_result.username.is_empty() {
            println!("{} {}", "用户名:".cyan(), login_result.username);
        } else {
            println!("{} {}", "用户名:".cyan(), "未知".yellow());
        }
        println!("{} {}", "Cookie数量:".cyan(), cookies.len());
        println!("\n{}", "关键Cookie (部分):".yellow().bold());
        for cookie_name in ["SESSDATA", "DedeUserID", "bili_jct", "sid"] {
            if let Some(cookie) = cookies.iter().find(|c| c.name == cookie_name) {
                println!("  {}: {}", cookie.name.cyan(), cookie.value);
            }
        }
        if !has_bili_jct {
            println!("\n{}", "注意: 未能自动获取bili_jct(CSRF令牌)。部分操作可能受限。".yellow());
        }
    } else {
        println!("{}", "⚠ 警告: 未获取到足够的关键Cookie (SESSDATA 和 DedeUserID)".yellow().bold());
    }
    
    let (output_format_to_use, default_filename_stem, default_extension) = 
        match cli.format {
            OutputFormat::Json => (OutputFormat::Json, "bilicookies-rs", "json"),
            OutputFormat::Netscape => (OutputFormat::Netscape, "bilicookies-rs", "txt"),
            OutputFormat::KeyValue => (OutputFormat::KeyValue, "bilicookies-rs-kv", "txt"),
            OutputFormat::Toml => (OutputFormat::Toml, "bilicookies-rs", "toml"),
            OutputFormat::Csv => (OutputFormat::Csv, "bilicookies-rs", "csv"),
        };

    let formatted_output = match output_format_to_use {
        OutputFormat::Json => output::format_as_json(&cookies)?,
        OutputFormat::Netscape => output::format_as_netscape(&cookies)?,
        OutputFormat::KeyValue => output::format_as_key_value(&cookies)?,
        OutputFormat::Toml => output::format_as_toml(&cookies)?,
        OutputFormat::Csv => output::format_as_csv(&cookies)?,
    };
    
    if let Some(output_path_str) = cli.output {
        output::save_to_file(&formatted_output, &output_path_str)?;
        println!("\n{} {}", "Cookies已保存到:".green(), output_path_str);
    } else {
        // Default file output logic
        let current_dir = std::env::current_dir()?;
        let filename = format!("{}.{}", default_filename_stem, default_extension);
        let default_output_path = current_dir.join(filename);
        
        // Ensure parent directory exists (though current_dir usually does)
        if let Some(parent) = default_output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        fs::write(&default_output_path, &formatted_output)?;
        println!(
            "\n{} {} (格式: {:?})", 
            "Cookies已默认保存到:".green(), 
            default_output_path.display(),
            output_format_to_use
        );
        // Optionally, still print to console if not saved via -o
        // println!("\n{}\n{}", "完整Cookies:".green(), formatted_output);
    }
    
    println!("{}", "操作完成!".green().bold());
    Ok(())
}
