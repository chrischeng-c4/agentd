//! Project registry for managing multiple agentd projects with a single HTTP MCP server
//!
//! The registry tracks:
//! - Server process information (PID, port)
//! - Registered projects (path, registration time)
//!
//! Registry file: ~/.agentd/registry.json

use crate::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Project registry stored in ~/.agentd/registry.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    /// Server information
    pub server: ServerInfo,

    /// Registered projects (key: project name)
    pub projects: HashMap<String, ProjectInfo>,
}

/// MCP server process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server process ID
    pub pid: u32,

    /// HTTP server port
    pub port: u16,

    /// Server start time
    pub started_at: DateTime<Utc>,
}

/// Individual project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Absolute path to project directory
    pub path: PathBuf,

    /// Project registration time
    pub registered_at: DateTime<Utc>,
}

impl Registry {
    /// Get the registry file path (~/.agentd/registry.json)
    pub fn registry_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

        let agentd_dir = home.join(".agentd");

        // Create directory if it doesn't exist
        if !agentd_dir.exists() {
            fs::create_dir_all(&agentd_dir)?;
        }

        Ok(agentd_dir.join("registry.json"))
    }

    /// Load registry from file, or create new one if it doesn't exist
    pub fn load() -> Result<Self> {
        let path = Self::registry_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let registry: Registry = serde_json::from_str(&content)?;
            Ok(registry)
        } else {
            // Return error if registry doesn't exist (server not started)
            Err(anyhow::anyhow!("Registry not found. Server not running?"))
        }
    }

    /// Save registry to file
    pub fn save(&self) -> Result<()> {
        let path = Self::registry_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Create a new registry with server info
    pub fn new(pid: u32, port: u16) -> Self {
        Self {
            server: ServerInfo {
                pid,
                port,
                started_at: Utc::now(),
            },
            projects: HashMap::new(),
        }
    }

    /// Check if server is still running
    pub fn is_server_running(&self) -> bool {
        process_exists(self.server.pid)
    }

    /// Register a new project
    pub fn register_project(&mut self, name: String, path: PathBuf) -> Result<()> {
        self.projects.insert(
            name,
            ProjectInfo {
                path,
                registered_at: Utc::now(),
            },
        );
        self.save()?;
        Ok(())
    }

    /// Unregister a project
    pub fn unregister_project(&mut self, name: &str) -> Result<()> {
        self.projects.remove(name);
        self.save()?;
        Ok(())
    }

    /// Get project path by name
    pub fn get_project_path(&self, name: &str) -> Option<&PathBuf> {
        self.projects.get(name).map(|p| &p.path)
    }

    /// Get project name from path
    pub fn get_project_name(&self, path: &Path) -> Option<String> {
        for (name, info) in &self.projects {
            if info.path == path {
                return Some(name.clone());
            }
        }
        None
    }

    /// List all registered projects
    pub fn list_projects(&self) -> Vec<(String, &ProjectInfo)> {
        self.projects
            .iter()
            .map(|(name, info)| (name.clone(), info))
            .collect()
    }

    /// Delete registry file (when server shuts down)
    pub fn delete() -> Result<()> {
        let path = Self::registry_path()?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

/// Check if a process with given PID exists
#[cfg(unix)]
fn process_exists(pid: u32) -> bool {
    use std::process::Command;

    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    use std::process::Command;

    Command::new("tasklist")
        .arg("/FI")
        .arg(format!("PID eq {}", pid))
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(&pid.to_string())
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = Registry::new(12345, 3000);
        assert_eq!(registry.server.pid, 12345);
        assert_eq!(registry.server.port, 3000);
        assert_eq!(registry.projects.len(), 0);
    }

    #[test]
    fn test_register_project() {
        let mut registry = Registry::new(12345, 3000);
        let path = PathBuf::from("/tmp/test-project");

        registry.register_project("test".to_string(), path.clone()).unwrap();

        assert_eq!(registry.projects.len(), 1);
        assert_eq!(registry.get_project_path("test"), Some(&path));
    }

    #[test]
    fn test_unregister_project() {
        let mut registry = Registry::new(12345, 3000);
        let path = PathBuf::from("/tmp/test-project");

        registry.register_project("test".to_string(), path).unwrap();
        assert_eq!(registry.projects.len(), 1);

        registry.unregister_project("test").unwrap();
        assert_eq!(registry.projects.len(), 0);
    }
}
