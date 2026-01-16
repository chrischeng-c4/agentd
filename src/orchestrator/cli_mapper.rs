//! Agent-agnostic CLI argument mapping
//!
//! Maps common LLM arguments to specific CLI syntax for each tool.

/// Common LLM CLI arguments
#[derive(Debug, Clone)]
pub enum LlmArg {
    /// Resume previous session
    Resume,
    /// Model name/ID
    Model(String),
    /// Reasoning level (Codex-specific, but abstracted)
    Reasoning(String),
    /// JSON output format
    Json,
    /// Full auto mode (no prompts)
    FullAuto,
    /// Output format (stream-json, etc.)
    OutputFormat(String),
    /// Task/prompt name (first positional arg)
    Task(String),
    /// Print mode (Claude -p flag for non-interactive)
    Print,
    /// Allowed tools (Claude --allowedTools)
    AllowedTools(String),
    /// Verbose output
    Verbose,
}

/// LLM provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    Gemini,
    Codex,
    Claude,
}

impl LlmProvider {
    /// Get the CLI command name
    pub fn command(&self) -> &'static str {
        match self {
            LlmProvider::Gemini => "gemini",
            LlmProvider::Codex => "codex",
            LlmProvider::Claude => "claude",
        }
    }

    /// Build CLI arguments from common LlmArgs
    ///
    /// # Arguments
    /// * `args` - Common arguments to map
    /// * `resume` - Whether this is a resume/continue call
    ///
    /// # Returns
    /// Vec of CLI argument strings
    pub fn build_args(&self, args: &[LlmArg], resume: bool) -> Vec<String> {
        let mut cli_args = Vec::new();

        // Handle resume/exec mode first (affects base command for Codex)
        match self {
            LlmProvider::Gemini => {
                // Gemini: task name first, then --resume latest if resuming
                for arg in args {
                    if let LlmArg::Task(task) = arg {
                        cli_args.push(task.clone());
                        break;
                    }
                }
                if resume {
                    cli_args.push("--resume".to_string());
                    cli_args.push("latest".to_string());
                }
            }
            LlmProvider::Codex => {
                // Codex: "resume --last" or "exec" as first args
                if resume {
                    cli_args.push("resume".to_string());
                    cli_args.push("--last".to_string());
                } else {
                    cli_args.push("exec".to_string());
                }
            }
            LlmProvider::Claude => {
                // Claude: --continue if resuming
                if resume {
                    cli_args.push("--continue".to_string());
                }
            }
        }

        // Map remaining common args
        for arg in args {
            match arg {
                LlmArg::Model(model) => {
                    // Codex resume inherits model from session, skip --model
                    if *self == LlmProvider::Codex && resume {
                        continue;
                    }
                    match self {
                        LlmProvider::Gemini => {
                            cli_args.push("-m".to_string());
                            cli_args.push(model.clone());
                        }
                        LlmProvider::Codex | LlmProvider::Claude => {
                            cli_args.push("--model".to_string());
                            cli_args.push(model.clone());
                        }
                    }
                }
                LlmArg::Reasoning(level) => {
                    // Codex resume inherits reasoning from session, skip --config
                    if *self == LlmProvider::Codex && resume {
                        continue;
                    }
                    // Only Codex supports reasoning levels (via --config)
                    if *self == LlmProvider::Codex {
                        cli_args.push("--config".to_string());
                        cli_args.push(format!("reasoning={}", level));
                    }
                }
                LlmArg::Json => {
                    // Codex resume doesn't support --json flag
                    if *self == LlmProvider::Codex && resume {
                        continue;
                    }
                    match self {
                        LlmProvider::Codex => {
                            cli_args.push("--json".to_string());
                        }
                        LlmProvider::Claude => {
                            cli_args.push("--output-format".to_string());
                            cli_args.push("json".to_string());
                        }
                        LlmProvider::Gemini => {
                            // Gemini uses --output-format stream-json
                        }
                    }
                }
                LlmArg::FullAuto => {
                    match self {
                        LlmProvider::Codex => {
                            cli_args.push("--full-auto".to_string());
                        }
                        LlmProvider::Claude => {
                            cli_args.push("--dangerously-skip-permissions".to_string());
                        }
                        LlmProvider::Gemini => {
                            // Gemini doesn't have full-auto equivalent
                        }
                    }
                }
                LlmArg::OutputFormat(format) => {
                    match self {
                        LlmProvider::Gemini => {
                            cli_args.push("--output-format".to_string());
                            cli_args.push(format.clone());
                        }
                        LlmProvider::Claude => {
                            cli_args.push("--output-format".to_string());
                            cli_args.push(format.clone());
                        }
                        LlmProvider::Codex => {
                            // Codex uses --json instead
                        }
                    }
                }
                LlmArg::Task(_) => {
                    // Already handled above for Gemini
                }
                LlmArg::Resume => {
                    // Handled by the resume flag above
                }
                LlmArg::Print => {
                    // Claude -p for print/non-interactive mode
                    if *self == LlmProvider::Claude {
                        cli_args.push("-p".to_string());
                    }
                }
                LlmArg::AllowedTools(tools) => {
                    // Claude --allowedTools
                    if *self == LlmProvider::Claude {
                        cli_args.push("--allowedTools".to_string());
                        cli_args.push(tools.clone());
                    }
                }
                LlmArg::Verbose => {
                    // Claude --verbose
                    if *self == LlmProvider::Claude {
                        cli_args.push("--verbose".to_string());
                    }
                }
            }
        }

        cli_args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_args_no_resume() {
        let args = vec![
            LlmArg::Task("agentd:proposal".to_string()),
            LlmArg::Model("gemini-2.5-flash".to_string()),
            LlmArg::OutputFormat("stream-json".to_string()),
        ];
        let cli_args = LlmProvider::Gemini.build_args(&args, false);
        assert_eq!(cli_args, vec![
            "agentd:proposal",
            "-m", "gemini-2.5-flash",
            "--output-format", "stream-json"
        ]);
    }

    #[test]
    fn test_gemini_args_with_resume() {
        let args = vec![
            LlmArg::Task("agentd:reproposal".to_string()),
            LlmArg::Model("gemini-2.5-flash".to_string()),
            LlmArg::OutputFormat("stream-json".to_string()),
        ];
        let cli_args = LlmProvider::Gemini.build_args(&args, true);
        assert_eq!(cli_args, vec![
            "agentd:reproposal",
            "--resume", "latest",
            "-m", "gemini-2.5-flash",
            "--output-format", "stream-json"
        ]);
    }

    #[test]
    fn test_codex_args_no_resume() {
        let args = vec![
            LlmArg::Model("gpt-5.2-codex".to_string()),
            LlmArg::Reasoning("medium".to_string()),
            LlmArg::FullAuto,
            LlmArg::Json,
        ];
        let cli_args = LlmProvider::Codex.build_args(&args, false);
        assert_eq!(cli_args, vec![
            "exec",
            "--model", "gpt-5.2-codex",
            "--config", "reasoning=medium",
            "--full-auto",
            "--json"
        ]);
    }

    #[test]
    fn test_codex_args_with_resume() {
        // Codex resume only supports --last and --full-auto
        // --model, --json, --config are inherited from the session
        let args = vec![
            LlmArg::Model("gpt-5.2-codex".to_string()),
            LlmArg::Reasoning("medium".to_string()),
            LlmArg::FullAuto,
            LlmArg::Json,
        ];
        let cli_args = LlmProvider::Codex.build_args(&args, true);
        assert_eq!(cli_args, vec![
            "resume", "--last",
            "--full-auto"
        ]);
    }

    #[test]
    fn test_claude_args_no_resume() {
        let args = vec![
            LlmArg::Model("sonnet".to_string()),
            LlmArg::FullAuto,
        ];
        let cli_args = LlmProvider::Claude.build_args(&args, false);
        assert_eq!(cli_args, vec![
            "--model", "sonnet",
            "--dangerously-skip-permissions"
        ]);
    }

    #[test]
    fn test_claude_args_with_resume() {
        let args = vec![
            LlmArg::Model("sonnet".to_string()),
            LlmArg::FullAuto,
        ];
        let cli_args = LlmProvider::Claude.build_args(&args, true);
        assert_eq!(cli_args, vec![
            "--continue",
            "--model", "sonnet",
            "--dangerously-skip-permissions"
        ]);
    }
}
