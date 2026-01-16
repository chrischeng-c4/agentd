use super::cli_mapper::{LlmArg, LlmProvider};
use super::prompts;
use super::{ModelSelector, ScriptRunner};
use crate::models::{AgentdConfig, Complexity};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Gemini orchestrator for proposal and documentation tasks
pub struct GeminiOrchestrator<'a> {
    model_selector: ModelSelector<'a>,
    runner: ScriptRunner,
    project_root: PathBuf,
}

impl<'a> GeminiOrchestrator<'a> {
    pub fn new(config: &'a AgentdConfig, project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        let model_selector = ModelSelector::new(config);
        let runner = ScriptRunner::new();

        Self {
            model_selector,
            runner,
            project_root,
        }
    }

    /// Build common Gemini environment variables
    fn build_env(&self, change_id: &str) -> HashMap<String, String> {
        let gemini_system_md = self
            .project_root
            .join("agentd")
            .join("changes")
            .join(change_id)
            .join("GEMINI.md");

        let mut env = HashMap::new();
        env.insert(
            "GEMINI_SYSTEM_MD".to_string(),
            gemini_system_md.to_string_lossy().to_string(),
        );
        env
    }

    /// Build common Gemini args
    fn build_args(&self, task: &str, complexity: Complexity, resume: bool) -> Vec<String> {
        let model = self.model_selector.select_gemini(complexity);
        let args = vec![
            LlmArg::Task(task.to_string()),
            LlmArg::Model(model.to_cli_arg()),
            LlmArg::OutputFormat("stream-json".to_string()),
        ];
        LlmProvider::Gemini.build_args(&args, resume)
    }

    /// Run proposal generation
    pub async fn run_proposal(
        &self,
        change_id: &str,
        description: &str,
        complexity: Complexity,
    ) -> Result<String> {
        let prompt = prompts::gemini_proposal_prompt(change_id, description);
        let env = self.build_env(change_id);
        let args = self.build_args("agentd:proposal", complexity, false);

        self.runner.run_llm(LlmProvider::Gemini, args, env, &prompt, true).await
    }

    /// Run reproposal (resume previous session for cached context)
    pub async fn run_reproposal(
        &self,
        change_id: &str,
        complexity: Complexity,
    ) -> Result<String> {
        let prompt = prompts::gemini_reproposal_prompt(change_id);
        let env = self.build_env(change_id);
        // Resume previous session (Plan stage)
        let args = self.build_args("agentd:reproposal", complexity, true);

        self.runner.run_llm(LlmProvider::Gemini, args, env, &prompt, true).await
    }

    /// Run spec merging (merge delta specs back to main specs)
    pub async fn run_merge_specs(
        &self,
        change_id: &str,
        strategy: &str,
        spec_file: &str,
        complexity: Complexity,
    ) -> Result<String> {
        let prompt = prompts::gemini_merge_specs_prompt(change_id, strategy, spec_file);
        let env = self.build_env(change_id);
        let args = self.build_args("agentd:merge-specs", complexity, false);

        self.runner.run_llm(LlmProvider::Gemini, args, env, &prompt, true).await
    }

    /// Run changelog generation
    pub async fn run_changelog(&self, change_id: &str, complexity: Complexity) -> Result<String> {
        let prompt = prompts::gemini_changelog_prompt(change_id);
        let env = self.build_env(change_id);
        // First call in Archive stage, no resume
        let args = self.build_args("agentd:changelog", complexity, false);

        self.runner.run_llm(LlmProvider::Gemini, args, env, &prompt, true).await
    }

    /// Run fillback (fill placeholders in files)
    pub async fn run_fillback(
        &self,
        change_id: &str,
        file_path: &str,
        placeholder: &str,
        complexity: Complexity,
    ) -> Result<String> {
        let prompt = prompts::gemini_fillback_prompt(change_id, file_path, placeholder);
        let env = self.build_env(change_id);
        let args = self.build_args("agentd:fillback", complexity, false);

        self.runner.run_llm(LlmProvider::Gemini, args, env, &prompt, true).await
    }

    /// Run archive fix (fix issues from archive review)
    pub async fn run_archive_fix(
        &self,
        change_id: &str,
        complexity: Complexity,
    ) -> Result<String> {
        let prompt = prompts::gemini_archive_fix_prompt(change_id);
        let env = self.build_env(change_id);
        // Resume previous session (Archive stage)
        let args = self.build_args("agentd:archive-fix", complexity, true);

        self.runner.run_llm(LlmProvider::Gemini, args, env, &prompt, true).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_orchestrator_creation() {
        let config = AgentdConfig::default();
        let orchestrator = GeminiOrchestrator::new(&config, "/tmp/test");
        assert_eq!(orchestrator.project_root, PathBuf::from("/tmp/test"));
    }
}
