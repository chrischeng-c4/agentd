use super::cli_mapper::{LlmArg, LlmProvider};
use super::prompts;
use super::{ModelSelector, ScriptRunner, UsageMetrics};
use crate::models::{AgentdConfig, Complexity};
use anyhow::Result;
use std::collections::HashMap;
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

    /// Build common Claude args
    fn build_args(&self, complexity: Complexity, resume: bool) -> Vec<String> {
        let model = self.model_selector.select_claude(complexity);
        let args = vec![
            LlmArg::Print,
            LlmArg::Model(model.to_cli_arg()),
            LlmArg::AllowedTools("Write,Edit,Read,Bash,Glob,Grep".to_string()),
            LlmArg::OutputFormat("stream-json".to_string()),
            LlmArg::Verbose,
        ];
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
        // First call in Impl stage, no resume
        let args = self.build_args(complexity, false);

        self.runner.run_llm(LlmProvider::Claude, args, env, &prompt, true).await
    }

    /// Run resolve (fix issues from review)
    pub async fn run_resolve(&self, change_id: &str, complexity: Complexity) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::claude_resolve_prompt(change_id);
        let env = HashMap::new();
        // Resume previous session (Impl stage)
        let args = self.build_args(complexity, true);

        self.runner.run_llm(LlmProvider::Claude, args, env, &prompt, true).await
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
