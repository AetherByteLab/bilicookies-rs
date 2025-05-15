# bilicookies-rs

B站扫码登录获取cookies的Rust实现，支持多种输出格式。

## 功能特点

- 通过扫描二维码实现B站账号登录。
    - 二维码会直接在终端显示。
    - 二维码图片会自动保存为 `qrcode.png` 文件，位于程序执行时的当前工作目录。
- 自动提取并保存登录后的 cookies (例如 `SESSDATA`, `bili_jct`, `DedeUserID` 等)。
- 支持将 cookies 信息保存为多种格式，默认保存在程序执行时当前工作目录下的 `./cookies/` 文件夹中：
    - 纯文本 (`cookies_raw.txt`)
    - JSON 格式 (`cookies.json`)
    - Netscape `cookies.txt` 格式 (`netscape_cookies.txt`)
    - LWP `cookies.txt` 格式 (`lwp_cookies.txt`)
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

1.  **编译项目**：
    首先，请按照上一节“安装”中的“从源码编译”步骤编译项目。编译成功后，可执行文件将位于项目根目录下的 `target/release/` 子目录中 (例如 `bilicookies-rs`，在Windows系统上通常是 `bilicookies-rs.exe`)。

2.  **运行程序**：
    a. 打开你的终端（命令行界面）。
    b. 导航到可执行文件所在的 `target/release/` 目录。例如，如果你的项目路径是 `/path/to/your/project/bilicookies-rs`，则输入：
       ```bash
       cd /path/to/your/project/bilicookies-rs/target/release
       ```
    c. 从 `target/release` 目录中执行指令：
       ```bash
       bilicookies-rs
       ```

3.  **程序流程与输出文件**：
    程序启动后，将会：
    a. 在终端中显示登录二维码，并将此二维码图片保存为 `qrcode.png`。
    b. 等待用户使用B站移动应用扫描二维码并确认登录。
    c. 成功登录后，自动从B站服务器提取cookies。
    d. 将提取到的cookies信息以多种预设格式保存到名为 `cookies` 的子文件夹中。

    **重要：关于输出文件的位置**
    -   当你遵循上述步骤，在 `target/release` 目录中启动并运行程序时：
        -   二维码图片 `qrcode.png` 将会保存在 `target/release/qrcode.png`。
        -   包含cookies文件的 `cookies` 文件夹将会创建在 `target/release/cookies/`。
    -   **替代方式**：如果你选择从项目的根目录来执行程序 (例如，通过命令 `./target/release/bilicookies-rs`)，那么 `qrcode.png` 和 `cookies` 文件夹则会创建在项目的根目录中。
    -   总而言之，所有相对路径的输出文件都是相对于程序启动时的**当前工作目录**来创建的。

### 命令行参数

请运行 `bilicookies-rs --help` 来获取最新和最准确的参数列表。
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
└── output.rs    # 负责将格式化后的数据写入不同类型的文件
```

## 注意事项

- 本工具仅用于学习和研究目的，请勿用于非法用途。
- 请妥善保管获取到的 cookies，避免泄露或分享给他人，这些信息等同于你的账户凭证。
- B站的API接口可能会发生变更，如果程序无法正常工作，请检查相关API是否已更新，并欢迎提交issue或PR。

## 许可证

[MIT](LICENSE) (如果项目使用MIT许可证，请确保根目录下存在 `LICENSE` 文件) 