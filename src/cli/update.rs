use crate::Result;
use colored::Colorize;
use std::env;
use std::process::Command;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = "chrischeng-c4/agentd";

/// Run update command
pub async fn run(check_only: bool) -> Result<()> {
    println!("{}", "🔍 Checking for updates...".cyan());
    println!();

    // Get latest version from GitHub
    let latest = get_latest_version()?;
    let current = CURRENT_VERSION;

    println!("   Current version: {}", current.yellow());
    println!("   Latest version:  {}", latest.green());
    println!();

    if current == latest {
        println!("{}", "✅ You're already on the latest version!".green());
        return Ok(());
    }

    // Version comparison (simple semver)
    if is_newer(&latest, current) {
        println!(
            "{}",
            format!("📦 New version available: {} → {}", current, latest).cyan()
        );
        println!();

        if check_only {
            println!("{}", "💡 Run 'agentd update' to install the update.".yellow());
            return Ok(());
        }

        // Perform update
        update_binary(&latest)?;
    } else {
        println!("{}", "✅ You're on a newer version than released!".green());
    }

    Ok(())
}

/// Get latest version from GitHub API
fn get_latest_version() -> Result<String> {
    let output = Command::new("curl")
        .args([
            "-fsSL",
            &format!("https://api.github.com/repos/{}/releases/latest", REPO),
        ])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to fetch latest version from GitHub");
    }

    let response = String::from_utf8_lossy(&output.stdout);

    // Parse tag_name from JSON response
    for line in response.lines() {
        if line.contains("\"tag_name\"") {
            // Extract version from "tag_name": "v0.1.0"
            if let Some(start) = line.find('"') {
                let rest = &line[start + 1..];
                if let Some(end) = rest.find('"') {
                    let rest = &rest[end + 1..];
                    if let Some(start) = rest.find('"') {
                        let rest = &rest[start + 1..];
                        if let Some(end) = rest.find('"') {
                            let version = &rest[..end];
                            // Remove 'v' prefix if present
                            return Ok(version.trim_start_matches('v').to_string());
                        }
                    }
                }
            }
        }
    }

    anyhow::bail!("Could not parse version from GitHub response")
}

/// Compare versions (simple semver comparison)
fn is_newer(new_version: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let new_parts = parse_version(new_version);
    let current_parts = parse_version(current);

    for (n, c) in new_parts.iter().zip(current_parts.iter()) {
        if n > c {
            return true;
        }
        if n < c {
            return false;
        }
    }

    new_parts.len() > current_parts.len()
}

/// Download and install the new binary
fn update_binary(version: &str) -> Result<()> {
    println!("{}", "📥 Downloading update...".cyan());

    // Detect platform
    let platform = detect_platform()?;
    println!("   Platform: {}", platform);

    // Use install.sh for the actual update
    let install_script = format!(
        "curl -fsSL https://raw.githubusercontent.com/{}/main/install.sh | VERSION=v{} bash",
        REPO, version
    );

    println!();
    println!("{}", "🔄 Installing update...".cyan());

    let status = Command::new("sh")
        .args(["-c", &install_script])
        .status()?;

    if !status.success() {
        anyhow::bail!("Update failed. Try running manually:\n   {}", install_script);
    }

    println!();
    println!("{}", "✅ Update complete!".green().bold());
    println!();
    println!("   Run 'agentd --version' to verify.");

    // If in an agentd project, suggest upgrading configs
    if std::path::Path::new("agentd").exists() {
        println!();
        println!("{}", "💡 To upgrade project configs:".yellow());
        println!("   agentd init --force");
    }

    Ok(())
}

/// Detect current platform
fn detect_platform() -> Result<String> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    let os_str = match os {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "windows",
        _ => anyhow::bail!("Unsupported OS: {}", os),
    };

    let arch_str = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => anyhow::bail!("Unsupported architecture: {}", arch),
    };

    Ok(format!("{}-{}", os_str, arch_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }
}
