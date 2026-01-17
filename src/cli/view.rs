//! View command implementation
//!
//! Opens the plan viewer UI for a specified change.

use crate::ui::viewer::{start_viewer, ReviewResult};
use std::env;
use std::process;

/// Validate that a change_id is safe (no path traversal)
///
/// Rejects change_ids containing:
/// - Path separators (/ or \)
/// - Parent directory references (..)
/// - Empty strings
fn validate_change_id(change_id: &str) -> Result<(), String> {
    if change_id.is_empty() {
        return Err("change_id cannot be empty".to_string());
    }

    // Reject path separators
    if change_id.contains('/') || change_id.contains('\\') {
        return Err(format!(
            "Invalid change_id '{}': path separators not allowed",
            change_id
        ));
    }

    // Reject parent directory references
    if change_id.contains("..") {
        return Err(format!(
            "Invalid change_id '{}': parent directory references not allowed",
            change_id
        ));
    }

    Ok(())
}

/// Run the view command
///
/// Opens a native viewer window for the specified change.
/// This function blocks the main thread until the window is closed.
pub fn run(change_id: &str) {
    // Validate change_id before using it in path construction
    if let Err(e) = validate_change_id(change_id) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    let project_root = match env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: Failed to get current directory: {}", e);
            process::exit(1);
        }
    };

    // Validate change exists
    let changes_dir = project_root.join("agentd/changes");
    let change_dir = changes_dir.join(change_id);
    if !change_dir.exists() {
        eprintln!("Change '{}' not found", change_id);
        process::exit(1);
    }

    // Extra safety: canonicalize and verify the path is within agentd/changes/
    let canonical_change = match change_dir.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: Failed to resolve change path: {}", e);
            process::exit(1);
        }
    };

    let canonical_changes = match changes_dir.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: Failed to resolve changes directory: {}", e);
            process::exit(1);
        }
    };

    if !canonical_change.starts_with(&canonical_changes) {
        eprintln!(
            "Error: Invalid change_id '{}': resolves outside of changes directory",
            change_id
        );
        process::exit(1);
    }

    // Start the viewer (blocks until window is closed)
    match start_viewer(change_id, &project_root) {
        Ok(ReviewResult::Approved) => {
            println!("Review approved.");
            process::exit(0);
        }
        Ok(ReviewResult::ChangesRequested) => {
            println!("Changes requested. Run 'agentd revise {}' to address comments.", change_id);
            process::exit(2);
        }
        Ok(ReviewResult::Cancelled) => {
            println!("Review cancelled.");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: Failed to start viewer: {}", e);
            process::exit(1);
        }
    }
}

/// Spawn a detached viewer process
///
/// Used by the proposal workflow to auto-open the viewer without blocking.
pub fn spawn_detached(change_id: &str) -> std::io::Result<()> {
    use std::process::Command;

    // Get the current executable path
    let exe = env::current_exe()?;

    // Spawn detached process
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        Command::new(&exe)
            .arg("view")
            .arg(change_id)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .process_group(0) // Create new process group (detach)
            .spawn()?;
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;

        Command::new(&exe)
            .arg("view")
            .arg(change_id)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
            .spawn()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_change_directory_check() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join("agentd/changes/test-change");
        fs::create_dir_all(&change_dir).unwrap();

        // Change should exist
        assert!(change_dir.exists());

        // Non-existent change
        let missing = temp_dir.path().join("agentd/changes/missing");
        assert!(!missing.exists());
    }

    #[test]
    fn test_validate_change_id_valid() {
        assert!(validate_change_id("my-change").is_ok());
        assert!(validate_change_id("add-oauth").is_ok());
        assert!(validate_change_id("feature_123").is_ok());
        assert!(validate_change_id("v2.0.0").is_ok());
    }

    #[test]
    fn test_validate_change_id_rejects_empty() {
        assert!(validate_change_id("").is_err());
    }

    #[test]
    fn test_validate_change_id_rejects_path_traversal() {
        // Parent directory references
        assert!(validate_change_id("..").is_err());
        assert!(validate_change_id("../tmp").is_err());
        assert!(validate_change_id("foo/..").is_err());
        assert!(validate_change_id("foo/../bar").is_err());
    }

    #[test]
    fn test_validate_change_id_rejects_forward_slash() {
        assert!(validate_change_id("foo/bar").is_err());
        assert!(validate_change_id("/etc/passwd").is_err());
        assert!(validate_change_id("../secret").is_err());
    }

    #[test]
    fn test_validate_change_id_rejects_backslash() {
        assert!(validate_change_id("foo\\bar").is_err());
        assert!(validate_change_id("..\\secret").is_err());
        assert!(validate_change_id("C:\\Windows").is_err());
    }

    #[test]
    fn test_validate_change_id_error_messages() {
        let err = validate_change_id("../tmp").unwrap_err();
        assert!(err.contains("path separators not allowed") || err.contains("parent directory"));

        let err = validate_change_id("foo/bar").unwrap_err();
        assert!(err.contains("path separators not allowed"));

        let err = validate_change_id("").unwrap_err();
        assert!(err.contains("cannot be empty"));
    }
}
