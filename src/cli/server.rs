//! Unified Server management CLI commands (R1)
//!
//! Manages the unified HTTP server lifecycle and project registration:
//! - start: Start server and register current project
//! - stop: Unregister a project
//! - list: List all registered projects
//! - view: Open browser to Plan Viewer URL (R7)
//! - shutdown: Stop the entire server

use crate::mcp::Registry;
use crate::Result;
use clap::Subcommand;
use colored::Colorize;
use std::env;
use std::process::{Command, Stdio};

#[derive(Subcommand)]
pub enum ServerCommands {
    /// Start unified server and register current project
    Start {
        /// HTTP server port (default: 3456)
        #[arg(long, default_value = "3456")]
        port: u16,

        /// Auto-update client configurations (.gemini/settings.json, ~/.codex/config.toml)
        #[arg(long)]
        update_clients: bool,

        /// Run in daemon mode (background)
        #[arg(long)]
        daemon: bool,
    },

    /// Stop/unregister a project from the server
    Stop {
        /// Project name to unregister (default: current directory name)
        project: Option<String>,
    },

    /// List all registered projects
    List,

    /// Open Plan Viewer in browser for a specific change (R7)
    View {
        /// Project name
        project: String,
        /// Change ID
        change: String,
    },

    /// Shutdown the entire server
    Shutdown,

    /// Run HTTP server (internal use, started by daemon)
    #[command(hide = true)]
    Run {
        #[arg(long)]
        port: u16,
    },
}

pub async fn run(cmd: ServerCommands) -> Result<()> {
    match cmd {
        ServerCommands::Start {
            port,
            update_clients,
            daemon,
        } => start_server(port, update_clients, daemon)?,

        ServerCommands::Stop { project } => stop_project(project)?,

        ServerCommands::List => list_projects()?,

        ServerCommands::View { project, change } => view_change(&project, &change)?,

        ServerCommands::Shutdown => shutdown_server()?,

        ServerCommands::Run { port } => {
            // This is called by the daemon process
            run_server_daemon(port).await?
        }
    }

    Ok(())
}

/// Start server and register current project
fn start_server(port: u16, update_clients: bool, daemon: bool) -> Result<()> {
    let current_dir = env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Cannot determine project name"))?
        .to_string();

    // Check if server is already running
    let mut registry = match Registry::load() {
        Ok(reg) => {
            // Server exists, check if it's still running
            if reg.is_server_running() {
                println!("{}", "✓ Server already running".green());
                reg
            } else {
                // Server process died, clean up and start new one
                println!(
                    "{}",
                    "⚠ Previous server process not found, starting new one".yellow()
                );
                Registry::delete()?;
                start_server_process(port, daemon)?
            }
        }
        Err(_) => {
            // No registry, start new server
            start_server_process(port, daemon)?
        }
    };

    // Register current project
    if registry.get_project_path(&project_name).is_some() {
        println!(
            "{}",
            format!("✓ Project '{}' already registered", project_name).green()
        );
    } else {
        registry.register_project(project_name.clone(), current_dir.clone())?;
        println!(
            "{}",
            format!("✓ Project '{}' registered", project_name).green()
        );
    }

    println!(
        "{}",
        format!(
            "✓ Server running on http://localhost:{}",
            registry.server.port
        )
        .cyan()
    );
    println!(
        "{}",
        format!("  Dashboard: http://localhost:{}/", registry.server.port).cyan()
    );
    println!(
        "{}",
        format!("✓ Project path: {}", current_dir.display()).cyan()
    );

    // Update client configurations if requested
    if update_clients {
        update_client_configs(&project_name, &current_dir, registry.server.port)?;
    }

    Ok(())
}

/// Start the actual server process
fn start_server_process(port: u16, daemon: bool) -> Result<Registry> {
    let exe_path = env::current_exe()?;

    // Create registry FIRST with placeholder PID
    let registry = Registry::new(0, port);
    registry.save()?;

    let mut cmd = Command::new(&exe_path);
    cmd.args(["server", "run", "--port", &port.to_string()]);

    if daemon {
        // Daemonize: detach from terminal
        #[cfg(unix)]
        {
            cmd.stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
        }

        let child = cmd.spawn()?;
        let pid = child.id();

        // Update registry with actual PID
        let mut registry = Registry::load()?;
        registry.server.pid = pid;
        registry.save()?;

        // Wait a bit to ensure server starts
        std::thread::sleep(std::time::Duration::from_millis(500));

        println!("{}", format!("✓ Server started (PID: {})", pid).green());

        Ok(registry)
    } else {
        // Run in foreground (for testing)
        let child = cmd.spawn()?;
        let pid = child.id();

        let mut registry = Registry::load()?;
        registry.server.pid = pid;
        registry.save()?;

        println!("{}", format!("✓ Server started (PID: {})", pid).green());

        Ok(registry)
    }
}

/// Run the HTTP server (called by daemon process)
async fn run_server_daemon(port: u16) -> Result<()> {
    // Load registry to get initial state
    let registry = Registry::load()?;

    // Start unified HTTP server
    crate::mcp::start_server(port, registry).await?;

    Ok(())
}

/// Stop/unregister a project
fn stop_project(project: Option<String>) -> Result<()> {
    let mut registry = Registry::load()?;

    let project_name = if let Some(name) = project {
        name
    } else {
        // Use current directory name
        let current_dir = env::current_dir()?;
        current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Cannot determine project name"))?
            .to_string()
    };

    if registry.get_project_path(&project_name).is_none() {
        println!(
            "{}",
            format!("⚠ Project '{}' not registered", project_name).yellow()
        );
        return Ok(());
    }

    registry.unregister_project(&project_name)?;
    println!(
        "{}",
        format!("✓ Project '{}' unregistered", project_name).green()
    );

    Ok(())
}

/// List all registered projects
fn list_projects() -> Result<()> {
    let registry = Registry::load().map_err(|_| {
        anyhow::anyhow!("No server running. Use 'agentd server start' to start the server.")
    })?;

    println!("\n{}", "Server Status".bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Port:    {}", registry.server.port);
    println!("  PID:     {}", registry.server.pid);
    println!(
        "  Started: {}",
        registry.server.started_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "  Status:  {}",
        if registry.is_server_running() {
            "Running".green()
        } else {
            "Dead".red()
        }
    );
    println!(
        "  Dashboard: http://localhost:{}/",
        registry.server.port
    );

    println!("\n{}", "Registered Projects".bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if registry.projects.is_empty() {
        println!("  {}", "No projects registered".yellow());
    } else {
        for (name, info) in registry.list_projects() {
            println!("  {} {}", "●".green(), name.bold());
            println!("    Path: {}", info.path.display());
            println!(
                "    Registered: {}",
                info.registered_at.format("%Y-%m-%d %H:%M:%S")
            );
        }
    }

    println!();
    Ok(())
}

/// Open Plan Viewer for a specific change in the default browser (R7)
fn view_change(project: &str, change: &str) -> Result<()> {
    let registry = Registry::load().map_err(|_| {
        anyhow::anyhow!("No server running. Use 'agentd server start' to start the server first.")
    })?;

    if !registry.is_server_running() {
        anyhow::bail!(
            "Server is not running. Use 'agentd server start' to start it first."
        );
    }

    // Verify project is registered
    let project_path = registry.get_project_path(project).ok_or_else(|| {
        anyhow::anyhow!("Project '{}' is not registered", project)
    })?;

    // Verify change exists
    let change_dir = project_path.join("agentd/changes").join(change);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found in project '{}'\nPath: {}",
            change,
            project,
            change_dir.display()
        );
    }

    let url = format!(
        "http://localhost:{}/view/{}/{}/",
        registry.server.port, project, change
    );

    println!("{}", format!("Opening: {}", url).cyan());

    #[cfg(feature = "ui")]
    {
        if let Err(e) = open::that(&url) {
            eprintln!(
                "Failed to open browser: {}. Please open {} manually.",
                e, url
            );
        }
    }

    #[cfg(not(feature = "ui"))]
    {
        println!("Open this URL in your browser: {}", url);
    }

    Ok(())
}

/// Shutdown the entire server
fn shutdown_server() -> Result<()> {
    let registry = Registry::load().map_err(|_| anyhow::anyhow!("No server running."))?;

    let pid = registry.server.pid;

    // Kill the server process
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill").arg(pid.to_string()).output()?;
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()?;
    }

    // Delete registry
    Registry::delete()?;

    println!(
        "{}",
        format!("✓ Server shut down (PID: {})", pid).green()
    );

    Ok(())
}

/// Update client configurations
fn update_client_configs(
    project_name: &str,
    project_path: &std::path::Path,
    port: u16,
) -> Result<()> {
    println!("\n{}", "Updating client configurations...".cyan());

    // Update .gemini/settings.json
    update_gemini_config(project_name, project_path, port)?;

    // Update ~/.codex/config.toml
    update_codex_config(project_name, project_path, port)?;

    println!("{}", "✓ Client configurations updated".green());

    Ok(())
}

/// Update Gemini CLI configuration
fn update_gemini_config(
    project_name: &str,
    project_path: &std::path::Path,
    port: u16,
) -> Result<()> {
    use serde_json::{json, Value};
    use std::fs;

    let gemini_dir = project_path.join(".gemini");
    fs::create_dir_all(&gemini_dir)?;

    let settings_file = gemini_dir.join("settings.json");

    // Read existing settings or create new
    let mut settings: Value = if settings_file.exists() {
        let content = fs::read_to_string(&settings_file)?;
        serde_json::from_str(&content)?
    } else {
        json!({})
    };

    // Update mcpServers section
    if settings.get("mcpServers").is_none() {
        settings["mcpServers"] = json!({});
    }

    settings["mcpServers"]["agentd"] = json!({
        "url": format!("http://localhost:{}/mcp", port),
        "headers": {
            "X-Agentd-Project": project_name,
            "X-Agentd-Cwd": project_path.to_str().unwrap()
        },
        "timeout": 30000
    });

    // Write back
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_file, content)?;

    println!("  ✓ Updated {}", settings_file.display());

    Ok(())
}

/// Update Codex configuration
fn update_codex_config(
    project_name: &str,
    project_path: &std::path::Path,
    port: u16,
) -> Result<()> {
    use std::fs;

    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    let codex_dir = home.join(".codex");
    fs::create_dir_all(&codex_dir)?;

    let config_file = codex_dir.join("config.toml");

    // Read existing config or create new
    let mut config_content = if config_file.exists() {
        fs::read_to_string(&config_file)?
    } else {
        String::new()
    };

    // Simple TOML generation (TODO: use toml crate for proper parsing)
    let server_config = format!(
        r#"
[mcp_servers.agentd]
url = "http://localhost:{}/mcp"
timeout = 30000

[mcp_servers.agentd.http_headers]
X-Agentd-Project = "{}"
X-Agentd-Cwd = "{}"
"#,
        port,
        project_name,
        project_path.display()
    );

    // Append if not already present
    if !config_content.contains("[mcp_servers.agentd]") {
        config_content.push_str(&server_config);
        fs::write(&config_file, config_content)?;
        println!("  ✓ Updated {}", config_file.display());
    } else {
        println!("  ⚠ Codex config already has agentd server, skipping");
    }

    Ok(())
}
