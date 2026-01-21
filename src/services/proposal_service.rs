//! Proposal service - Business logic for proposal creation and management

use crate::Result;
use chrono::Utc;
use std::path::Path;

/// Input structure for creating a proposal
#[derive(Debug, Clone)]
pub struct CreateProposalInput {
    pub change_id: String,
    pub summary: String,
    pub why: String,
    pub what_changes: Vec<String>,
    pub impact: ImpactData,
}

/// Impact data for a proposal
#[derive(Debug, Clone)]
pub struct ImpactData {
    pub scope: String,
    pub affected_files: i64,
    pub new_files: i64,
    pub affected_specs: Vec<String>,
    pub affected_code: Vec<String>,
    pub breaking_changes: Option<String>,
}

/// Create a new proposal with validation
pub fn create_proposal(input: CreateProposalInput, project_root: &Path) -> Result<String> {
    // Validate change_id format
    if !input
        .change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("change_id must be lowercase alphanumeric with hyphens only");
    }

    // Validate summary length
    if input.summary.len() < 10 {
        anyhow::bail!("summary must be at least 10 characters");
    }

    // Validate why length
    if input.why.len() < 50 {
        anyhow::bail!("why must be at least 50 characters");
    }

    // Validate scope
    if !["patch", "minor", "major"].contains(&input.impact.scope.as_str()) {
        anyhow::bail!("impact.scope must be 'patch', 'minor', or 'major'");
    }

    // Create change directory
    let change_dir = project_root.join("agentd/changes").join(&input.change_id);
    std::fs::create_dir_all(&change_dir)?;

    // Generate proposal.md content
    let now = Utc::now();
    let mut content = String::new();

    // Frontmatter
    content.push_str("---\n");
    content.push_str(&format!("id: {}\n", input.change_id));
    content.push_str("type: proposal\n");
    content.push_str("version: 1\n");
    content.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    content.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));
    content.push_str("author: mcp\n");
    content.push_str("status: proposed\n");
    content.push_str("iteration: 1\n");
    content.push_str(&format!(
        "summary: \"{}\"\n",
        input.summary.replace('"', "\\\"")
    ));

    // Impact section in frontmatter
    content.push_str("impact:\n");
    content.push_str(&format!("  scope: {}\n", input.impact.scope));
    content.push_str(&format!("  affected_files: {}\n", input.impact.affected_files));
    content.push_str(&format!("  new_files: {}\n", input.impact.new_files));

    // Affected specs
    if !input.impact.affected_specs.is_empty() {
        content.push_str("affected_specs:\n");
        for spec_id in &input.impact.affected_specs {
            content.push_str(&format!("  - id: {}\n", spec_id));
            content.push_str(&format!("    path: specs/{}.md\n", spec_id));
        }
    }

    content.push_str("---\n\n");

    // Wrap proposal content in XML
    content.push_str("<proposal>\n\n");

    // Title
    content.push_str(&format!("# Change: {}\n\n", input.change_id));

    // Summary section
    content.push_str("## Summary\n\n");
    content.push_str(&format!("{}\n\n", input.summary));

    // Why section
    content.push_str("## Why\n\n");
    content.push_str(&format!("{}\n\n", input.why));

    // What Changes section
    content.push_str("## What Changes\n\n");
    for change in &input.what_changes {
        content.push_str(&format!("- {}\n", change));
    }
    content.push('\n');

    // Impact section in markdown
    content.push_str("## Impact\n\n");
    content.push_str(&format!("- **Scope**: {}\n", input.impact.scope));
    content.push_str(&format!(
        "- **Affected Files**: ~{}\n",
        input.impact.affected_files
    ));
    content.push_str(&format!("- **New Files**: ~{}\n", input.impact.new_files));

    if !input.impact.affected_specs.is_empty() {
        let specs_list: Vec<String> = input
            .impact
            .affected_specs
            .iter()
            .map(|id| format!("`{}`", id))
            .collect();
        content.push_str(&format!("- Affected specs: {}\n", specs_list.join(", ")));
    }

    if !input.impact.affected_code.is_empty() {
        let code_list: Vec<String> = input
            .impact
            .affected_code
            .iter()
            .map(|path| format!("`{}`", path))
            .collect();
        content.push_str(&format!("- Affected code: {}\n", code_list.join(", ")));
    }

    if let Some(breaking) = &input.impact.breaking_changes {
        content.push_str(&format!("- **Breaking Changes**: {}\n", breaking));
    }
    content.push('\n');

    // Close proposal XML tag
    content.push_str("</proposal>\n");

    // Write the file
    let proposal_path = change_dir.join("proposal.md");
    std::fs::write(&proposal_path, &content)?;

    // Create specs directory
    let specs_dir = change_dir.join("specs");
    std::fs::create_dir_all(&specs_dir)?;

    // Initialize STATE.yaml
    let state_content = format!(
        r#"change_id: {}
schema_version: "2.0"
created_at: {}
updated_at: {}
phase: proposed
iteration: 1
last_action: create_proposal (mcp)
checksums: {{}}
validations: []
"#,
        input.change_id,
        now.to_rfc3339(),
        now.to_rfc3339()
    );
    std::fs::write(change_dir.join("STATE.yaml"), state_content)?;

    Ok(format!(
        "Created proposal.md for change '{}' at {}",
        input.change_id,
        proposal_path.display()
    ))
}

/// Append review block to existing proposal.md
pub fn append_review(
    proposal_path: &Path,
    status: &str,
    iteration: u32,
    reviewer: &str,
    review_content: &str,
) -> Result<()> {
    let content = std::fs::read_to_string(proposal_path)?;

    // Create attributes map
    let mut attrs = std::collections::HashMap::new();
    attrs.insert("status".to_string(), status.to_string());
    attrs.insert("iteration".to_string(), iteration.to_string());
    attrs.insert("reviewer".to_string(), reviewer.to_string());

    // Wrap review content in XML tags
    let review_xml = crate::parser::wrap_in_xml("review", review_content, attrs);

    // Check existing reviews
    let existing_reviews = crate::parser::extract_xml_blocks(&content, "review")?;

    // Check if this iteration exists and is resolved
    let should_replace = existing_reviews.iter().any(|r| {
        let iter = r
            .attributes
            .get("iteration")
            .and_then(|i| i.parse::<u32>().ok())
            .unwrap_or(0);
        let r_status = r.attributes.get("status").map(|s| s.as_str()).unwrap_or("");
        iter == iteration && r_status == "resolved"
    });

    let updated = if should_replace {
        crate::parser::update_xml_blocks(
            &content,
            "review",
            &review_xml,
            crate::parser::UpdateMode::ReplaceLatest,
        )?
    } else {
        crate::parser::update_xml_blocks(
            &content,
            "review",
            &review_xml,
            crate::parser::UpdateMode::Append,
        )?
    };

    std::fs::write(proposal_path, updated)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create agentd directory structure
        std::fs::create_dir_all(project_root.join("agentd/changes")).unwrap();

        let input = CreateProposalInput {
            change_id: "test-change".to_string(),
            summary: "This is a test change with sufficient length".to_string(),
            why: "This change is needed because we want to test the service functionality and ensure it works correctly".to_string(),
            what_changes: vec![
                "Add new feature X".to_string(),
                "Modify existing module Y".to_string(),
            ],
            impact: ImpactData {
                scope: "minor".to_string(),
                affected_files: 5,
                new_files: 2,
                affected_specs: vec!["test-spec".to_string()],
                affected_code: vec!["src/services/".to_string()],
                breaking_changes: None,
            },
        };

        let result = create_proposal(input, project_root).unwrap();
        assert!(result.contains("Created proposal.md"));

        // Verify file was created
        let proposal_path = project_root.join("agentd/changes/test-change/proposal.md");
        assert!(proposal_path.exists());

        let content = std::fs::read_to_string(&proposal_path).unwrap();
        assert!(content.contains("id: test-change"));
        assert!(content.contains("## Summary"));
        assert!(content.contains("## Why"));
        assert!(content.contains("## What Changes"));
    }

    #[test]
    fn test_create_proposal_validation() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Test invalid change_id
        let input = CreateProposalInput {
            change_id: "Invalid_ID".to_string(),
            summary: "Test summary with sufficient length".to_string(),
            why: "Test why explanation that is long enough to pass validation".to_string(),
            what_changes: vec!["Change 1".to_string()],
            impact: ImpactData {
                scope: "patch".to_string(),
                affected_files: 1,
                new_files: 0,
                affected_specs: vec![],
                affected_code: vec![],
                breaking_changes: None,
            },
        };

        let result = create_proposal(input, project_root);
        assert!(result.is_err());
    }
}
