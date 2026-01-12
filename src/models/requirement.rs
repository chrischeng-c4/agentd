use serde::{Deserialize, Serialize};
use super::Scenario;

/// Represents a requirement with scenarios
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Requirement {
    /// Requirement name (e.g., "User Authentication")
    pub name: String,

    /// Description of the requirement
    pub description: String,

    /// List of scenarios that validate this requirement
    pub scenarios: Vec<Scenario>,
}

impl Requirement {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            scenarios: Vec::new(),
        }
    }

    pub fn with_scenario(mut self, scenario: Scenario) -> Self {
        self.scenarios.push(scenario);
        self
    }

    /// Validate requirement format and completeness
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Requirement name cannot be empty".to_string());
        }
        if self.description.is_empty() {
            return Err(format!("Requirement '{}' has no description", self.name));
        }
        if self.scenarios.is_empty() {
            return Err(format!("Requirement '{}' has no scenarios", self.name));
        }

        // Validate all scenarios
        for scenario in &self.scenarios {
            scenario.validate()?;
        }

        Ok(())
    }
}

/// Represents different types of requirement changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequirementDelta {
    /// New requirement added
    Added(Requirement),

    /// Existing requirement modified (full text required)
    Modified(Requirement),

    /// Requirement removed (name + reason)
    Removed {
        name: String,
        reason: String,
        migration: Option<String>,
    },

    /// Requirement renamed
    Renamed {
        from: String,
        to: String,
    },
}

impl RequirementDelta {
    pub fn added(req: Requirement) -> Self {
        Self::Added(req)
    }

    pub fn modified(req: Requirement) -> Self {
        Self::Modified(req)
    }

    pub fn removed(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Removed {
            name: name.into(),
            reason: reason.into(),
            migration: None,
        }
    }

    pub fn renamed(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::Renamed {
            from: from.into(),
            to: to.into(),
        }
    }
}
