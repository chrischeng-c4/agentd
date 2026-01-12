use anyhow::{Context, Result};
use indicatif::{ProgressBar as IndicatifProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

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

    /// Run Gemini proposal generation
    pub async fn run_gemini_proposal(&self, change_id: &str, description: &str) -> Result<String> {
        self.run_script(
            "gemini-proposal.sh",
            &[change_id.to_string(), description.to_string()],
            true,
        )
        .await
    }

    /// Run Codex challenge
    pub async fn run_codex_challenge(&self, change_id: &str) -> Result<String> {
        self.run_script("codex-challenge.sh", &[change_id.to_string()], true)
            .await
    }

    /// Run Gemini reproposal
    pub async fn run_gemini_reproposal(&self, change_id: &str) -> Result<String> {
        self.run_script("gemini-reproposal.sh", &[change_id.to_string()], true)
            .await
    }

    /// Run Claude implementation
    pub async fn run_claude_implement(
        &self,
        change_id: &str,
        tasks: Option<&str>,
    ) -> Result<String> {
        let args = if let Some(t) = tasks {
            vec![change_id.to_string(), "--tasks".to_string(), t.to_string()]
        } else {
            vec![change_id.to_string()]
        };

        self.run_script("claude-implement.sh", &args, true).await
    }

    /// Run Codex verification
    pub async fn run_codex_verify(&self, change_id: &str) -> Result<String> {
        self.run_script("codex-verify.sh", &[change_id.to_string()], true)
            .await
    }
}
