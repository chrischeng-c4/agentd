use crate::Result;
use colored::Colorize;
use semver::Version;
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

/// Compare versions using semver (fully compliant with Semantic Versioning 2.0.0)
///
/// This function compares two version strings according to the Semantic Versioning specification:
/// - Pre-release versions are lower than normal versions (1.0.0-alpha < 1.0.0)
/// - Pre-release identifiers are compared by parts: numeric < alphanumeric
/// - Examples: 1.0.0-alpha.1 < 1.0.0-alpha.beta < 1.0.0-beta.2 < 1.0.0
pub fn is_newer(new_version: &str, current: &str) -> bool {
    // Normalize versions (remove 'v' prefix if present)
    let new_ver = new_version.trim_start_matches('v');
    let curr_ver = current.trim_start_matches('v');

    // Parse versions using semver crate
    let new_parsed = match Version::parse(new_ver) {
        Ok(v) => v,
        Err(_) => {
            // If parsing fails, fall back to string comparison
            // This handles malformed versions gracefully
            return new_ver > curr_ver;
        }
    };

    let current_parsed = match Version::parse(curr_ver) {
        Ok(v) => v,
        Err(_) => {
            // If current version is malformed, consider new version as newer
            return true;
        }
    };

    // Use semver's built-in comparison (fully compliant with Semver 2.0.0)
    new_parsed > current_parsed
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
        // Basic version comparison
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));

        // Pre-release version comparison (Semver 2.0.0 compliant)
        assert!(is_newer("0.1.15", "0.1.15-alpha")); // Release > pre-release
        assert!(!is_newer("0.1.15-alpha", "0.1.15")); // Pre-release < release
        assert!(is_newer("0.1.15-beta", "0.1.15-alpha")); // beta > alpha
        assert!(is_newer("0.1.15-alpha", "0.1.14")); // 0.1.15-alpha > 0.1.14
        assert!(!is_newer("0.1.13", "0.1.15-alpha")); // 0.1.13 < 0.1.15-alpha

        // Equal versions
        assert!(!is_newer("0.1.15-alpha", "0.1.15-alpha")); // Same version
        assert!(is_newer("0.1.15-rc.1", "0.1.15-beta.1")); // rc > beta

        // Semver-compliant numeric comparison in pre-release
        assert!(is_newer("1.0.0-alpha.11", "1.0.0-alpha.2")); // Numeric: 11 > 2
        assert!(is_newer("1.0.0-alpha.beta", "1.0.0-alpha.1")); // Alphanumeric > numeric
        assert!(is_newer("1.0.0-beta.2", "1.0.0-beta")); // More identifiers > fewer

        // Version with 'v' prefix
        assert!(is_newer("v1.0.0", "v0.9.0"));
        assert!(is_newer("v1.0.0", "0.9.0")); // Mixed prefix handling
    }
}
