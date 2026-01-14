use crate::models::{
    AgentdConfig, ClaudeModelConfig, CodexModelConfig, Complexity, GeminiModelConfig,
};

/// Model selection result for different AI tools
#[derive(Debug, Clone)]
pub enum SelectedModel {
    /// Gemini model selection
    Gemini {
        model: String,
        command: String,
    },
    /// Codex model selection (includes reasoning level)
    Codex {
        model: String,
        reasoning: Option<String>,
        command: String,
    },
    /// Claude model selection
    Claude {
        model: String,
        command: String,
    },
}

impl SelectedModel {
    /// Get the CLI argument for the selected model
    pub fn to_cli_arg(&self) -> String {
        match self {
            SelectedModel::Gemini { model, .. } => model.clone(),
            SelectedModel::Codex { model, reasoning, .. } => {
                match reasoning {
                    Some(level) => format!("{} {}", model, level),
                    None => model.clone(),
                }
            }
            SelectedModel::Claude { model, .. } => model.clone(),
        }
    }

    /// Get the command to invoke (e.g., "gemini", "codex", "claude")
    pub fn command(&self) -> &str {
        match self {
            SelectedModel::Gemini { command, .. } => command,
            SelectedModel::Codex { command, .. } => command,
            SelectedModel::Claude { command, .. } => command,
        }
    }
}

/// Model selector for choosing appropriate models based on task complexity
pub struct ModelSelector<'a> {
    config: &'a AgentdConfig,
}

impl<'a> ModelSelector<'a> {
    pub fn new(config: &'a AgentdConfig) -> Self {
        Self { config }
    }

    /// Select Gemini model based on complexity
    pub fn select_gemini(&self, complexity: Complexity) -> SelectedModel {
        let model_config = self.config.gemini.select_model(complexity);
        SelectedModel::Gemini {
            model: model_config.model.clone(),
            command: self.config.gemini.command.clone(),
        }
    }

    /// Select Codex model based on complexity
    pub fn select_codex(&self, complexity: Complexity) -> SelectedModel {
        let model_config = self.config.codex.select_model(complexity);
        SelectedModel::Codex {
            model: model_config.model.clone(),
            reasoning: model_config.reasoning.clone(),
            command: self.config.codex.command.clone(),
        }
    }

    /// Select Claude model based on complexity
    pub fn select_claude(&self, complexity: Complexity) -> SelectedModel {
        let model_config = self.config.claude.select_model(complexity);
        SelectedModel::Claude {
            model: model_config.model.clone(),
            command: self.config.claude.command.clone(),
        }
    }

    /// Get default Gemini model
    pub fn default_gemini(&self) -> SelectedModel {
        let model_config = self.config.gemini.default_model();
        SelectedModel::Gemini {
            model: model_config.model.clone(),
            command: self.config.gemini.command.clone(),
        }
    }

    /// Get default Codex model
    pub fn default_codex(&self) -> SelectedModel {
        let model_config = self.config.codex.default_model();
        SelectedModel::Codex {
            model: model_config.model.clone(),
            reasoning: model_config.reasoning.clone(),
            command: self.config.codex.command.clone(),
        }
    }

    /// Get default Claude model
    pub fn default_claude(&self) -> SelectedModel {
        let model_config = self.config.claude.default_model();
        SelectedModel::Claude {
            model: model_config.model.clone(),
            command: self.config.claude.command.clone(),
        }
    }

    /// Get Gemini model config reference
    pub fn gemini_config(&self, complexity: Complexity) -> &GeminiModelConfig {
        self.config.gemini.select_model(complexity)
    }

    /// Get Codex model config reference
    pub fn codex_config(&self, complexity: Complexity) -> &CodexModelConfig {
        self.config.codex.select_model(complexity)
    }

    /// Get Claude model config reference
    pub fn claude_config(&self, complexity: Complexity) -> &ClaudeModelConfig {
        self.config.claude.select_model(complexity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_selection_by_complexity() {
        let config = AgentdConfig::default();
        let selector = ModelSelector::new(&config);

        // Low complexity should select flash model
        let gemini = selector.select_gemini(Complexity::Low);
        assert!(matches!(gemini, SelectedModel::Gemini { model, .. } if model.contains("flash")));

        // Critical complexity should select pro model
        let gemini = selector.select_gemini(Complexity::Critical);
        assert!(matches!(gemini, SelectedModel::Gemini { model, .. } if model.contains("pro")));

        // Codex low should have low reasoning
        let codex = selector.select_codex(Complexity::Low);
        if let SelectedModel::Codex { reasoning, .. } = codex {
            assert_eq!(reasoning, Some("low".to_string()));
        }

        // Codex critical should have extra high reasoning
        let codex = selector.select_codex(Complexity::Critical);
        if let SelectedModel::Codex { reasoning, .. } = codex {
            assert_eq!(reasoning, Some("extra high".to_string()));
        }
    }

    #[test]
    fn test_codex_cli_arg() {
        let config = AgentdConfig::default();
        let selector = ModelSelector::new(&config);

        let codex = selector.select_codex(Complexity::Medium);
        let arg = codex.to_cli_arg();
        assert!(arg.contains("medium"));
    }
}
