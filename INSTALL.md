# Specter 安裝指南

Specter 是用 Rust 編寫的命令行工具，提供多種安裝方式。

## 系統要求

- **作業系統**: macOS, Linux, Windows (WSL2 推薦)
- **Rust**: 1.70 或更高版本
- **Git**: 任何現代版本

## 安裝方式

### 方式 1: 從源碼編譯安裝（推薦）

```bash
# 1. 克隆倉庫
git clone https://github.com/your-username/specter.git
cd specter

# 2. 編譯並安裝
cargo install --path .

# 3. 驗證安裝
specter --version
```

安裝後，`specter` 命令將在系統 PATH 中可用。

**安裝位置**: `~/.cargo/bin/specter`

### 方式 2: 使用 Cargo Install（發布到 crates.io 後）

```bash
# 從 crates.io 安裝
cargo install specter

# 驗證安裝
specter --version
```

> ⚠️ 目前 Specter 尚未發布到 crates.io，請使用方式 1。

### 方式 3: 下載預編譯二進制文件（未來）

```bash
# macOS (Apple Silicon)
curl -L https://github.com/your-username/specter/releases/latest/download/specter-aarch64-apple-darwin.tar.gz | tar xz
sudo mv specter /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/your-username/specter/releases/latest/download/specter-x86_64-apple-darwin.tar.gz | tar xz
sudo mv specter /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/your-username/specter/releases/latest/download/specter-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv specter /usr/local/bin/
```

> ⚠️ 預編譯二進制文件尚未提供，請使用方式 1。

### 方式 4: 開發模式（不安裝）

```bash
# 克隆倉庫
git clone https://github.com/your-username/specter.git
cd specter

# 編譯
cargo build --release

# 使用完整路徑運行
./target/release/specter --version

# 或創建別名（添加到 ~/.bashrc 或 ~/.zshrc）
alias specter="/path/to/specter/target/release/specter"
```

## 安裝 Rust（如果尚未安裝）

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Windows

下載並運行 [rustup-init.exe](https://rustup.rs/)

## 驗證安裝

```bash
# 檢查版本
specter --version
# 輸出: specter 0.1.0

# 查看幫助
specter --help

# 測試初始化
mkdir test-project
cd test-project
specter init --name "Test Project"
```

## 配置 AI 工具整合

Specter 需要配置 AI 工具腳本才能正常工作：

```bash
# 1. 初始化專案
cd your-project
specter init

# 2. 複製示例腳本
cp /path/to/specter/examples/scripts/* .specter/scripts/

# 3. 使腳本可執行（Unix 系統）
chmod +x .specter/scripts/*.sh

# 4. 編輯腳本以整合你的 AI 工具
nano .specter/scripts/gemini-proposal.sh
nano .specter/scripts/codex-challenge.sh
# ... 其他腳本
```

### 配置環境變量

創建 `.env` 文件：

```bash
# API Keys
ANTHROPIC_API_KEY=sk-ant-...
GEMINI_API_KEY=...
OPENAI_API_KEY=sk-...

# CLI 路徑（可選）
GEMINI_CLI=/usr/local/bin/gemini
CODEX_CLI=/usr/local/bin/codex
CLAUDE_CLI=/usr/local/bin/claude
```

## 安裝 AI CLI 工具（可選）

### Gemini CLI

```bash
npm install -g gemini-cli
# 或訪問: https://geminicli.com
```

### Claude Code

```bash
# 從 Anthropic 下載
# 訪問: https://claude.ai/code
```

### Codex（如果可用）

```bash
# 根據你的 Codex 提供商安裝
```

## 更新 Specter

### 從源碼更新

```bash
cd specter
git pull
cargo install --path . --force
```

### 從 crates.io 更新

```bash
cargo install specter --force
```

## 卸載

```bash
# 刪除二進制文件
cargo uninstall specter

# 或手動刪除
rm ~/.cargo/bin/specter
```

## 故障排除

### 問題: `cargo: command not found`

**解決方案**: 安裝 Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 問題: `specter: command not found` (安裝後)

**解決方案**: 確保 `~/.cargo/bin` 在 PATH 中
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### 問題: 編譯錯誤

**解決方案**: 更新 Rust
```bash
rustup update stable
```

### 問題: 腳本執行失敗

**解決方案**: 檢查腳本權限
```bash
chmod +x .specter/scripts/*.sh
```

### 問題: AI 工具未找到

**解決方案**:
1. 檢查 AI CLI 是否已安裝
2. 檢查環境變量是否正確設置
3. 編輯 `.specter/config.toml` 設置正確的命令路徑

## 性能優化

### 發布模式編譯（更快的執行速度）

```bash
cargo build --release
cargo install --path . --profile release
```

### 減小二進制文件大小

在 `Cargo.toml` 中添加：
```toml
[profile.release]
strip = true
opt-level = "z"
lto = true
codegen-units = 1
```

然後重新編譯：
```bash
cargo build --release
```

## Docker 安裝（可選）

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/specter /usr/local/bin/
ENTRYPOINT ["specter"]
```

```bash
# 構建 Docker 映像
docker build -t specter .

# 運行
docker run -v $(pwd):/workspace -w /workspace specter init
```

## 支持的平台

| 平台 | 架構 | 狀態 |
|------|------|------|
| macOS | x86_64 | ✅ 支持 |
| macOS | aarch64 (Apple Silicon) | ✅ 支持 |
| Linux | x86_64 | ✅ 支持 |
| Linux | aarch64 | ✅ 支持 |
| Windows | x86_64 | ⚠️ 需要 WSL2 |

## 下一步

安裝完成後，請閱讀：
- [README.md](README.md) - 使用指南
- [examples/scripts/README.md](examples/scripts/README.md) - AI 整合範例

或直接開始：
```bash
specter init
specter proposal my-first-change "Add awesome feature"
```

## 需要幫助？

- 📖 查看 [文檔](README.md)
- 🐛 報告問題: [GitHub Issues](https://github.com/your-username/specter/issues)
- 💬 討論: [GitHub Discussions](https://github.com/your-username/specter/discussions)
