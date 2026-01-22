use super::cli_mapper::LlmProvider;
use anyhow::{Context, Result};
use indicatif::{ProgressBar as IndicatifProgressBar, ProgressStyle};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// Usage metrics returned from an LLM call
#[derive(Debug, Clone, Default)]
pub struct UsageMetrics {
    /// Number of input tokens
    pub tokens_in: Option<u64>,
    /// Number of output tokens
    pub tokens_out: Option<u64>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Cost in USD (if provided directly by CLI)
    pub cost_usd: Option<f64>,
    /// Gemini session ID (from init message)
    pub session_id: Option<String>,
}

/// Gemini stream-json init response format
/// Actual format: {"type":"init","session_id":"..."}
#[derive(Debug, Deserialize)]
struct GeminiInitResponse {
    #[serde(rename = "type")]
    response_type: Option<String>,
    session_id: Option<String>,
}

/// Gemini stream-json usage response format
/// Actual format: {"type":"result","stats":{"input_tokens":N,"output_tokens":N,...}}
#[derive(Debug, Deserialize)]
struct GeminiUsageResponse {
    #[serde(rename = "type")]
    response_type: Option<String>,
    stats: Option<GeminiStats>,
}

#[derive(Debug, Deserialize)]
struct GeminiStats {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    #[allow(dead_code)]
    total_tokens: Option<u64>,
    #[allow(dead_code)]
    duration_ms: Option<u64>,
}

/// Claude stream-json usage response format
/// Actual format: {"type":"result","total_cost_usd":N,"usage":{"input_tokens":N,"output_tokens":N,...},"duration_ms":N}
#[derive(Debug, Deserialize)]
struct ClaudeUsageResponse {
    #[serde(rename = "type")]
    response_type: Option<String>,
    usage: Option<ClaudeUsage>,
    total_cost_usd: Option<f64>,
    duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cache_read_input_tokens: Option<u64>,
    cache_creation_input_tokens: Option<u64>,
}

/// Codex JSON usage response format
/// Actual format: {"type":"turn.completed","usage":{"input_tokens":N,"cached_input_tokens":N,"output_tokens":N}}
#[derive(Debug, Deserialize)]
struct CodexUsageResponse {
    #[serde(rename = "type")]
    response_type: Option<String>,
    usage: Option<CodexUsage>,
}

#[derive(Debug, Deserialize)]
struct CodexUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cached_input_tokens: Option<u64>,
}

/// Runner for executing CLI commands directly
#[derive(Default)]
pub struct ScriptRunner {}

impl ScriptRunner {
    pub fn new() -> Self {
        Self {}
    }

    // =========================================================================
    // Direct CLI Execution (Rust-based, no shell scripts)
    // =========================================================================

    /// Run an LLM CLI command with provider-agnostic interface
    ///
    /// # Arguments
    /// * `provider` - The LLM provider (Gemini, Codex, Claude)
    /// * `args` - CLI arguments (already mapped via LlmProvider::build_args)
    /// * `env` - Environment variables
    /// * `prompt` - The prompt to pipe to stdin
    /// * `show_progress` - Whether to show a progress spinner
    ///
    /// # Returns
    /// A tuple of (output_text, usage_metrics)
    pub async fn run_llm(
        &self,
        provider: LlmProvider,
        args: Vec<String>,
        env: HashMap<String, String>,
        prompt: &str,
        show_progress: bool,
    ) -> Result<(String, UsageMetrics)> {
        self.run_llm_with_cwd(provider, args, env, prompt, show_progress, None).await
    }

    /// Run an LLM CLI command with optional working directory
    ///
    /// # Arguments
    /// * `provider` - The LLM provider (Gemini, Codex, Claude)
    /// * `args` - CLI arguments (already mapped via LlmProvider::build_args)
    /// * `env` - Environment variables
    /// * `prompt` - The prompt to pipe to stdin
    /// * `show_progress` - Whether to show a progress spinner
    /// * `cwd` - Optional working directory (required for Gemini session-scoped operations)
    ///
    /// # Returns
    /// A tuple of (output_text, usage_metrics)
    pub async fn run_llm_with_cwd(
        &self,
        provider: LlmProvider,
        args: Vec<String>,
        env: HashMap<String, String>,
        prompt: &str,
        show_progress: bool,
        cwd: Option<&std::path::Path>,
    ) -> Result<(String, UsageMetrics)> {
        let start = Instant::now();
        let command_name = provider.command();
        let output = self.run_command_with_cwd(command_name, &args, env, prompt, show_progress, cwd).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Parse usage from output
        let mut usage = Self::parse_usage_from_output(&output, provider);
        usage.duration_ms = Some(duration_ms);

        Ok((output, usage))
    }

    /// Parse token usage from CLI output based on provider
    fn parse_usage_from_output(output: &str, provider: LlmProvider) -> UsageMetrics {
        match provider {
            LlmProvider::Gemini => Self::parse_gemini_usage(output),
            LlmProvider::Claude => Self::parse_claude_usage(output),
            LlmProvider::Codex => Self::parse_codex_usage(output),
        }
    }

    /// Parse Gemini stream-json output for usage metadata and session_id
    fn parse_gemini_usage(output: &str) -> UsageMetrics {
        let mut metrics = UsageMetrics::default();

        // Gemini stream-json outputs JSON objects one per line
        // Look for "init" type to extract session_id, and "result" type for stats
        for line in output.lines() {
            // Try to parse init message for session_id (appears early in output)
            if metrics.session_id.is_none() {
                if let Ok(init) = serde_json::from_str::<GeminiInitResponse>(line) {
                    if init.response_type.as_deref() == Some("init") {
                        metrics.session_id = init.session_id;
                    }
                }
            }

            // Try to parse result message for stats (appears at end of output)
            if let Ok(response) = serde_json::from_str::<GeminiUsageResponse>(line) {
                if response.response_type.as_deref() == Some("result") {
                    if let Some(stats) = response.stats {
                        metrics.tokens_in = stats.input_tokens;
                        metrics.tokens_out = stats.output_tokens;
                        metrics.duration_ms = stats.duration_ms;
                    }
                }
            }
        }

        metrics
    }

    /// Parse Claude stream-json output for usage
    fn parse_claude_usage(output: &str) -> UsageMetrics {
        let mut metrics = UsageMetrics::default();

        // Claude outputs JSON objects, look for "result" type with usage
        for line in output.lines().rev() {
            if let Ok(response) = serde_json::from_str::<ClaudeUsageResponse>(line) {
                if response.response_type.as_deref() == Some("result") {
                    if let Some(usage) = response.usage {
                        // Include cache tokens in input count
                        let cache_tokens = usage.cache_read_input_tokens.unwrap_or(0)
                            + usage.cache_creation_input_tokens.unwrap_or(0);
                        metrics.tokens_in = Some(usage.input_tokens.unwrap_or(0) + cache_tokens);
                        metrics.tokens_out = usage.output_tokens;
                    }
                    metrics.duration_ms = response.duration_ms;
                    metrics.cost_usd = response.total_cost_usd;
                    return metrics;
                }
            }
        }

        metrics
    }

    /// Parse Codex JSON output for usage
    fn parse_codex_usage(output: &str) -> UsageMetrics {
        let mut metrics = UsageMetrics::default();

        // Codex --json outputs JSON with turn.completed containing usage
        for line in output.lines().rev() {
            if let Ok(response) = serde_json::from_str::<CodexUsageResponse>(line) {
                if response.response_type.as_deref() == Some("turn.completed") {
                    if let Some(usage) = response.usage {
                        // Include cached tokens in input count
                        let cached = usage.cached_input_tokens.unwrap_or(0);
                        metrics.tokens_in = Some(usage.input_tokens.unwrap_or(0) + cached);
                        metrics.tokens_out = usage.output_tokens;
                        return metrics;
                    }
                }
            }
        }

        metrics
    }

    /// Internal: Run a CLI command with optional working directory
    async fn run_command_with_cwd(
        &self,
        command_name: &str,
        args: &[String],
        env: HashMap<String, String>,
        prompt: &str,
        show_progress: bool,
        cwd: Option<&std::path::Path>,
    ) -> Result<String> {
        let mut cmd = Command::new(command_name);

        // Only use stdin if prompt is provided (backward compatibility)
        // When prompt is in CLI args, pass empty string and skip stdin
        let use_stdin = !prompt.is_empty();

        cmd.args(args)
            .stdin(if use_stdin { Stdio::piped() } else { Stdio::null() })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set working directory if provided
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }

        let progress = if show_progress {
            let pb = IndicatifProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .unwrap()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
            );
            pb.set_message(format!("Running {}...", command_name));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };

        let mut child = cmd
            .spawn()
            .with_context(|| {
                format!(
                    "Command '{}' not found. Please ensure it is installed and in your PATH.",
                    command_name
                )
            })?;

        // Write prompt to stdin only if provided
        if use_stdin {
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(prompt.as_bytes())
                    .await
                    .context("Failed to write to stdin")?;
                stdin.flush().await.context("Failed to flush stdin")?;
                drop(stdin);
            }
        }

        // Stream stdout and stderr concurrently to avoid backpressure deadlock
        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let stderr = child.stderr.take().context("Failed to capture stderr")?;

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut output = String::new();
        let mut stderr_output = String::new();
        let mut stdout_done = false;
        let mut stderr_done = false;

        // Read stdout and stderr concurrently
        // When progress spinner is disabled, stream output to terminal in real-time
        while !stdout_done || !stderr_done {
            tokio::select! {
                line = stdout_reader.next_line(), if !stdout_done => {
                    match line {
                        Ok(Some(line)) => {
                            output.push_str(&line);
                            output.push('\n');

                            if let Some(ref pb) = progress {
                                // Update progress message with recent output
                                let short_line = if line.chars().count() > 60 {
                                    let truncated: String = line.chars().take(60).collect();
                                    format!("{}...", truncated)
                                } else {
                                    line.clone()
                                };
                                pb.set_message(short_line);
                            } else {
                                // Stream to stdout in real-time when no progress spinner
                                println!("{}", line);
                            }
                        }
                        Ok(None) => stdout_done = true,
                        Err(e) => return Err(anyhow::anyhow!("Failed to read stdout: {}", e)),
                    }
                }
                line = stderr_reader.next_line(), if !stderr_done => {
                    match line {
                        Ok(Some(line)) => {
                            stderr_output.push_str(&line);
                            stderr_output.push('\n');

                            // Stream stderr to stderr in real-time (always, even with progress)
                            if progress.is_none() {
                                eprintln!("{}", line);
                            }
                        }
                        Ok(None) => stderr_done = true,
                        Err(e) => return Err(anyhow::anyhow!("Failed to read stderr: {}", e)),
                    }
                }
            }
        }

        let status = child.wait().await?;

        if let Some(pb) = progress {
            pb.finish_and_clear();
        }

        if !status.success() {
            anyhow::bail!(
                "Command '{}' failed with exit code {:?}\nStderr: {}",
                command_name,
                status.code(),
                stderr_output
            );
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_script_runner_creation() {
        let runner = ScriptRunner::new();
        // Just verify it can be created
        drop(runner);
    }

    #[tokio::test]
    async fn test_run_llm_with_nonexistent_provider() {
        let runner = ScriptRunner::new();
        // Gemini CLI might not be installed, which tests the "not found" path
        let args = vec!["--model".to_string(), "test-model".to_string()];
        let env = HashMap::new();
        let prompt = "test prompt";

        // This should fail because the command doesn't exist (or might work if gemini is installed)
        let result = runner.run_llm(LlmProvider::Gemini, args, env, prompt, false).await;

        // We expect this to fail with either "not found" error or auth error
        if result.is_err() {
            let err = result.unwrap_err();
            let err_str = err.to_string();
            assert!(
                err_str.contains("not found")
                    || err_str.contains("Please ensure it is installed")
                    || err_str.contains("auth")
                    || err_str.contains("API"),
                "Error should mention command not found or auth issue: {}",
                err
            );
        }
    }

    #[tokio::test]
    async fn test_run_llm_environment_variables() {
        let runner = ScriptRunner::new();
        let args = vec!["--model".to_string(), "o3-mini".to_string()];
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());
        let prompt = "test";

        // This will fail because 'codex' likely doesn't exist, but tests env handling
        let result = runner.run_llm(LlmProvider::Codex, args, env, prompt, false).await;
        // Just verify it runs without panic - may succeed or fail depending on CLI availability
        let _ = result;
    }

    #[tokio::test]
    async fn test_run_llm_stderr_capture() {
        let runner = ScriptRunner::new();
        let args = vec!["--version".to_string()];
        let env = HashMap::new();
        let prompt = "";

        // Test that stderr is captured when command fails
        let result = runner.run_llm(LlmProvider::Claude, args, env, prompt, false).await;

        // Should fail with stderr in error message if claude CLI is not installed
        if let Err(e) = result {
            let error_msg = e.to_string();
            // Error should mention either "not found" or include stderr
            assert!(
                error_msg.contains("not found") || error_msg.contains("Stderr"),
                "Error message should mention command not found or include stderr: {}",
                error_msg
            );
        }
    }

    // =========================================================================
    // Usage Parsing Tests
    // =========================================================================

    #[test]
    fn test_parse_gemini_usage() {
        let output = r#"{"type":"init","session_id":"abc"}
{"type":"message","role":"user","content":"Hello"}
{"type":"message","role":"assistant","content":"Hi there"}
{"type":"result","status":"success","stats":{"total_tokens":150,"input_tokens":100,"output_tokens":50,"duration_ms":1234}}"#;

        let metrics = ScriptRunner::parse_gemini_usage(output);
        assert_eq!(metrics.tokens_in, Some(100));
        assert_eq!(metrics.tokens_out, Some(50));
        assert_eq!(metrics.duration_ms, Some(1234));
    }

    #[test]
    fn test_parse_gemini_usage_extracts_session_id() {
        let output = r#"{"type":"init","session_id":"abc123-def456-789"}
{"type":"message","role":"user","content":"Hello"}
{"type":"message","role":"assistant","content":"Hi there"}
{"type":"result","status":"success","stats":{"input_tokens":100,"output_tokens":50,"duration_ms":1234}}"#;

        let metrics = ScriptRunner::parse_gemini_usage(output);
        assert_eq!(metrics.session_id, Some("abc123-def456-789".to_string()));
        assert_eq!(metrics.tokens_in, Some(100));
    }

    #[test]
    fn test_parse_gemini_usage_no_init_message() {
        // When there's no init message, session_id should be None
        let output = r#"{"type":"message","role":"assistant","content":"Hi there"}
{"type":"result","status":"success","stats":{"input_tokens":100,"output_tokens":50}}"#;

        let metrics = ScriptRunner::parse_gemini_usage(output);
        assert_eq!(metrics.session_id, None);
    }

    #[test]
    fn test_parse_gemini_usage_with_uuid_format_session_id() {
        // Test with a realistic UUID format
        let output = r#"{"type":"init","session_id":"550e8400-e29b-41d4-a716-446655440000"}
{"type":"result","status":"success","stats":{"input_tokens":50,"output_tokens":25}}"#;

        let metrics = ScriptRunner::parse_gemini_usage(output);
        assert_eq!(
            metrics.session_id,
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn test_parse_gemini_usage_no_metadata() {
        let output = r#"{"type":"message","content":"Hello world"}"#;

        let metrics = ScriptRunner::parse_gemini_usage(output);
        assert_eq!(metrics.tokens_in, None);
        assert_eq!(metrics.tokens_out, None);
        assert_eq!(metrics.session_id, None);
    }

    #[test]
    fn test_parse_claude_usage() {
        let output = r#"{"type":"system","subtype":"init"}
{"type":"assistant","message":{"content":"Hello"}}
{"type":"result","subtype":"success","duration_ms":2940,"total_cost_usd":0.051,"usage":{"input_tokens":2,"cache_creation_input_tokens":6745,"cache_read_input_tokens":14269,"output_tokens":72}}"#;

        let metrics = ScriptRunner::parse_claude_usage(output);
        // input_tokens = 2 + 6745 + 14269 = 21016
        assert_eq!(metrics.tokens_in, Some(21016));
        assert_eq!(metrics.tokens_out, Some(72));
        assert_eq!(metrics.duration_ms, Some(2940));
        assert_eq!(metrics.cost_usd, Some(0.051));
    }

    #[test]
    fn test_parse_claude_usage_no_usage() {
        let output = r#"{"type":"assistant","message":{"content":"Hello"}}"#;

        let metrics = ScriptRunner::parse_claude_usage(output);
        assert_eq!(metrics.tokens_in, None);
        assert_eq!(metrics.tokens_out, None);
    }

    #[test]
    fn test_parse_codex_usage() {
        let output = r#"{"type":"thread.started","thread_id":"abc"}
{"type":"turn.started"}
{"type":"item.completed","item":{"type":"agent_message","text":"Hello!"}}
{"type":"turn.completed","usage":{"input_tokens":3448,"cached_input_tokens":1664,"output_tokens":8}}"#;

        let metrics = ScriptRunner::parse_codex_usage(output);
        // input_tokens = 3448 + 1664 = 5112
        assert_eq!(metrics.tokens_in, Some(5112));
        assert_eq!(metrics.tokens_out, Some(8));
    }

    #[test]
    fn test_parse_codex_usage_no_usage() {
        let output = r#"{"type":"thread.started","thread_id":"abc"}"#;

        let metrics = ScriptRunner::parse_codex_usage(output);
        assert_eq!(metrics.tokens_in, None);
        assert_eq!(metrics.tokens_out, None);
    }

    #[test]
    fn test_parse_empty_output() {
        let metrics = ScriptRunner::parse_gemini_usage("");
        assert_eq!(metrics.tokens_in, None);
        assert_eq!(metrics.tokens_out, None);

        let metrics = ScriptRunner::parse_claude_usage("");
        assert_eq!(metrics.tokens_in, None);
        assert_eq!(metrics.tokens_out, None);

        let metrics = ScriptRunner::parse_codex_usage("");
        assert_eq!(metrics.tokens_in, None);
        assert_eq!(metrics.tokens_out, None);
    }

    #[test]
    fn test_parse_malformed_json() {
        let malformed = "not json at all\n{broken:json}";

        let metrics = ScriptRunner::parse_gemini_usage(malformed);
        assert_eq!(metrics.tokens_in, None);
        assert_eq!(metrics.tokens_out, None);
    }

    #[test]
    fn test_usage_metrics_default() {
        let metrics = UsageMetrics::default();
        assert_eq!(metrics.tokens_in, None);
        assert_eq!(metrics.tokens_out, None);
        assert_eq!(metrics.duration_ms, None);
        assert_eq!(metrics.cost_usd, None);
    }
}
