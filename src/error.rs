use thiserror::Error;

#[derive(Error, Debug)]
pub enum BiliError {
    #[error("网络请求错误: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("JSON解析错误: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("二维码生成错误: {0}")]
    QrCodeError(#[from] qrcode::types::QrError),
    
    #[error("图像处理错误: {0}")]
    ImageError(#[from] image::error::ImageError),
    
    #[error("TOML序列化错误: {0}")]
    TomlError(#[from] toml::ser::Error),
    
    #[error("CSV错误: {0}")]
    CsvError(#[from] csv::Error),
    
    #[error("登录失败: {0}")]
    LoginError(String),
    
    #[error("Cookie提取失败: {0}")]
    CookieError(String),
    
    #[error("API错误: 状态码 {0}, 消息: {1}")]
    ApiError(i32, String),
} 