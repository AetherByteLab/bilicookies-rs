use anyhow::Result;
use colored::*;
use image::{ImageBuffer, Luma};
use qrcode::QrCode;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

use crate::api::{
    create_client, generate_qrcode, get_user_info, poll_qrcode, QrCodePollData, UserInfoData,
};
use crate::error::BiliError;

/// 登录成功后的结果
#[derive(Debug)]
pub struct LoginResult {
    pub client: Client,
    pub refresh_token: String,
    pub uid: u64,
    pub username: String,
}

// 二维码状态常量
const QR_CODE_STATUS_SUCCESS: i32 = 0; // 成功 (已确认)
const QR_CODE_STATUS_SCANNED: i32 = 86038; // 已扫描，待确认
const QR_CODE_STATUS_EXPIRED: i32 = 86039; // 二维码已过期
const QR_CODE_STATUS_NOT_SCANNED_YET: i32 = 86101; // 未扫描 (或还在初始化)

// 二维码登录流程
pub async fn login_with_qrcode() -> Result<LoginResult> {
    let client = create_client()?;
    let qr_data = generate_qrcode(&client).await?;
    let code_for_image = QrCode::new(qr_data.url.as_bytes())?; // For image generation
    let code_for_terminal = QrCode::new(qr_data.url.as_bytes())?; // For terminal rendering

    // --- 保存二维码到文件 ---
    let width = code_for_image.width();
    if width == 0 {
        println!("{}", "无法生成二维码图像：宽度为0".red());
    } else {
        let colors = code_for_image.to_colors(); // Vec<qrcode::Color>
        let scale = 6u32;
        let image_size = (width as u32) * scale;
        let mut img_buf = ImageBuffer::new(image_size, image_size);

        for y_qr in 0..width {
            for x_qr in 0..width {
                let module_index = y_qr * width + x_qr;
                let pixel_color = match colors[module_index] {
                    qrcode::Color::Dark => Luma([0u8]),
                    qrcode::Color::Light => Luma([255u8]),
                };

                for y_offset in 0..scale {
                    for x_offset in 0..scale {
                        img_buf.put_pixel(
                            (x_qr as u32) * scale + x_offset,
                            (y_qr as u32) * scale + y_offset,
                            pixel_color,
                        );
                    }
                }
            }
        }
        match img_buf.save("qrcode.png") {
            Ok(_) => println!("二维码图片已保存为 qrcode.png, 您也可以扫描此文件。"),
            Err(e) => println!(
                "无法保存二维码图片到文件: {}. 请扫描下方终端二维码。",
                e.to_string().red()
            ),
        }
    }
    // --- 结束保存二维码到文件 ---

    // --- 终端二维码 ---
    let terminal_qr_string = code_for_terminal
        .render::<qrcode::render::unicode::Dense1x2>()
        .dark_color(qrcode::render::unicode::Dense1x2::Light)
        .light_color(qrcode::render::unicode::Dense1x2::Dark)
        .build();
    println!("\n{}", terminal_qr_string);
    println!(
        "{}",
        "请使用B站手机APP扫描上方二维码或 qrcode.png 文件并确认登录...".yellow()
    );

    let mut poll_attempts = 0;
    let max_poll_attempts = 90;

    loop {
        if poll_attempts >= max_poll_attempts {
            println!("\n{}", "✗ 轮询超时，二维码可能已过期或网络问题。".red());
            return Err(BiliError::LoginError("二维码轮询超时或已过期".to_string()).into());
        }
        sleep(Duration::from_secs(2)).await;
        poll_attempts += 1;

        let poll_data: QrCodePollData = poll_qrcode(&client, &qr_data.qrcode_key).await?;

        match poll_data.code {
            QR_CODE_STATUS_SUCCESS => {
                println!("\n{}", "✓ 扫码成功!".green());
                println!("正在获取用户信息...");

                let refresh_token = poll_data.refresh_token;

                let user_info: UserInfoData = get_user_info(&client).await?;
                let uid = user_info.mid;
                let username = user_info.uname;

                return Ok(LoginResult {
                    client,
                    refresh_token,
                    uid,
                    username,
                });
            }
            QR_CODE_STATUS_NOT_SCANNED_YET => {
                print!(".");
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            QR_CODE_STATUS_SCANNED => {
                println!("\n{}", "已扫描，等待App确认...".yellow());
            }
            QR_CODE_STATUS_EXPIRED => {
                println!("\n{}", "✗ 二维码已过期".red());
                return Err(BiliError::LoginError("二维码已过期".to_string()).into());
            }
            other_code => {
                println!(
                    "\n{}",
                    format!("未知轮询状态，代码: {}。将继续尝试...", other_code).yellow()
                );
            }
        }
    }
}
