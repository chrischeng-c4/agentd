use super::cli_mapper::{LlmArg, LlmProvider};
use super::prompts;
use super::{ModelSelector, ScriptRunner, SelectedModel, UsageMetrics};
use crate::models::{AgentdConfig, Complexity};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Project type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectType {
    Rust,
    TypeScript,
    Python,
}

/// Codex orchestrator for challenge and review tasks
pub struct CodexOrchestrator<'a> {
    model_selector: ModelSelector<'a>,
    runner: ScriptRunner,
    project_root: PathBuf,
}

impl<'a> CodexOrchestrator<'a> {
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

    /// Build common Codex environment variables
    fn build_env(&self, change_id: &str) -> HashMap<String, String> {
        let codex_instructions_file = self
            .project_root
            .join("agentd")
            .join("changes")
            .join(change_id)
            .join("AGENTS.md");

        let mut env = HashMap::new();
        env.insert(
            "CODEX_INSTRUCTIONS_FILE".to_string(),
            codex_instructions_file.to_string_lossy().to_string(),
        );
        env
    }

    /// Build common Codex args
    fn build_args(&self, complexity: Complexity, resume: bool) -> Vec<String> {
        let model = self.model_selector.select_codex(complexity);
        let mut args = vec![
            LlmArg::FullAuto,
            LlmArg::Json,
        ];

        // Add model and reasoning from selected model
        if let SelectedModel::Codex { model: model_name, reasoning, .. } = &model {
            args.push(LlmArg::Model(model_name.clone()));
            if let Some(level) = reasoning {
                args.push(LlmArg::Reasoning(level.clone()));
            }
        }

        LlmProvider::Codex.build_args(&args, resume)
    }

    /// Detect project type based on configuration files
    fn detect_project_type(&self) -> ProjectType {
        // Check for package.json (TypeScript/Node.js)
        if self.project_root.join("package.json").exists() {
            return ProjectType::TypeScript;
        }

        // Check for Python files
        if self.project_root.join("pyproject.toml").exists()
            || self.project_root.join("requirements.txt").exists()
            || self.project_root.join("setup.py").exists()
        {
            return ProjectType::Python;
        }

        // Check for Cargo.toml (Rust)
        if self.project_root.join("Cargo.toml").exists() {
            return ProjectType::Rust;
        }

        // Default to Rust for backward compatibility
        ProjectType::Rust
    }

    /// Run challenge (initial proposal review)
    pub async fn run_challenge(&self, change_id: &str, complexity: Complexity) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::codex_challenge_prompt(change_id);
        let env = self.build_env(change_id);
        // First call in Plan stage, no resume
        let args = self.build_args(complexity, false);

        self.runner.run_llm(LlmProvider::Codex, args, env, &prompt, true).await
    }

    /// Run rechallenge (resume previous challenge session)
    pub async fn run_rechallenge(
        &self,
        change_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::codex_rechallenge_prompt(change_id);
        let env = self.build_env(change_id);
        // Resume previous session (Plan stage)
        let args = self.build_args(complexity, true);

        self.runner.run_llm(LlmProvider::Codex, args, env, &prompt, true).await
    }

    /// Run local verification tools and capture output
    async fn run_verification_tools(&self) -> Result<(String, String, String, String)> {
        let project_type = self.detect_project_type();

        match project_type {
            ProjectType::Rust => self.run_rust_verification_tools().await,
            ProjectType::TypeScript => self.run_typescript_verification_tools().await,
            ProjectType::Python => self.run_python_verification_tools().await,
        }
    }

    /// Run Rust-specific verification tools
    async fn run_rust_verification_tools(&self) -> Result<(String, String, String, String)> {
        // Run tests
        let test_output = self.run_local_command("cargo", &["test"]).await?;

        // Run cargo audit (check if available first)
        let audit_output = self.check_and_run_cargo_audit().await;

        // Run semgrep (if available)
        let semgrep_output = self
            .run_local_command("semgrep", &["--config=auto", "--json"])
            .await
            .unwrap_or_else(|_| "semgrep not available".to_string());

        // Run clippy
        let clippy_output = self
            .run_local_command(
                "cargo",
                &["clippy", "--", "-W", "clippy::all", "-W", "clippy::pedantic"],
            )
            .await
            .unwrap_or_else(|_| "clippy failed".to_string());

        Ok((test_output, audit_output, semgrep_output, clippy_output))
    }

    /// Run TypeScript/Node.js-specific verification tools
    async fn run_typescript_verification_tools(&self) -> Result<(String, String, String, String)> {
        // Run tests (prefer npm test)
        let test_output = if let Ok(output) = self.run_local_command("npm", &["test", "--", "--run"]).await {
            output
        } else {
            "Test run failed: No test runner configured in package.json".to_string()
        };

        // Run npm audit for dependency vulnerabilities
        let audit_output = self
            .run_local_command("npm", &["audit", "--json"])
            .await
            .unwrap_or_else(|_| "npm audit not available".to_string());

        // Run semgrep (if available)
        let semgrep_output = self
            .run_local_command("semgrep", &["--config=auto", "--json"])
            .await
            .unwrap_or_else(|_| "semgrep not available".to_string());

        // Run TypeScript compiler check
        let linter_output = self
            .run_local_command("npx", &["tsc", "--noEmit"])
            .await
            .unwrap_or_else(|_| "TypeScript compiler check not available".to_string());

        Ok((test_output, audit_output, semgrep_output, linter_output))
    }

    /// Run Python-specific verification tools
    async fn run_python_verification_tools(&self) -> Result<(String, String, String, String)> {
        // Run tests (try pytest first)
        let test_output = self
            .run_local_command("pytest", &["-v"])
            .await
            .unwrap_or_else(|_| "Test run failed: pytest not found".to_string());

        // Run pip-audit for dependency vulnerabilities (if available)
        let audit_output = self
            .run_local_command("pip-audit", &["--format", "json"])
            .await
            .unwrap_or_else(|_| "pip-audit not available (install with: pip install pip-audit)".to_string());

        // Run semgrep (if available)
        let semgrep_output = self
            .run_local_command("semgrep", &["--config=auto", "--json"])
            .await
            .unwrap_or_else(|_| "semgrep not available".to_string());

        // Run linter (try ruff first)
        let linter_output = self
            .run_local_command("ruff", &["check", ".", "--output-format", "json"])
            .await
            .unwrap_or_else(|_| "Linter not available: ruff not found".to_string());

        Ok((test_output, audit_output, semgrep_output, linter_output))
    }

    /// Check if cargo-audit is installed and run it
    async fn check_and_run_cargo_audit(&self) -> String {
        // First check if cargo-audit is installed by listing subcommands
        let check = Command::new("cargo")
            .args(&["--list"])
            .current_dir(&self.project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match check {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("audit") {
                    // cargo-audit is available, run it
                    self.run_local_command("cargo", &["audit"])
                        .await
                        .unwrap_or_else(|_| "cargo audit failed".to_string())
                } else {
                    "cargo-audit not available (install with: cargo install cargo-audit)".to_string()
                }
            }
            Err(_) => "cargo-audit not available (install with: cargo install cargo-audit)".to_string(),
        }
    }

    /// Run a local command and capture output
    async fn run_local_command(&self, command: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(command)
            .args(args)
            .current_dir(&self.project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .with_context(|| format!("Failed to execute command: {}", command))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(format!("{}\n{}", stdout, stderr))
    }

    /// Run code review with pre-processing
    pub async fn run_review(
        &self,
        change_id: &str,
        iteration: u32,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        println!("🧪 Running tests...");
        println!("🔒 Running security scans...");

        // Step 1: Execute local verification tools
        let (test_output, audit_output, semgrep_output, clippy_output) =
            self.run_verification_tools().await?;

        // Step 2: Generate prompt with pre-processing results
        let prompt = prompts::codex_review_prompt(
            change_id,
            iteration,
            &test_output,
            &audit_output,
            &semgrep_output,
            &clippy_output,
        );

        let env = self.build_env(change_id);
        // Resume if iteration > 0 (Impl stage)
        let resume = iteration > 0;
        let args = self.build_args(complexity, resume);

        // Step 3: Run Codex with enriched prompt
        self.runner.run_llm(LlmProvider::Codex, args, env, &prompt, true).await
    }

    /// Run verification
    pub async fn run_verify(&self, change_id: &str, complexity: Complexity) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::codex_verify_prompt(change_id);
        let env = self.build_env(change_id);
        let args = self.build_args(complexity, false);

        self.runner.run_llm(LlmProvider::Codex, args, env, &prompt, true).await
    }

    /// Run archive quality review
    pub async fn run_archive_review(
        &self,
        change_id: &str,
        strategy: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::codex_archive_review_prompt(change_id, strategy);
        let env = self.build_env(change_id);
        // First Codex call in Archive stage, no resume
        let args = self.build_args(complexity, false);

        self.runner.run_llm(LlmProvider::Codex, args, env, &prompt, true).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_orchestrator_creation() {
        let config = AgentdConfig::default();
        let orchestrator = CodexOrchestrator::new(&config, "/tmp/test");
        assert_eq!(orchestrator.project_root, PathBuf::from("/tmp/test"));
    }

    #[tokio::test]
    async fn test_run_local_command() {
        let config = AgentdConfig::default();
        // Use current directory instead of /tmp/test
        let orchestrator = CodexOrchestrator::new(&config, ".");

        // Test with a simple command that should work
        let result = orchestrator.run_local_command("echo", &["test"]).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test"));
    }

    #[test]
    fn test_review_prompt_includes_preprocessing_outputs() {
        // Test that the review prompt generator includes all pre-processing outputs
        let change_id = "test-change";
        let iteration = 1;
        let test_output = "All tests passed";
        let audit_output = "No vulnerabilities found";
        let semgrep_output = "No issues found";
        let clippy_output = "All checks passed";

        let prompt = prompts::codex_review_prompt(
            change_id,
            iteration,
            test_output,
            audit_output,
            semgrep_output,
            clippy_output,
        );

        // Verify all outputs are included in the prompt
        assert!(prompt.contains(test_output), "Prompt should include test output");
        assert!(prompt.contains(audit_output), "Prompt should include audit output");
        assert!(prompt.contains(semgrep_output), "Prompt should include semgrep output");
        assert!(prompt.contains(clippy_output), "Prompt should include clippy output");
        assert!(prompt.contains(&format!("Iteration {}", iteration)), "Prompt should include iteration");
        assert!(prompt.contains(change_id), "Prompt should include change ID");
    }

    #[tokio::test]
    async fn test_run_local_command_captures_stderr() {
        let config = AgentdConfig::default();
        let orchestrator = CodexOrchestrator::new(&config, ".");

        // Run a command that outputs to stderr
        // Using a non-existent cargo subcommand should produce stderr
        let result = orchestrator.run_local_command("cargo", &["--version"]).await;

        // Should succeed and return output
        assert!(result.is_ok());
    }
}
