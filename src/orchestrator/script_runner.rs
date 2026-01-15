use super::SelectedModel;
use anyhow::{Context, Result};
use indicatif::{ProgressBar as IndicatifProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Environment variables for model selection
/// Scripts can read these to use the appropriate model
const ENV_AGENTD_MODEL: &str = "AGENTD_MODEL";
const ENV_AGENTD_MODEL_COMMAND: &str = "AGENTD_MODEL_COMMAND";
const ENV_AGENTD_MODEL_REASONING: &str = "AGENTD_MODEL_REASONING";

/// Runner for executing AI integration scripts
pub struct ScriptRunner {
    scripts_dir: PathBuf,
}

impl ScriptRunner {
    pub fn new(scripts_dir: impl Into<PathBuf>) -> Self {
        Self {
            scripts_dir: scripts_dir.into(),
        }
    }

    /// Run a script and stream output with progress bar
    pub async fn run_script(
        &self,
        script_name: &str,
        args: &[String],
        show_progress: bool,
    ) -> Result<String> {
        self.run_script_with_model(script_name, args, show_progress, None)
            .await
    }

    /// Run a script with model selection via environment variables
    pub async fn run_script_with_model(
        &self,
        script_name: &str,
        args: &[String],
        show_progress: bool,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        let script_path = self.scripts_dir.join(script_name);

        if !script_path.exists() {
            anyhow::bail!("Script not found: {:?}", script_path);
        }

        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }

        let mut cmd = Command::new(&script_path);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        // Set model environment variables if provided
        if let Some(selected) = model {
            cmd.env(ENV_AGENTD_MODEL, selected.to_cli_arg());
            cmd.env(ENV_AGENTD_MODEL_COMMAND, selected.command());

            // For Codex, also set the reasoning level separately
            if let SelectedModel::Codex { reasoning, .. } = selected {
                if let Some(level) = reasoning {
                    cmd.env(ENV_AGENTD_MODEL_REASONING, level);
                }
            }
        }

        let progress = if show_progress {
            let pb = IndicatifProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .unwrap()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
            );
            pb.set_message(format!("Running {}...", script_name));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };

        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn script: {:?}", script_path))?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let mut reader = BufReader::new(stdout).lines();

        let mut output = String::new();
        while let Ok(Some(line)) = reader.next_line().await {
            output.push_str(&line);
            output.push('\n');

            if let Some(ref pb) = progress {
                // Update progress message with recent output
                let short_line = if line.len() > 60 {
                    format!("{}...", &line[..60])
                } else {
                    line.clone()
                };
                pb.set_message(short_line);
            }
        }

        let status = child.wait().await?;

        if let Some(pb) = progress {
            pb.finish_and_clear();
        }

        if !status.success() {
            anyhow::bail!("Script failed with exit code: {:?}", status.code());
        }

        Ok(output)
    }

    // =========================================================================
    // Gemini Methods
    // =========================================================================

    /// Run Gemini proposal generation
    pub async fn run_gemini_proposal(&self, change_id: &str, description: &str) -> Result<String> {
        self.run_gemini_proposal_with_model(change_id, description, None)
            .await
    }

    /// Run Gemini proposal generation with model selection
    pub async fn run_gemini_proposal_with_model(
        &self,
        change_id: &str,
        description: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model(
            "gemini-proposal.sh",
            &[change_id.to_string(), description.to_string()],
            true,
            model,
        )
        .await
    }

    /// Run Gemini reproposal (resumes previous session for cached context)
    pub async fn run_gemini_reproposal(&self, change_id: &str) -> Result<String> {
        self.run_gemini_reproposal_with_model(change_id, None).await
    }

    /// Run Gemini reproposal with model selection
    pub async fn run_gemini_reproposal_with_model(
        &self,
        change_id: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model(
            "gemini-reproposal.sh",
            &[change_id.to_string()],
            true,
            model,
        )
        .await
    }

    /// Run Gemini spec merging (merge delta specs back to main specs)
    pub async fn run_gemini_merge_specs(
        &self,
        change_id: &str,
        strategy: &str,
        spec_file: &str,
    ) -> Result<String> {
        self.run_gemini_merge_specs_with_model(change_id, strategy, spec_file, None)
            .await
    }

    /// Run Gemini spec merging with model selection
    pub async fn run_gemini_merge_specs_with_model(
        &self,
        change_id: &str,
        strategy: &str,
        spec_file: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model(
            "gemini-merge-specs.sh",
            &[
                change_id.to_string(),
                strategy.to_string(),
                spec_file.to_string(),
            ],
            true,
            model,
        )
        .await
    }

    /// Run Gemini CHANGELOG generation
    pub async fn run_gemini_changelog(&self, change_id: &str) -> Result<String> {
        self.run_gemini_changelog_with_model(change_id, None).await
    }

    /// Run Gemini CHANGELOG generation with model selection
    pub async fn run_gemini_changelog_with_model(
        &self,
        change_id: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model(
            "gemini-changelog.sh",
            &[change_id.to_string()],
            true,
            model,
        )
        .await
    }

    // =========================================================================
    // Codex Methods
    // =========================================================================

    /// Run Codex challenge (creates new session)
    pub async fn run_codex_challenge(&self, change_id: &str) -> Result<String> {
        self.run_codex_challenge_with_model(change_id, None).await
    }

    /// Run Codex challenge with model selection
    pub async fn run_codex_challenge_with_model(
        &self,
        change_id: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model(
            "codex-challenge.sh",
            &[change_id.to_string()],
            true,
            model,
        )
        .await
    }

    /// Run Codex re-challenge (resumes previous session for cached context)
    pub async fn run_codex_rechallenge(&self, change_id: &str) -> Result<String> {
        self.run_codex_rechallenge_with_model(change_id, None).await
    }

    /// Run Codex re-challenge with model selection
    pub async fn run_codex_rechallenge_with_model(
        &self,
        change_id: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model(
            "codex-rechallenge.sh",
            &[change_id.to_string()],
            true,
            model,
        )
        .await
    }

    /// Run Codex code review with iteration tracking
    pub async fn run_codex_review(&self, change_id: &str, iteration: u32) -> Result<String> {
        self.run_codex_review_with_model(change_id, iteration, None)
            .await
    }

    /// Run Codex code review with model selection
    pub async fn run_codex_review_with_model(
        &self,
        change_id: &str,
        iteration: u32,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model(
            "codex-review.sh",
            &[change_id.to_string(), iteration.to_string()],
            true,
            model,
        )
        .await
    }

    /// Run Codex verification
    pub async fn run_codex_verify(&self, change_id: &str) -> Result<String> {
        self.run_codex_verify_with_model(change_id, None).await
    }

    /// Run Codex verification with model selection
    pub async fn run_codex_verify_with_model(
        &self,
        change_id: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model("codex-verify.sh", &[change_id.to_string()], true, model)
            .await
    }

    /// Run Codex archive quality review
    pub async fn run_codex_archive_review(
        &self,
        change_id: &str,
        strategy: &str,
    ) -> Result<String> {
        self.run_codex_archive_review_with_model(change_id, strategy, None)
            .await
    }

    /// Run Codex archive quality review with model selection
    pub async fn run_codex_archive_review_with_model(
        &self,
        change_id: &str,
        strategy: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model(
            "codex-archive-review.sh",
            &[change_id.to_string(), strategy.to_string()],
            true,
            model,
        )
        .await
    }

    // =========================================================================
    // Claude Methods
    // =========================================================================

    /// Run Claude implementation
    pub async fn run_claude_implement(
        &self,
        change_id: &str,
        tasks: Option<&str>,
    ) -> Result<String> {
        self.run_claude_implement_with_model(change_id, tasks, None)
            .await
    }

    /// Run Claude implementation with model selection
    pub async fn run_claude_implement_with_model(
        &self,
        change_id: &str,
        tasks: Option<&str>,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        let args = if let Some(t) = tasks {
            vec![change_id.to_string(), "--tasks".to_string(), t.to_string()]
        } else {
            vec![change_id.to_string()]
        };

        self.run_script_with_model("claude-implement.sh", &args, true, model)
            .await
    }

    /// Run Claude resolve (fix issues from review)
    pub async fn run_claude_resolve(&self, change_id: &str) -> Result<String> {
        self.run_claude_resolve_with_model(change_id, None).await
    }

    /// Run Claude resolve with model selection
    pub async fn run_claude_resolve_with_model(
        &self,
        change_id: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model("claude-resolve.sh", &[change_id.to_string()], true, model)
            .await
    }

    /// Run Claude fix (fix issues from verification)
    pub async fn run_claude_fix(&self, change_id: &str) -> Result<String> {
        self.run_claude_fix_with_model(change_id, None).await
    }

    /// Run Claude fix with model selection
    pub async fn run_claude_fix_with_model(
        &self,
        change_id: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model("claude-fix.sh", &[change_id.to_string()], true, model)
            .await
    }

    /// Run Claude archive fix (fix issues from archive review)
    pub async fn run_claude_archive_fix(&self, change_id: &str) -> Result<String> {
        self.run_claude_archive_fix_with_model(change_id, None).await
    }

    /// Run Claude archive fix with model selection
    pub async fn run_claude_archive_fix_with_model(
        &self,
        change_id: &str,
        model: Option<&SelectedModel>,
    ) -> Result<String> {
        self.run_script_with_model("claude-archive-fix.sh", &[change_id.to_string()], true, model)
            .await
    }
}
