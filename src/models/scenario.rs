use serde::{Deserialize, Serialize};

/// Represents a scenario with WHEN/THEN clauses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scenario {
    /// Scenario name (e.g., "Valid credentials")
    pub name: String,

    /// WHEN clauses (preconditions)
    pub when: Vec<String>,

    /// THEN clauses (expected outcomes)
    pub then: Vec<String>,

    /// AND clauses (additional conditions, optional)
    pub and: Vec<String>,
}

impl Scenario {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            when: Vec::new(),
            then: Vec::new(),
            and: Vec::new(),
        }
    }

    pub fn with_when(mut self, clause: impl Into<String>) -> Self {
        self.when.push(clause.into());
        self
    }

    pub fn with_then(mut self, clause: impl Into<String>) -> Self {
        self.then.push(clause.into());
        self
    }

    pub fn with_and(mut self, clause: impl Into<String>) -> Self {
        self.and.push(clause.into());
        self
    }

    /// Validate that scenario has at least one WHEN and one THEN clause
    pub fn validate(&self) -> Result<(), String> {
        if self.when.is_empty() {
            return Err(format!("Scenario '{}' has no WHEN clauses", self.name));
        }
        if self.then.is_empty() {
            return Err(format!("Scenario '{}' has no THEN clauses", self.name));
        }
        Ok(())
    }
}
