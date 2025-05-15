# bilicookies-rs

B站扫码登录获取cookies的Rust实现，支持多种输出格式。

## 功能特点

- 通过扫描二维码实现B站账号登录
- 自动提取并保存登录后的cookies
- 支持多种输出格式：
  - JSON格式
  - Netscape cookies.txt格式 (适用于curl、wget等工具)
  - 键值对格式
  - TOML格式
  - CSV格式
- 可选择保存到文件或直接输出到控制台
- 模块化设计，便于扩展

## 安装

### 从源码编译

```bash
git clone https://github.com/yourusername/bilicookies-rs.git
cd bilicookies-rs
cargo build --release
```

编译成功后，可执行文件将位于 `target/release/bilicookies-rs`。

## 使用方法

### 基本使用

```bash
# 使用默认JSON格式输出
bilicookies-rs

# 指定输出格式
bilicookies-rs -f json
bilicookies-rs -f netscape
bilicookies-rs -f keyvalue
bilicookies-rs -f toml
bilicookies-rs -f csv

# 保存到文件
bilicookies-rs -o cookies.json
bilicookies-rs -f netscape -o cookies.txt
```

### 命令行参数

- `-f, --format <FORMAT>`: 指定输出格式 (json, netscape, keyvalue, toml, csv)
- `-o, --output <FILE>`: 将输出保存到指定文件
- `-h, --help`: 显示帮助信息
- `-V, --version`: 显示版本信息

## 项目结构

```
src/
├── main.rs      # 程序入口
├── api.rs       # B站API相关
├── auth.rs      # 认证与登录
├── config.rs    # 配置管理
├── cookies.rs   # Cookies处理
├── error.rs     # 错误处理
└── output.rs    # 输出格式化
```

## 注意事项

- 本工具仅用于学习和研究目的，请勿用于非法用途
- 请勿将获取到的cookies分享给他人
- B站API可能随时变更，如遇问题请提交issue或PR

## 许可证

[MIT](LICENSE) 