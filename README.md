# <div align="center">🍪 bilicookies-rs</div>

<div align="center"><em>一个通过 B 站二维码扫描登录并获取 Cookies 的 Rust 工具，支持多种输出格式。</em></div>
<br>
<div align="center">
  <a href="https://github.com/MechNexusLab/bilicookies-rs/releases"><img src="https://img.shields.io/badge/version-1.0.0-blue?style=for-the-badge" alt="Version"></a>
  <a href="https://github.com/MechNexusLab/bilicookies-rs/blob/main/LICENSE"><img src="https://img.shields.io/github/license/MechNexusLab/bilicookies-rs?style=for-the-badge" alt="License"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust%20edition-2021-orange?style=for-the-badge&logo=rust&logoColor=white" alt="Rust Edition"></a>
  <a href="https://github.com/MechNexusLab/bilicookies-rs/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/MechNexusLab/bilicookies-rs/release.yml?branch=main&style=for-the-badge&logo=githubactions&logoColor=white" alt="Build Status"></a>
</div>
<div align="center">
  <a href="https://github.com/MechNexusLab/bilicookies-rs/commits/main"><img src="https://img.shields.io/badge/updated-2025--06--02-0097A7?style=for-the-badge&logo=calendar&logoColor=white" alt="Last Updated"></a>
</div>

## 主要功能

- **二维码登录**: 在终端显示二维码，同时自动保存为 `qrcode.png` 文件，方便用户通过 B 站移动客户端扫描登录。
- **Cookies 提取**: 成功登录后，自动从 B 站服务器提取并解析 Cookies (例如 `SESSDATA`, `bili_jct`, `DedeUserID` 等关键信息)。
- **多种输出格式**: 支持将获取到的 Cookies 保存为多种常用格式，满足不同场景的需求：
  - JSON (`bilicookies-rs.json`)
  - Netscape `cookies.txt` (`bilicookies-rs.txt`)
  - 键值对 (`bilicookies-rs-kv.txt`)
  - TOML (`bilicookies-rs.toml`)
  - CSV (`bilicookies-rs.csv`)
- **灵活的文件输出**: 用户可以通过命令行参数指定输出文件的名称和路径，默认为当前工作目录。
- **用户信息展示**: 登录成功后，会显示用户 ID 和用户名（如果可用）。

## 安装

### 环境要求

- **Rust**: 请确保您已安装 Rust 工具链。如果尚未安装，请访问 [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) 获取安装指南。

### 从源码编译

1.  克隆本仓库到本地：
    ```bash
    git clone https://github.com/MechNexusLab/bilicookies-rs.git
    ```
2.  进入项目目录：
    ```bash
    cd bilicookies-rs
    ```
3.  编译项目：
    ```bash
    cargo build --release
    ```
    编译成功后，可执行文件将位于 `target/release/bilicookies-rs` (Windows 系统下为 `bilicookies-rs.exe`)。

## 使用方法

### 基本步骤

1.  **运行程序**:
    打开您的终端（命令行界面），导航到可执行文件所在的目录 (通常是 `target/release/`)，然后运行程序：

    ```bash
    ./bilicookies-rs # Linux 或 macOS
    .\\bilicookies-rs.exe # Windows
    ```

    或者，如果您已将可执行文件路径添加到了系统环境变量 `PATH` 中，可以直接运行 `bilicookies-rs`。

2.  **扫描二维码**:
    程序启动后，会在终端中显示一个二维码，同时会在程序运行的当前目录下生成一个 `qrcode.png` 文件。请使用您的 B 站手机客户端扫描此二维码并确认登录。

3.  **获取 Cookies**:
    扫描并确认登录后，程序会自动获取 Cookies，并在终端显示部分关键信息（如用户 ID、用户名和部分 Cookie 值）。
    默认情况下，Cookies 会以 Netscape 格式保存到当前工作目录下的 `bilicookies-rs.txt` 文件中。

### 命令行参数

您可以通过 `--help` 参数查看所有可用的命令行选项：

```bash
bilicookies-rs --help
```

以下是一些常用参数：

- `-f, --format <FORMAT>`: 指定输出的 Cookies 格式。
  可选值: `json`, `netscape` (默认), `key-value`, `toml`, `csv`.
  示例: `bilicookies-rs --format json`

- `-o, --output <OUTPUT_PATH>`: 指定保存 Cookies 的文件路径和名称。
  如果未指定，则会根据选择的格式生成默认文件名 (例如 `bilicookies-rs.json`, `bilicookies-rs.txt`) 并保存在当前工作目录。
  示例: `bilicookies-rs --output my_cookies.txt`
  示例 (指定格式和输出路径): `bilicookies-rs --format json --output /path/to/my_bili_cookies.json`

### 输出文件说明

- **二维码图片**: `qrcode.png` (始终在程序运行的当前工作目录生成)。
- **Cookies 文件**: 默认文件名和格式取决于 `--format` 参数，默认保存位置为程序运行的当前工作目录，可通过 `--output` 参数自定义。

## 项目结构

```
src/
├── main.rs      # 程序主入口，处理命令行参数，协调各模块
├── api.rs       # 封装与B站API的交互逻辑 (如获取二维码、轮询登录状态)
├── auth.rs      # 处理认证和登录流程
├── config.rs    # (未来可能用于) 应用配置管理
├── cookies.rs   # Cookies数据结构定义、提取和关键信息筛选
├── error.rs     # 自定义错误类型和错误处理
└── output.rs    # 负责将Cookies格式化并输出到文件或控制台
```

## 注意事项

- 本工具仅供学习和研究 Rust 及网络编程之用，请勿用于任何非法用途。
- 获取到的 Cookies 包含敏感信息，等同于您的账户凭证，请务必妥善保管，切勿泄露或分享给他人。
- B 站的 API 接口可能会发生变化。如果程序无法正常工作，请检查相关 API 是否已更新。欢迎提交 Issue 或 Pull Request 参与改进。

## 贡献

欢迎各种形式的贡献，包括但不限于：

- 报告 Bug
- 提交功能建议
- 发送 Pull Request

## 许可证

本项目基于 [MIT License](LICENSE) 开源
