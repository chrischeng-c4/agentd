# Agentd 安裝指南

Agentd 是用 Rust 編寫的命令行工具，提供多種安裝方式。

## 系統要求

- **作業系統**: macOS, Linux, Windows (WSL2 推薦)
- **Rust**: 1.70 或更高版本
- **Git**: 任何現代版本

## 安裝方式

### 方式 1: 從源碼編譯安裝（推薦）

```bash
# 1. 克隆倉庫
git clone https://github.com/your-username/agentd.git
cd agentd

# 2. 編譯並安裝
cargo install --path .

# 3. 驗證安裝
agentd --version
```

安裝後，`agentd` 命令將在系統 PATH 中可用。

**安裝位置**: `~/.cargo/bin/agentd`

### 方式 2: 使用 Cargo Install（發布到 crates.io 後）

```bash
# 從 crates.io 安裝
cargo install agentd

# 驗證安裝
agentd --version
```

> ⚠️ 目前 Agentd 尚未發布到 crates.io，請使用方式 1。

### 方式 3: 下載預編譯二進制文件（未來）

```bash
# macOS (Apple Silicon)
curl -L https://github.com/your-username/agentd/releases/latest/download/agentd-aarch64-apple-darwin.tar.gz | tar xz
sudo mv agentd /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/your-username/agentd/releases/latest/download/agentd-x86_64-apple-darwin.tar.gz | tar xz
sudo mv agentd /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/your-username/agentd/releases/latest/download/agentd-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv agentd /usr/local/bin/
```

> ⚠️ 預編譯二進制文件尚未提供，請使用方式 1。

### 方式 4: 開發模式（不安裝）

```bash
# 克隆倉庫
git clone https://github.com/your-username/agentd.git
cd agentd

# 編譯
cargo build --release

# 使用完整路徑運行
./target/release/agentd --version

# 或創建別名（添加到 ~/.bashrc 或 ~/.zshrc）
alias agentd="/path/to/agentd/target/release/agentd"
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
agentd --version
# 輸出: agentd 0.1.0

# 查看幫助
agentd --help

# 測試初始化
mkdir test-project
cd test-project
agentd init --name "Test Project"
```

## 配置 AI 工具整合

Agentd 直接調用 AI CLI 工具，無需配置腳本：

```bash
# 1. 初始化專案
cd your-project
agentd init

# 2. 確保 AI CLI 工具已安裝並在 PATH 中
which gemini  # Gemini CLI
which claude  # Claude Code
which codex   # Codex CLI (如果可用)
```

### 配置環境變量

創建 `.env` 文件：

```bash
# API Keys (如果需要)
ANTHROPIC_API_KEY=sk-ant-...
GEMINI_API_KEY=...
OPENAI_API_KEY=sk-...
```

## 安裝 AI CLI 工具（必需）

Agentd 需要以下 CLI 工具才能正常工作：

### Gemini CLI

```bash
npm install -g @google/generative-ai-cli
# 或訪問: https://ai.google.dev/gemini-api/docs/cli
```

### Claude Code

```bash
# 從 Anthropic 下載
# 訪問: https://claude.ai/code
```

### Codex CLI（如果可用）

```bash
# 根據你的 Codex 提供商安裝
# 確保 'codex' 命令在 PATH 中可用
```

## 更新 Agentd

### 從源碼更新

```bash
cd agentd
git pull
cargo install --path . --force
```

### 從 crates.io 更新

```bash
cargo install agentd --force
```

## 卸載

```bash
# 刪除二進制文件
cargo uninstall agentd

# 或手動刪除
rm ~/.cargo/bin/agentd
```

## 故障排除

### 問題: `cargo: command not found`

**解決方案**: 安裝 Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 問題: `agentd: command not found` (安裝後)

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

### 問題: AI 工具未找到

**解決方案**:
1. 確保 AI CLI 工具已安裝並在 PATH 中
   ```bash
   which gemini
   which claude
   which codex
   ```
2. 檢查環境變量是否正確設置
3. 編輯 `agentd/config.toml` 配置模型選擇

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
COPY --from=builder /app/target/release/agentd /usr/local/bin/
ENTRYPOINT ["agentd"]
```

```bash
# 構建 Docker 映像
docker build -t agentd .

# 運行
docker run -v $(pwd):/workspace -w /workspace agentd init
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

或直接開始：
```bash
agentd init
agentd proposal my-first-change "Add awesome feature"
```

## 需要幫助？

- 📖 查看 [文檔](README.md)
- 🐛 報告問題: [GitHub Issues](https://github.com/your-username/agentd/issues)
- 💬 討論: [GitHub Discussions](https://github.com/your-username/agentd/discussions)
