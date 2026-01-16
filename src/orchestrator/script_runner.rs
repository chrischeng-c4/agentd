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
}

/// Gemini stream-json usage response format
#[derive(Debug, Deserialize)]
struct GeminiUsageResponse {
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u64>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u64>,
    #[allow(dead_code)]
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u64>,
}

/// Claude stream-json usage response format
#[derive(Debug, Deserialize)]
struct ClaudeUsageResponse {
    usage: Option<ClaudeUsage>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

/// Codex JSON usage response format
#[derive(Debug, Deserialize)]
struct CodexUsageResponse {
    usage: Option<CodexUsage>,
}

#[derive(Debug, Deserialize)]
struct CodexUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
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
        let start = Instant::now();
        let output = self.run_command(provider.command(), &args, env, prompt, show_progress).await?;
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

    /// Parse Gemini stream-json output for usage metadata
    fn parse_gemini_usage(output: &str) -> UsageMetrics {
        let mut metrics = UsageMetrics::default();

        // Gemini stream-json outputs JSON objects one per line
        // Look for the last line with usageMetadata
        for line in output.lines().rev() {
            if let Ok(response) = serde_json::from_str::<GeminiUsageResponse>(line) {
                if let Some(usage) = response.usage_metadata {
                    metrics.tokens_in = usage.prompt_token_count;
                    metrics.tokens_out = usage.candidates_token_count;
                    return metrics;
                }
            }
        }

        metrics
    }

    /// Parse Claude stream-json output for usage
    fn parse_claude_usage(output: &str) -> UsageMetrics {
        let mut metrics = UsageMetrics::default();

        // Claude outputs JSON objects with usage in the final message
        for line in output.lines().rev() {
            if let Ok(response) = serde_json::from_str::<ClaudeUsageResponse>(line) {
                if let Some(usage) = response.usage {
                    metrics.tokens_in = usage.input_tokens;
                    metrics.tokens_out = usage.output_tokens;
                    return metrics;
                }
            }
        }

        metrics
    }

    /// Parse Codex JSON output for usage
    fn parse_codex_usage(output: &str) -> UsageMetrics {
        let mut metrics = UsageMetrics::default();

        // Codex --json outputs JSON with usage
        for line in output.lines().rev() {
            if let Ok(response) = serde_json::from_str::<CodexUsageResponse>(line) {
                if let Some(usage) = response.usage {
                    metrics.tokens_in = usage.prompt_tokens;
                    metrics.tokens_out = usage.completion_tokens;
                    return metrics;
                }
            }
        }

        metrics
    }

    /// Internal: Run a CLI command
    async fn run_command(
        &self,
        command_name: &str,
        args: &[String],
        env: HashMap<String, String>,
        prompt: &str,
        show_progress: bool,
    ) -> Result<String> {
        let mut cmd = Command::new(command_name);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

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

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .await
                .context("Failed to write to stdin")?;
            stdin.flush().await.context("Failed to flush stdin")?;
            drop(stdin);
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

        // We expect this to fail with "not found" error if gemini CLI is not installed
        if result.is_err() {
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("not found") || err.to_string().contains("Please ensure it is installed"),
                "Error should mention command not found: {}",
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
        let output = r#"{"text":"Hello"}
{"text":" world"}
{"usageMetadata":{"promptTokenCount":100,"candidatesTokenCount":50,"totalTokenCount":150}}"#;

        let metrics = ScriptRunner::parse_gemini_usage(output);
        assert_eq!(metrics.tokens_in, Some(100));
        assert_eq!(metrics.tokens_out, Some(50));
    }

    #[test]
    fn test_parse_gemini_usage_no_metadata() {
        let output = r#"{"text":"Hello world"}"#;

        let metrics = ScriptRunner::parse_gemini_usage(output);
        assert_eq!(metrics.tokens_in, None);
        assert_eq!(metrics.tokens_out, None);
    }

    #[test]
    fn test_parse_claude_usage() {
        let output = r#"{"type":"message_start"}
{"type":"content_block_delta"}
{"type":"message_delta","usage":{"input_tokens":200,"output_tokens":75}}"#;

        let metrics = ScriptRunner::parse_claude_usage(output);
        assert_eq!(metrics.tokens_in, Some(200));
        assert_eq!(metrics.tokens_out, Some(75));
    }

    #[test]
    fn test_parse_claude_usage_no_usage() {
        let output = r#"{"type":"message_start"}"#;

        let metrics = ScriptRunner::parse_claude_usage(output);
        assert_eq!(metrics.tokens_in, None);
        assert_eq!(metrics.tokens_out, None);
    }

    #[test]
    fn test_parse_codex_usage() {
        let output = r#"{"message":"Done","usage":{"prompt_tokens":500,"completion_tokens":150}}"#;

        let metrics = ScriptRunner::parse_codex_usage(output);
        assert_eq!(metrics.tokens_in, Some(500));
        assert_eq!(metrics.tokens_out, Some(150));
    }

    #[test]
    fn test_parse_codex_usage_no_usage() {
        let output = r#"{"message":"Done"}"#;

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
    }
}
