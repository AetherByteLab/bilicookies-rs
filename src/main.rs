use anyhow::Result;
use clap::{Parser, ValueEnum};
use colored::Colorize;

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
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
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
    
    // 执行登录流程
    let login_result = auth::login_with_qrcode().await?;
    
    // 处理登录结果，获取cookies
    let cookies = cookies::extract_cookies(&login_result).await?;
    
    // 检查是否提取到核心cookie
    let important_cookies = cookies::get_important_cookies(&cookies);
    let has_sessdata = important_cookies.iter().any(|c| c.name == "SESSDATA");
    let has_bili_jct = important_cookies.iter().any(|c| c.name == "bili_jct");
    let has_dedeuserid = important_cookies.iter().any(|c| c.name == "DedeUserID");
    
    // 通常bili_jct是必须的，但有时候可能获取不到，只要有SESSDATA和DedeUserID就可以
    let has_key_cookies = has_sessdata && has_dedeuserid;
    
    if has_key_cookies {
        println!("{}", "✓ 已成功获取关键Cookie".green().bold());
        // 显示用户信息
        if let Some(uid) = important_cookies.iter().find(|c| c.name == "DedeUserID") {
            println!("{} {}", "用户ID:".cyan(), uid.value);
        }
        
        // 尝试显示用户名
        if login_result.username.is_empty() {
            println!("{} {}", "用户名:".cyan(), "未知".yellow());
        } else {
            println!("{} {}", "用户名:".cyan(), login_result.username);
        }
        
        println!("{} {}", "Cookie数量:".cyan(), cookies.len());
        
        // 显示关键cookie的值
        println!("\n{}", "关键Cookie:".yellow().bold());
        for cookie in &important_cookies {
            println!("{}: {}", cookie.name.cyan(), cookie.value);
        }
        
        // 显示bili_jct缺失警告
        if !has_bili_jct {
            println!("\n{}", "注意: 未获取到bili_jct(CSRF令牌)，可能影响部分需要提交数据的操作".yellow());
        }
    } else {
        println!("{}", "⚠ 警告: 未获取到完整的关键Cookie".yellow().bold());
    }
    
    // 根据输出格式输出cookies
    let formatted_output = match cli.format {
        OutputFormat::Json => output::format_as_json(&cookies)?,
        OutputFormat::Netscape => output::format_as_netscape(&cookies)?,
        OutputFormat::KeyValue => output::format_as_key_value(&cookies)?,
        OutputFormat::Toml => output::format_as_toml(&cookies)?,
        OutputFormat::Csv => output::format_as_csv(&cookies)?,
    };
    
    // 保存或输出结果
    if let Some(output_path) = cli.output {
        output::save_to_file(&formatted_output, &output_path)?;
        println!("\n{} {}", "Cookies已保存到:".green(), output_path);
    } else {
        println!("\n{}\n{}", "完整Cookies:".green(), formatted_output);
    }
    
    println!("{}", "登录成功!".green().bold());
    Ok(())
}
