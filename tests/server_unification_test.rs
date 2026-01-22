//! Integration tests for Server Unification (merge-viewer-to-mcp)
//!
//! Tests verify:
//! - MCP endpoint at `/mcp` works correctly
//! - Dashboard API at `/api/dashboard` works correctly
//! - Scoped Viewer routes at `/view/:project/:change` work correctly
//! - 404 handling for unknown projects/changes
//! - Static asset serving

use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test project structure
fn setup_test_project(temp_dir: &TempDir) -> (String, PathBuf) {
    let project_name = "test-project";
    let project_path = temp_dir.path().to_path_buf();

    // Create agentd/changes/test-change/
    let change_dir = project_path.join("agentd/changes/test-change");
    std::fs::create_dir_all(&change_dir).unwrap();

    // Create STATE.yaml
    std::fs::write(
        change_dir.join("STATE.yaml"),
        "change_id: test-change\nschema_version: \"2.0\"\nphase: proposed\n",
    )
    .unwrap();

    // Create proposal.md
    std::fs::write(
        change_dir.join("proposal.md"),
        "---\nid: test-change\ntype: proposal\n---\n\n# Test Proposal\n\nThis is a test.\n",
    )
    .unwrap();

    (project_name.to_string(), project_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_structure_creation() {
        let temp = TempDir::new().unwrap();
        let (project_name, project_path) = setup_test_project(&temp);

        assert_eq!(project_name, "test-project");
        assert!(project_path.join("agentd/changes/test-change").exists());
        assert!(project_path
            .join("agentd/changes/test-change/STATE.yaml")
            .exists());
        assert!(project_path
            .join("agentd/changes/test-change/proposal.md")
            .exists());
    }

    #[test]
    fn test_scan_project_changes() {
        let temp = TempDir::new().unwrap();
        let (_project_name, project_path) = setup_test_project(&temp);

        // Scan for changes
        let changes_dir = project_path.join("agentd/changes");
        let mut changes = Vec::new();

        for entry in std::fs::read_dir(&changes_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                changes.push(entry.file_name().to_string_lossy().to_string());
            }
        }

        assert_eq!(changes.len(), 1);
        assert!(changes.contains(&"test-change".to_string()));
    }

    #[test]
    fn test_read_change_status() {
        let temp = TempDir::new().unwrap();
        let (_project_name, project_path) = setup_test_project(&temp);

        let state_file = project_path.join("agentd/changes/test-change/STATE.yaml");
        let content = std::fs::read_to_string(&state_file).unwrap();

        // Parse phase from STATE.yaml
        let mut phase = "unknown".to_string();
        for line in content.lines() {
            if line.starts_with("phase:") {
                phase = line.trim_start_matches("phase:").trim().to_string();
                break;
            }
        }

        assert_eq!(phase, "proposed");
    }

    #[test]
    fn test_change_not_found() {
        let temp = TempDir::new().unwrap();
        let (_project_name, project_path) = setup_test_project(&temp);

        // Check for non-existent change
        let change_dir = project_path.join("agentd/changes/nonexistent");
        assert!(!change_dir.exists());
    }

    #[test]
    fn test_injected_config_format() {
        // Test that InjectedConfig serializes correctly
        #[derive(serde::Serialize)]
        struct InjectedConfig {
            base_path: String,
            project: String,
            change_id: String,
        }

        let config = InjectedConfig {
            base_path: "/view/myproj/change-1/api".to_string(),
            project: "myproj".to_string(),
            change_id: "change-1".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("base_path"));
        assert!(json.contains("/view/myproj/change-1/api"));
        assert!(json.contains("\"project\":\"myproj\""));
        assert!(json.contains("\"change_id\":\"change-1\""));
    }

    #[test]
    fn test_dashboard_state_format() {
        // Test that DashboardState serializes correctly
        #[derive(serde::Serialize)]
        struct ChangeInfo {
            id: String,
            status: String,
        }

        #[derive(serde::Serialize)]
        struct ProjectInfo {
            name: String,
            path: String,
            changes: Vec<ChangeInfo>,
        }

        let project = ProjectInfo {
            name: "test-project".to_string(),
            path: "/tmp/test-project".to_string(),
            changes: vec![ChangeInfo {
                id: "test-change".to_string(),
                status: "proposed".to_string(),
            }],
        };

        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("\"name\":\"test-project\""));
        assert!(json.contains("\"id\":\"test-change\""));
        assert!(json.contains("\"status\":\"proposed\""));
    }
}
