---
title: Codex CLI 使用須知
source: 實際測試發現的行為
date: 2026-01-23
---

# Codex CLI 使用須知

## Headless 模式 (codex exec)

`codex exec` 命令**不接受 stdin 輸入**。

### 正確用法

```bash
# Prompt 必須作為 positional argument 傳遞
codex exec "Your prompt here"

# 帶選項
codex exec --model o3-mini --full-auto "Your prompt here"
```

### 錯誤用法

```bash
# ❌ 這不會工作 - stdin 會被忽略
echo "Your prompt" | codex exec
```

## Resume 模式

```bash
# 繼續上一個 session
codex resume --last
```

## 在 agentd 中的實現

`src/orchestrator/codex.rs` 使用 `LlmArg::Prompt` 將 prompt 放入 CLI 參數：

```rust
fn build_args_with_prompt(&self, prompt: &str, complexity: Complexity, resume: bool) -> Vec<String> {
    let mut args = vec![LlmArg::FullAuto, LlmArg::Json];
    // ... model, reasoning ...
    
    // Prompt 必須作為 CLI arg，不能用 stdin
    if !prompt.is_empty() {
        args.push(LlmArg::Prompt(prompt.to_string()));
    }
    
    LlmProvider::Codex.build_args(&args, resume)
}
```

然後調用時 stdin 傳空字串：

```rust
self.runner.run_llm(LlmProvider::Codex, args, env, "", true).await
```