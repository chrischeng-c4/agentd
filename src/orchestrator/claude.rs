use super::cli_mapper::{LlmArg, LlmProvider};
use super::prompts;
use super::{ModelSelector, ScriptRunner, UsageMetrics};
use crate::models::{AgentdConfig, Complexity};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Claude orchestrator for implementation tasks
pub struct ClaudeOrchestrator<'a> {
    model_selector: ModelSelector<'a>,
    runner: ScriptRunner,
}

impl<'a> ClaudeOrchestrator<'a> {
    pub fn new(config: &'a AgentdConfig, _project_root: impl Into<PathBuf>) -> Self {
        let model_selector = ModelSelector::new(config);
        let runner = ScriptRunner::new();

        Self {
            model_selector,
            runner,
        }
    }

    /// Generate temporary MCP configuration file for Claude
    ///
    /// Creates a JSON file in /tmp with filtered MCP tools for the implement stage.
    /// If the file already exists, it will be reused.
    /// Returns the path to the temporary config file.
    fn generate_tmp_mcp_config(&self, change_id: &str, stage: &str) -> Result<PathBuf> {
        let tmp_config_path = PathBuf::from(format!(
            "/tmp/agentd-{}-{}.mcp.json",
            stage, change_id
        ));

        // Reuse existing config if present
        if tmp_config_path.exists() {
            eprintln!(
                "[agentd] Reusing existing MCP config: {}",
                tmp_config_path.display()
            );
            return Ok(tmp_config_path);
        }

        let config_json = json!({
            "mcpServers": {
                "agentd": {
                    "command": "agentd",
                    "args": ["mcp-server", "--tools", stage],
                    "env": {}
                }
            }
        });

        let config_str = serde_json::to_string_pretty(&config_json)?;
        fs::write(&tmp_config_path, config_str)?;

        eprintln!(
            "[agentd] Generated MCP config for {} stage: {}",
            stage,
            tmp_config_path.display()
        );

        Ok(tmp_config_path)
    }

    /// Clean up temporary MCP configuration file
    ///
    /// Removes the temporary config file created for this change.
    /// This is optional since /tmp is automatically cleaned by the OS.
    pub fn cleanup_tmp_mcp_config(change_id: &str, stage: &str) -> Result<()> {
        let tmp_config_path = PathBuf::from(format!(
            "/tmp/agentd-{}-{}.mcp.json",
            stage, change_id
        ));

        if tmp_config_path.exists() {
            fs::remove_file(&tmp_config_path)?;
            eprintln!(
                "[agentd] Cleaned up MCP config: {}",
                tmp_config_path.display()
            );
        }

        Ok(())
    }

    /// Build Claude args with optional MCP config
    fn build_args_with_mcp(
        &self,
        complexity: Complexity,
        resume: bool,
        mcp_config: Option<&PathBuf>,
    ) -> Vec<String> {
        let model = self.model_selector.select_claude(complexity);
        let mut args = vec![
            LlmArg::Print,
            LlmArg::Model(model.to_cli_arg()),
            LlmArg::AllowedTools("Write,Edit,Read,Bash,Glob,Grep".to_string()),
            LlmArg::OutputFormat("stream-json".to_string()),
            LlmArg::Verbose,
        ];

        // Add MCP config if provided
        if let Some(config_path) = mcp_config {
            args.push(LlmArg::McpConfig(config_path.display().to_string()));
        }

        LlmProvider::Claude.build_args(&args, resume)
    }

    /// Run implementation task
    pub async fn run_implement(
        &self,
        change_id: &str,
        tasks: Option<&str>,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::claude_implement_prompt(change_id, tasks);
        let env = HashMap::new();

        // Generate temporary MCP config for implement stage (4 tools)
        let mcp_config = self.generate_tmp_mcp_config(change_id, "implement")?;

        // First call in Impl stage, no resume
        let args = self.build_args_with_mcp(complexity, false, Some(&mcp_config));

        self.runner
            .run_llm(LlmProvider::Claude, args, env, &prompt, true)
            .await
    }

    /// Run resolve (fix issues from review)
    pub async fn run_resolve(&self, change_id: &str, complexity: Complexity) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::claude_resolve_prompt(change_id);
        let env = HashMap::new();

        // Use same MCP config as implement stage
        let mcp_config = self.generate_tmp_mcp_config(change_id, "implement")?;

        // Resume previous session (Impl stage)
        let args = self.build_args_with_mcp(complexity, true, Some(&mcp_config));

        self.runner
            .run_llm(LlmProvider::Claude, args, env, &prompt, true)
            .await
    }

    /// Run spec-level implementation (new)
    pub async fn run_implement_spec(
        &self,
        change_id: &str,
        spec_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::claude_implement_spec_prompt(change_id, spec_id);
        let env = HashMap::new();

        // Use MCP config for implement stage
        let mcp_config = self.generate_tmp_mcp_config(change_id, "implement")?;

        // Resume session if available, else start fresh
        let args = self.build_args_with_mcp(complexity, true, Some(&mcp_config));

        self.runner
            .run_llm(LlmProvider::Claude, args, env, &prompt, true)
            .await
    }

    /// Run self-review for spec (new)
    pub async fn run_self_review_spec(
        &self,
        change_id: &str,
        spec_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::claude_self_review_spec_prompt(change_id, spec_id);
        let env = HashMap::new();

        // Use MCP config for implement stage
        let mcp_config = self.generate_tmp_mcp_config(change_id, "implement")?;

        // Resume same session for self-review
        let args = self.build_args_with_mcp(complexity, true, Some(&mcp_config));

        self.runner
            .run_llm(LlmProvider::Claude, args, env, &prompt, true)
            .await
    }

    /// Run spec-level fix (resolve issues from self-review)
    pub async fn run_resolve_spec(
        &self,
        change_id: &str,
        spec_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::claude_resolve_spec_prompt(change_id, spec_id);
        let env = HashMap::new();

        // Use MCP config for implement stage
        let mcp_config = self.generate_tmp_mcp_config(change_id, "implement")?;

        // Resume session to fix issues
        let args = self.build_args_with_mcp(complexity, true, Some(&mcp_config));

        self.runner
            .run_llm(LlmProvider::Claude, args, env, &prompt, true)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_orchestrator_creation() {
        let config = AgentdConfig::default();
        let _orchestrator = ClaudeOrchestrator::new(&config, "/tmp/test");
        // Orchestrator created successfully
    }
}
