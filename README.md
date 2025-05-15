# bilicookies-rs

B站扫码登录获取cookies的Rust实现，支持多种输出格式。

## 功能特点

- 通过扫描二维码实现B站账号登录。
    - 二维码会直接在终端显示。
    - 二维码图片会自动保存为 `qrcode.png` 文件于程序运行目录下。
- 自动提取并保存登录后的 cookies (例如 `SESSDATA`, `bili_jct`, `DedeUserID` 等)。
- 支持将 cookies 信息保存为多种格式，默认保存在运行目录下的 `./cookies/` 文件夹中：
    - 纯文本 (`cookies_raw.txt`)
    - JSON 格式 (`cookies.json`)
    - Netscape `cookies.txt` 格式 (`netscape_cookies.txt`)
    - LWP `cookies.txt` 格式 (`lwp_cookies.txt`)
- (若已实现) 支持从命令行参数传入B站API返回的JSON数据直接解析和保存Cookie信息。
- 模块化设计，便于扩展。

## 安装

### 从源码编译

```bash
# 确保已安装 Rust 工具链 (https://www.rust-lang.org/tools/install)
git clone https://github.com/yourusername/bilicookies-rs.git # 请替换为你的仓库实际地址
cd bilicookies-rs
cargo build --release
```

编译成功后，可执行文件将位于 `target/release/bilicookies-rs`。

## 使用方法

### 基本使用 (通过二维码登录)

```bash
# 在项目根目录下运行
./target/release/bilicookies-rs
```
程序将：
1. 生成登录二维码并在终端显示，同时保存为 `qrcode.png`。
2. 等待用户扫描二维码并确认登录。
3. 成功登录后，自动提取 cookies。
4. 将提取的信息以多种格式保存到 `./cookies/` 目录下。

### 命令行参数

请运行 `./target/release/bilicookies-rs --help` 来获取最新和最准确的参数列表。
以下是一些可能的参数（具体实现可能不同）：

- `--formats <FORMATS>`: (可选) 指定输出的格式，用逗号分隔 (例如: `json,netscape`)。如果未指定，可能会使用一组默认格式。
- `--output-dir <DIRECTORY>`: (可选) 指定保存输出文件的目录 (默认为 `./cookies`)。
- `--json-input <JSON_PATH_OR_STRING>`: (可选, 若已实现) 从指定的JSON文件路径或JSON字符串解析cookies，此模式下将跳过二维码登录流程。

## 项目结构

```
src/
├── main.rs      # 程序入口，处理命令行参数，协调各模块工作
├── api.rs       # 封装B站API接口调用 (二维码生成、轮询、用户信息获取等)
├── auth.rs      # 处理认证与登录逻辑 (包括二维码的生成、显示和登录状态轮询)
├── config.rs    # (可能用于) 应用配置管理 (例如默认保存路径、API端点等)
├── cookies.rs   # 核心模块：Cookies的解析、提取、数据结构定义及格式化输出准备
├── error.rs     # 定义项目特定的错误类型和错误处理帮助函数
└── output.rs    # (若单独存在) 负责将格式化后的数据写入不同类型的文件 (此功能也可能整合在 main.rs 或 cookies.rs 中)
```

## 注意事项

- 本工具仅用于学习和研究目的，请勿用于非法用途。
- 请妥善保管获取到的 cookies，避免泄露或分享给他人，这些信息等同于你的账户凭证。
- B站的API接口可能会发生变更，如果程序无法正常工作，请检查相关API是否已更新，并欢迎提交issue或PR。

## 许可证

[MIT](LICENSE) (如果项目使用MIT许可证，请确保根目录下存在 `LICENSE` 文件) 