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
/// If the file exists, adds mcpServers.agentd if missing.
/// If the file doesn't exist, creates it with the MCP server config.
pub fn ensure_gemini_mcp_config(project_root: &Path) -> Result<()> {
    let settings_path = project_root.join(".gemini/settings.json");

    if settings_path.exists() {
        // Read existing settings
        let content = std::fs::read_to_string(&settings_path)?;
        let mut settings: serde_json::Value = serde_json::from_str(&content)?;

        // Check if mcpServers.agentd exists
        let has_agentd = settings
            .get("mcpServers")
            .and_then(|s| s.get("agentd"))
            .is_some();

        if !has_agentd {
            // Ensure mcpServers object exists
            if settings.get("mcpServers").is_none() {
                settings["mcpServers"] = serde_json::json!({});
            }

            // Add agentd server config
            settings["mcpServers"]["agentd"] = serde_json::json!({
                "command": "agentd-mcp",
                "args": ["mcp-server"]
            });

            // Write back
            let content = serde_json::to_string_pretty(&settings)?;
            std::fs::write(&settings_path, content)?;
        }
    } else {
        // Create new settings.json with mcpServers
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let settings = serde_json::json!({
            "mcpServers": {
                "agentd": {
                    "command": "agentd-mcp",
                    "args": ["mcp-server"]
                }
            }
        });

        let content = serde_json::to_string_pretty(&settings)?;
        std::fs::write(&settings_path, content)?;
    }

    Ok(())
}

/// Ensure Claude Code MCP config exists in .mcp.json (project root)
///
/// Creates .mcp.json with the agentd MCP server config.
/// This is read by Claude Code to discover available MCP servers.
pub fn ensure_claude_mcp_json(project_root: &Path) -> Result<()> {
    let mcp_json_path = project_root.join(".mcp.json");

    if mcp_json_path.exists() {
        // Read existing config
        let content = std::fs::read_to_string(&mcp_json_path)?;
        let mut config: serde_json::Value = serde_json::from_str(&content)?;

        // Check if mcpServers.agentd exists
        let has_agentd = config
            .get("mcpServers")
            .and_then(|s| s.get("agentd"))
            .is_some();

        if !has_agentd {
            // Ensure mcpServers object exists
            if config.get("mcpServers").is_none() {
                config["mcpServers"] = serde_json::json!({});
            }

            // Add agentd server config (stdio mode for Claude Code)
            config["mcpServers"]["agentd"] = serde_json::json!({
                "command": "agentd-mcp",
                "args": ["mcp-server"]
            });

            // Write back
            let content = serde_json::to_string_pretty(&config)?;
            std::fs::write(&mcp_json_path, content)?;
        }
    } else {
        // Create new .mcp.json
        let config = serde_json::json!({
            "mcpServers": {
                "agentd": {
                    "command": "agentd-mcp",
                    "args": ["mcp-server"]
                }
            }
        });

        let content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&mcp_json_path, content)?;
    }

    Ok(())
}

/// Ensure Claude Code settings enable the agentd MCP server
///
/// Updates .claude/settings.local.json to enable the agentd MCP server.
pub fn ensure_claude_settings(project_root: &Path) -> Result<()> {
    let claude_dir = project_root.join(".claude");
    std::fs::create_dir_all(&claude_dir)?;

    let settings_path = claude_dir.join("settings.local.json");

    if settings_path.exists() {
        // Read existing settings
        let content = std::fs::read_to_string(&settings_path)?;
        let mut settings: serde_json::Value = serde_json::from_str(&content)?;

        // Check if enableAllProjectMcpServers is set
        let all_enabled = settings
            .get("enableAllProjectMcpServers")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Check if agentd is in enabledMcpjsonServers
        let has_agentd = settings
            .get("enabledMcpjsonServers")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().any(|v| v.as_str() == Some("agentd")))
            .unwrap_or(false);

        if !all_enabled && !has_agentd {
            // Add agentd to enabledMcpjsonServers
            let servers = settings
                .get_mut("enabledMcpjsonServers")
                .and_then(|v| v.as_array_mut());

            if let Some(servers) = servers {
                servers.push(serde_json::json!("agentd"));
            } else {
                settings["enabledMcpjsonServers"] = serde_json::json!(["agentd"]);
            }

            // Write back
            let content = serde_json::to_string_pretty(&settings)?;
            std::fs::write(&settings_path, content)?;
        }
    } else {
        // Create new settings.local.json
        let settings = serde_json::json!({
            "enabledMcpjsonServers": ["agentd"]
        });

        let content = serde_json::to_string_pretty(&settings)?;
        std::fs::write(&settings_path, content)?;
    }

    Ok(())
}

/// Ensure Codex MCP config exists in ~/.codex/config.toml
///
/// If the file exists, adds [mcp_servers.agentd] if missing.
/// If the file doesn't exist, creates it with the MCP server config.
pub fn ensure_codex_mcp_config() -> Result<()> {
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;

    let config_path = std::path::PathBuf::from(&home_dir).join(".codex/config.toml");

    if config_path.exists() {
        // Read existing config
        let content = std::fs::read_to_string(&config_path)?;
        let mut config: toml::Value = content.parse()?;

        // Check if mcp_servers.agentd exists
        let has_agentd = config
            .get("mcp_servers")
            .and_then(|s| s.get("agentd"))
            .is_some();

        if !has_agentd {
            // Build the agentd server config
            let mut agentd_config = toml::map::Map::new();
            agentd_config.insert("command".to_string(), toml::Value::String("agentd-mcp".to_string()));
            agentd_config.insert(
                "args".to_string(),
                toml::Value::Array(vec![toml::Value::String("mcp-server".to_string())]),
            );

            // Get or create mcp_servers table
            if let Some(root_table) = config.as_table_mut() {
                let mcp_servers = root_table
                    .entry("mcp_servers".to_string())
                    .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

                if let Some(mcp_table) = mcp_servers.as_table_mut() {
                    mcp_table.insert("agentd".to_string(), toml::Value::Table(agentd_config));
                }
            }

            // Write back
            let content = toml::to_string_pretty(&config)?;
            std::fs::write(&config_path, content)?;
        }
    } else {
        // Create new config.toml with mcp_servers section
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = r#"[mcp_servers.agentd]
command = "agentd-mcp"
args = ["mcp-server"]
"#;
        std::fs::write(&config_path, content)?;
    }

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

        // Verify content
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(settings["mcpServers"]["agentd"]["command"].as_str() == Some("agentd-mcp"));
        assert!(settings["mcpServers"]["agentd"]["args"][0].as_str() == Some("mcp-server"));
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
        assert!(settings["mcpServers"]["agentd"]["command"].as_str() == Some("agentd-mcp"));
    }

    #[test]
    fn test_ensure_gemini_mcp_config_preserves_existing_agentd() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .gemini directory with existing mcpServers.agentd
        let gemini_dir = project_root.join(".gemini");
        std::fs::create_dir_all(&gemini_dir).unwrap();
        let settings_path = gemini_dir.join("settings.json");
        let existing = r#"{"mcpServers": {"agentd": {"command": "custom-agentd", "args": ["custom-arg"]}}}"#;
        std::fs::write(&settings_path, existing).unwrap();

        // Ensure config
        ensure_gemini_mcp_config(project_root).unwrap();

        // Verify existing config was preserved (not overwritten)
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(settings["mcpServers"]["agentd"]["command"].as_str() == Some("custom-agentd"));
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

        // Verify content
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[mcp_servers.agentd]"));
        assert!(content.contains("command = \"agentd-mcp\""));
        assert!(content.contains("args = [\"mcp-server\"]"));
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
        assert!(content.contains("[mcp_servers.agentd]"));
    }

    #[test]
    fn test_ensure_codex_mcp_config_preserves_existing_agentd() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path();

        // Create .codex directory with existing mcp_servers.agentd
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

        // Verify existing config was preserved (not overwritten)
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("custom-agentd"));
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

        // Verify content
        let content = std::fs::read_to_string(&mcp_json_path).unwrap();
        let config: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(config["mcpServers"]["agentd"]["command"].as_str() == Some("agentd-mcp"));
        assert!(config["mcpServers"]["agentd"]["args"][0].as_str() == Some("mcp-server"));
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

        // Verify agentd was added while preserving existing
        let content = std::fs::read_to_string(&mcp_json_path).unwrap();
        let config: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(config["mcpServers"]["other"]["command"].as_str() == Some("other-cmd"));
        assert!(config["mcpServers"]["agentd"]["command"].as_str() == Some("agentd-mcp"));
    }

    #[test]
    fn test_ensure_claude_mcp_json_preserves_existing_agentd() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .mcp.json with existing agentd config
        let mcp_json_path = project_root.join(".mcp.json");
        let existing = r#"{"mcpServers": {"agentd": {"command": "custom-agentd", "args": ["custom-arg"]}}}"#;
        std::fs::write(&mcp_json_path, existing).unwrap();

        // Ensure config
        ensure_claude_mcp_json(project_root).unwrap();

        // Verify existing config was preserved
        let content = std::fs::read_to_string(&mcp_json_path).unwrap();
        let config: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(config["mcpServers"]["agentd"]["command"].as_str() == Some("custom-agentd"));
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
        assert!(servers.iter().any(|v| v.as_str() == Some("agentd")));
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

        // Verify agentd was added while preserving existing
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(settings["permissions"]["allow"][0].as_str() == Some("Bash"));
        let servers = settings["enabledMcpjsonServers"].as_array().unwrap();
        assert!(servers.iter().any(|v| v.as_str() == Some("agentd")));
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
