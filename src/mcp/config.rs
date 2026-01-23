//! MCP Configuration Generator
//!
//! Generates MCP configuration files for LLM providers to connect to the agentd MCP server.
//!
//! Supports:
//! - Gemini: `.gemini/settings.json` (project-level)
//! - Codex: `~/.codex/config.toml` (user-level)

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// MCP configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

impl McpConfig {
    /// Create a new MCP config with the agentd server
    pub fn new() -> Self {
        let mut servers = HashMap::new();
        servers.insert(
            "agentd".to_string(),
            McpServerConfig {
                command: "agentd-mcp".to_string(),
                args: vec!["mcp-server".to_string()],
                env: None,
            },
        );
        Self {
            mcp_servers: servers,
        }
    }

    /// Write the config to a file
    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the default config file path for a project
    pub fn default_path(project_root: &Path) -> std::path::PathBuf {
        project_root.join("agentd/mcp.json")
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Ensure MCP config file exists, creating it if necessary
pub fn ensure_mcp_config(project_root: &Path) -> Result<std::path::PathBuf> {
    let config_path = McpConfig::default_path(project_root);

    if !config_path.exists() {
        // Create parent directory if needed
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let config = McpConfig::new();
        config.write_to_file(&config_path)?;
    }

    Ok(config_path)
}

/// Ensure Gemini MCP config exists in .gemini/settings.json
///
/// Always overwrites mcpServers.agentd-mcp with HTTP format config.
/// Note: project_path is now a parameter in each tool call, not in headers.
pub fn ensure_gemini_mcp_config(project_root: &Path) -> Result<()> {
    let settings_path = project_root.join(".gemini/settings.json");

    let mut settings = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content)?
    } else {
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::json!({})
    };

    // Ensure mcpServers object exists
    if settings.get("mcpServers").is_none() {
        settings["mcpServers"] = serde_json::json!({});
    }

    // Remove old "agentd" entry if exists
    if let Some(servers) = settings["mcpServers"].as_object_mut() {
        servers.remove("agentd");
    }

    // Force overwrite agentd-mcp server config (HTTP mode, no headers)
    settings["mcpServers"]["agentd-mcp"] = serde_json::json!({
        "type": "http",
        "url": "http://localhost:3456/mcp"
    });

    let content = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&settings_path, content)?;

    Ok(())
}

/// Ensure Claude Code MCP config exists in .mcp.json (project root)
///
/// Always overwrites mcpServers.agentd-mcp with HTTP format config.
/// Uses "agentd-mcp" as server name to avoid Claude Code blocking "agentd" CLI.
/// Note: project_path is now a parameter in each tool call, not in headers.
pub fn ensure_claude_mcp_json(project_root: &Path) -> Result<()> {
    let mcp_json_path = project_root.join(".mcp.json");

    let mut config = if mcp_json_path.exists() {
        let content = std::fs::read_to_string(&mcp_json_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Ensure mcpServers object exists
    if config.get("mcpServers").is_none() {
        config["mcpServers"] = serde_json::json!({});
    }

    // Remove old "agentd" entry if exists
    if let Some(servers) = config["mcpServers"].as_object_mut() {
        servers.remove("agentd");
    }

    // Force overwrite agentd-mcp server config (HTTP mode, no headers)
    config["mcpServers"]["agentd-mcp"] = serde_json::json!({
        "type": "http",
        "url": "http://localhost:3456/mcp"
    });

    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&mcp_json_path, content)?;

    Ok(())
}

/// Ensure Claude Code settings enable the agentd-mcp MCP server
///
/// Updates .claude/settings.local.json to enable the agentd-mcp MCP server.
pub fn ensure_claude_settings(project_root: &Path) -> Result<()> {
    let claude_dir = project_root.join(".claude");
    std::fs::create_dir_all(&claude_dir)?;

    let settings_path = claude_dir.join("settings.local.json");

    let mut settings = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Check if enableAllProjectMcpServers is set
    let all_enabled = settings
        .get("enableAllProjectMcpServers")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !all_enabled {
        // Get or create enabledMcpjsonServers array
        let servers = settings
            .get_mut("enabledMcpjsonServers")
            .and_then(|v| v.as_array_mut());

        if let Some(servers) = servers {
            // Remove old "agentd" entry if exists
            servers.retain(|v| v.as_str() != Some("agentd"));
            // Add "agentd-mcp" if not present
            if !servers.iter().any(|v| v.as_str() == Some("agentd-mcp")) {
                servers.push(serde_json::json!("agentd-mcp"));
            }
        } else {
            settings["enabledMcpjsonServers"] = serde_json::json!(["agentd-mcp"]);
        }

        // Write back
        let content = serde_json::to_string_pretty(&settings)?;
        std::fs::write(&settings_path, content)?;
    }

    Ok(())
}

/// Ensure Codex MCP config exists in ~/.codex/config.toml
///
/// Always overwrites mcp_servers.agentd with HTTP format config.
pub fn ensure_codex_mcp_config() -> Result<()> {
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;

    let config_path = std::path::PathBuf::from(&home_dir).join(".codex/config.toml");

    let mut config: toml::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        content.parse()?
    } else {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        toml::Value::Table(toml::map::Map::new())
    };

    // Build the agentd server config (HTTP mode)
    let mut agentd_config = toml::map::Map::new();
    agentd_config.insert("type".to_string(), toml::Value::String("http".to_string()));
    agentd_config.insert("url".to_string(), toml::Value::String("http://localhost:3456/mcp".to_string()));

    // Get or create mcp_servers table and force overwrite agentd-mcp
    if let Some(root_table) = config.as_table_mut() {
        let mcp_servers = root_table
            .entry("mcp_servers".to_string())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

        if let Some(mcp_table) = mcp_servers.as_table_mut() {
            // Remove old "agentd" entry if exists
            mcp_table.remove("agentd");
            mcp_table.insert("agentd-mcp".to_string(), toml::Value::Table(agentd_config));
        }
    }

    let content = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_mcp_config_generation() {
        let config = McpConfig::new();
        assert!(config.mcp_servers.contains_key("agentd"));

        let agentd = &config.mcp_servers["agentd"];
        assert_eq!(agentd.command, "agentd-mcp");
        assert_eq!(agentd.args, vec!["mcp-server"]);
    }

    #[test]
    fn test_mcp_config_serialization() {
        let config = McpConfig::new();
        let json = serde_json::to_string_pretty(&config).unwrap();

        assert!(json.contains("\"mcpServers\""));
        assert!(json.contains("\"agentd\""));
        assert!(json.contains("\"mcp-server\""));
    }

    #[test]
    fn test_ensure_mcp_config() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create agentd directory
        std::fs::create_dir_all(project_root.join("agentd")).unwrap();

        let config_path = ensure_mcp_config(project_root).unwrap();
        assert!(config_path.exists());

        // Read back and verify
        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: McpConfig = serde_json::from_str(&content).unwrap();
        assert!(config.mcp_servers.contains_key("agentd"));
    }

    // =========================================================================
    // Gemini MCP Config Tests
    // =========================================================================

    #[test]
    fn test_ensure_gemini_mcp_config_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Ensure config (creates .gemini/settings.json)
        ensure_gemini_mcp_config(project_root).unwrap();

        // Verify file was created
        let settings_path = project_root.join(".gemini/settings.json");
        assert!(settings_path.exists());

        // Verify content (HTTP format for Gemini)
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(settings["mcpServers"]["agentd-mcp"]["type"].as_str(), Some("http"));
        assert_eq!(settings["mcpServers"]["agentd-mcp"]["url"].as_str(), Some("http://localhost:3456/mcp"));
    }

    #[test]
    fn test_ensure_gemini_mcp_config_adds_to_existing() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .gemini directory and existing settings.json without mcpServers
        let gemini_dir = project_root.join(".gemini");
        std::fs::create_dir_all(&gemini_dir).unwrap();
        let settings_path = gemini_dir.join("settings.json");
        std::fs::write(&settings_path, r#"{"tools": {"allowed": ["read_file"]}}"#).unwrap();

        // Ensure config
        ensure_gemini_mcp_config(project_root).unwrap();

        // Verify mcpServers was added while preserving existing content
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(settings["tools"]["allowed"][0].as_str() == Some("read_file"));
        assert_eq!(settings["mcpServers"]["agentd-mcp"]["type"].as_str(), Some("http"));
    }

    #[test]
    fn test_ensure_gemini_mcp_config_removes_old_agentd() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .gemini directory with existing mcpServers.agentd (old format)
        let gemini_dir = project_root.join(".gemini");
        std::fs::create_dir_all(&gemini_dir).unwrap();
        let settings_path = gemini_dir.join("settings.json");
        let existing = r#"{"mcpServers": {"agentd": {"command": "custom-agentd", "args": ["custom-arg"]}}}"#;
        std::fs::write(&settings_path, existing).unwrap();

        // Ensure config
        ensure_gemini_mcp_config(project_root).unwrap();

        // Verify old "agentd" removed and "agentd-mcp" added
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(settings["mcpServers"].get("agentd").is_none());
        assert_eq!(settings["mcpServers"]["agentd-mcp"]["type"].as_str(), Some("http"));
        assert_eq!(settings["mcpServers"]["agentd-mcp"]["url"].as_str(), Some("http://localhost:3456/mcp"));
    }

    // =========================================================================
    // Codex MCP Config Tests
    // =========================================================================

    #[test]
    fn test_ensure_codex_mcp_config_with_temp_home() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path();

        // Set HOME to temp directory for this test
        std::env::set_var("HOME", temp_home.to_str().unwrap());

        // Ensure config
        ensure_codex_mcp_config().unwrap();

        // Verify file was created
        let config_path = temp_home.join(".codex/config.toml");
        assert!(config_path.exists());

        // Verify content (HTTP format)
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[mcp_servers.agentd-mcp]"));
        assert!(content.contains("type = \"http\""));
        assert!(content.contains("url = \"http://localhost:3456/mcp\""));
    }

    #[test]
    fn test_ensure_codex_mcp_config_adds_to_existing() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path();

        // Create .codex directory and existing config.toml
        let codex_dir = temp_home.join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let config_path = codex_dir.join("config.toml");
        std::fs::write(&config_path, "[model]\ndefault = \"gpt-4\"\n").unwrap();

        // Set HOME to temp directory for this test
        std::env::set_var("HOME", temp_home.to_str().unwrap());

        // Ensure config
        ensure_codex_mcp_config().unwrap();

        // Verify mcp_servers was added while preserving existing content
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[model]"));
        assert!(content.contains("default = \"gpt-4\""));
        assert!(content.contains("[mcp_servers.agentd-mcp]"));
        assert!(content.contains("type = \"http\""));
    }

    #[test]
    fn test_ensure_codex_mcp_config_removes_old_agentd() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path();

        // Create .codex directory with existing mcp_servers.agentd (old config)
        let codex_dir = temp_home.join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let config_path = codex_dir.join("config.toml");
        let existing = r#"[mcp_servers.agentd]
command = "custom-agentd"
args = ["custom-arg"]
"#;
        std::fs::write(&config_path, existing).unwrap();

        // Set HOME to temp directory for this test
        std::env::set_var("HOME", temp_home.to_str().unwrap());

        // Ensure config
        ensure_codex_mcp_config().unwrap();

        // Verify old "agentd" removed and "agentd-mcp" added
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[mcp_servers.agentd-mcp]"));
        assert!(content.contains("type = \"http\""));
        assert!(!content.contains("custom-agentd"));
        assert!(!content.contains("[mcp_servers.agentd]\n"));
    }

    // =========================================================================
    // Claude MCP Config Tests
    // =========================================================================

    #[test]
    fn test_ensure_claude_mcp_json_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Ensure config (creates .mcp.json)
        ensure_claude_mcp_json(project_root).unwrap();

        // Verify file was created
        let mcp_json_path = project_root.join(".mcp.json");
        assert!(mcp_json_path.exists());

        // Verify content (HTTP format with agentd-mcp name)
        let content = std::fs::read_to_string(&mcp_json_path).unwrap();
        let config: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(config["mcpServers"]["agentd-mcp"]["type"].as_str(), Some("http"));
        assert_eq!(config["mcpServers"]["agentd-mcp"]["url"].as_str(), Some("http://localhost:3456/mcp"));
    }

    #[test]
    fn test_ensure_claude_mcp_json_adds_to_existing() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create existing .mcp.json with other server
        let mcp_json_path = project_root.join(".mcp.json");
        std::fs::write(&mcp_json_path, r#"{"mcpServers": {"other": {"command": "other-cmd"}}}"#).unwrap();

        // Ensure config
        ensure_claude_mcp_json(project_root).unwrap();

        // Verify agentd-mcp was added while preserving existing
        let content = std::fs::read_to_string(&mcp_json_path).unwrap();
        let config: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(config["mcpServers"]["other"]["command"].as_str() == Some("other-cmd"));
        assert_eq!(config["mcpServers"]["agentd-mcp"]["type"].as_str(), Some("http"));
    }

    #[test]
    fn test_ensure_claude_mcp_json_removes_old_agentd() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .mcp.json with old "agentd" entry
        let mcp_json_path = project_root.join(".mcp.json");
        let existing = r#"{"mcpServers": {"agentd": {"type": "http", "url": "http://localhost:3000/mcp"}}}"#;
        std::fs::write(&mcp_json_path, existing).unwrap();

        // Ensure config
        ensure_claude_mcp_json(project_root).unwrap();

        // Verify old "agentd" removed and "agentd-mcp" added
        let content = std::fs::read_to_string(&mcp_json_path).unwrap();
        let config: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(config["mcpServers"].get("agentd").is_none());
        assert_eq!(config["mcpServers"]["agentd-mcp"]["type"].as_str(), Some("http"));
        assert_eq!(config["mcpServers"]["agentd-mcp"]["url"].as_str(), Some("http://localhost:3456/mcp"));
    }

    #[test]
    fn test_ensure_claude_settings_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Ensure settings
        ensure_claude_settings(project_root).unwrap();

        // Verify file was created
        let settings_path = project_root.join(".claude/settings.local.json");
        assert!(settings_path.exists());

        // Verify content
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        let servers = settings["enabledMcpjsonServers"].as_array().unwrap();
        assert!(servers.iter().any(|v| v.as_str() == Some("agentd-mcp")));
    }

    #[test]
    fn test_ensure_claude_settings_adds_to_existing() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .claude directory with existing settings
        let claude_dir = project_root.join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        let settings_path = claude_dir.join("settings.local.json");
        std::fs::write(&settings_path, r#"{"permissions": {"allow": ["Bash"]}}"#).unwrap();

        // Ensure settings
        ensure_claude_settings(project_root).unwrap();

        // Verify agentd-mcp was added while preserving existing
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(settings["permissions"]["allow"][0].as_str() == Some("Bash"));
        let servers = settings["enabledMcpjsonServers"].as_array().unwrap();
        assert!(servers.iter().any(|v| v.as_str() == Some("agentd-mcp")));
    }

    #[test]
    fn test_ensure_claude_settings_removes_old_agentd() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create settings with old "agentd" entry
        let claude_dir = project_root.join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        let settings_path = claude_dir.join("settings.local.json");
        std::fs::write(&settings_path, r#"{"enabledMcpjsonServers": ["agentd", "other"]}"#).unwrap();

        // Ensure settings
        ensure_claude_settings(project_root).unwrap();

        // Verify old "agentd" removed and "agentd-mcp" added
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        let servers = settings["enabledMcpjsonServers"].as_array().unwrap();
        assert!(!servers.iter().any(|v| v.as_str() == Some("agentd")));
        assert!(servers.iter().any(|v| v.as_str() == Some("agentd-mcp")));
        assert!(servers.iter().any(|v| v.as_str() == Some("other")));
    }

    #[test]
    fn test_ensure_claude_settings_skips_if_all_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create settings with enableAllProjectMcpServers = true
        let claude_dir = project_root.join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        let settings_path = claude_dir.join("settings.local.json");
        std::fs::write(&settings_path, r#"{"enableAllProjectMcpServers": true}"#).unwrap();

        // Ensure settings
        ensure_claude_settings(project_root).unwrap();

        // Verify file unchanged (no enabledMcpjsonServers added)
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(settings.get("enabledMcpjsonServers").is_none());
    }
}
