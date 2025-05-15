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
        println!("{} {}", "Cookies已保存到:".green(), output_path);
    } else {
        println!("{}\n{}", "Cookies:".green(), formatted_output);
    }
    
    println!("{}", "登录成功!".green().bold());
    Ok(())
}
