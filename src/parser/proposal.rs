use crate::services::proposal_service::AffectedSpec;
use crate::Result;
use regex::Regex;

/// Parse affected_specs list from proposal.md
///
/// Looks for the "Affected specs:" line in the Impact section and extracts
/// the list of spec IDs with their dependencies.
///
/// # Supported formats in YAML frontmatter:
/// ```yaml
/// affected_specs:
///   - id: user-model
///     depends: []
///   - id: auth-flow
///     depends: [user-model]
/// ```
///
/// # Supported formats in markdown (for backward compatibility):
/// - Array notation: `["auth-flow", "user-model"]`
/// - Backtick list: `auth-flow`, `user-model`
/// - Comma-separated: auth-flow, user-model
///
/// Returns: Vec<AffectedSpec> with id and depends fields
pub fn parse_affected_specs(content: &str) -> Result<Vec<AffectedSpec>> {
    // Try to parse from YAML frontmatter first (new format with dependencies)
    // If frontmatter exists and has affected_specs (even if empty), use that result
    if let Some(specs) = parse_affected_specs_from_frontmatter(content) {
        return Ok(specs);
    }

    // Try XML format
    if let Ok(Some(proposal_block)) = crate::parser::extract_xml_block(content, "proposal") {
        return parse_affected_specs_from_markdown(&proposal_block.content);
    }

    // Fallback to old markdown format (backward compatibility)
    parse_affected_specs_from_markdown(content)
}

/// Parse affected specs from YAML frontmatter
fn parse_affected_specs_from_frontmatter(content: &str) -> Option<Vec<AffectedSpec>> {
    // Extract frontmatter between --- delimiters
    if !content.starts_with("---") {
        return None;
    }

    let end_idx = content[3..].find("---")?;
    let frontmatter = &content[3..end_idx + 3];

    // Parse affected_specs section
    let mut specs = Vec::new();
    let mut in_affected_specs = false;
    let mut current_id: Option<String> = None;
    let mut current_depends: Vec<String> = Vec::new();

    for line in frontmatter.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("affected_specs:") {
            in_affected_specs = true;
            continue;
        }

        if in_affected_specs {
            // Check if we've exited the affected_specs section
            if !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() && !trimmed.starts_with('-') {
                in_affected_specs = false;
                // Save last spec if any
                if let Some(id) = current_id.take() {
                    specs.push(AffectedSpec {
                        id,
                        depends: std::mem::take(&mut current_depends),
                    });
                }
                continue;
            }

            // Parse "- id: spec-name" lines
            if trimmed.starts_with("- id:") {
                // Save previous spec if any
                if let Some(id) = current_id.take() {
                    specs.push(AffectedSpec {
                        id,
                        depends: std::mem::take(&mut current_depends),
                    });
                }
                current_id = Some(trimmed.trim_start_matches("- id:").trim().to_string());
            }
            // Parse "depends: [dep1, dep2]" lines
            else if trimmed.starts_with("depends:") {
                let deps_str = trimmed.trim_start_matches("depends:").trim();
                let deps_str = deps_str.trim_matches(|c| c == '[' || c == ']');
                current_depends = deps_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }

    // Don't forget the last spec
    if let Some(id) = current_id {
        specs.push(AffectedSpec {
            id,
            depends: current_depends,
        });
    }

    Some(specs)
}

/// Topological sort of specs based on dependencies
///
/// Returns specs in dependency order (specs with no dependencies first).
/// Returns an error if there's a cycle in the dependency graph.
pub fn topological_sort_specs(specs: &[AffectedSpec]) -> Result<Vec<AffectedSpec>> {
    use std::collections::{HashMap, HashSet};

    // Build a map from id to spec
    let spec_map: HashMap<&str, &AffectedSpec> = specs.iter().map(|s| (s.id.as_str(), s)).collect();

    // Validate all dependencies exist
    for spec in specs {
        for dep in &spec.depends {
            if !spec_map.contains_key(dep.as_str()) {
                anyhow::bail!(
                    "Spec '{}' depends on '{}' which is not in affected_specs",
                    spec.id,
                    dep
                );
            }
        }
    }

    // Build adjacency list (id -> specs that depend on it)
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for spec in specs {
        in_degree.entry(spec.id.as_str()).or_insert(0);
        for dep in &spec.depends {
            dependents.entry(dep.as_str()).or_default().push(spec.id.as_str());
            *in_degree.entry(spec.id.as_str()).or_insert(0) += 1;
        }
    }

    // Kahn's algorithm for topological sort
    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut sorted = Vec::new();
    let mut visited = HashSet::new();

    while let Some(id) = queue.pop() {
        if visited.contains(id) {
            continue;
        }
        visited.insert(id);

        if let Some(spec) = spec_map.get(id) {
            sorted.push((*spec).clone());
        }

        if let Some(deps) = dependents.get(id) {
            for &dep_id in deps {
                let deg = in_degree.get_mut(dep_id).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(dep_id);
                }
            }
        }
    }

    // Check for cycles
    if sorted.len() != specs.len() {
        anyhow::bail!("Dependency cycle detected in specs");
    }

    Ok(sorted)
}

/// Parse affected specs from markdown content (backward compatibility)
fn parse_affected_specs_from_markdown(content: &str) -> Result<Vec<AffectedSpec>> {
    // Look for "Affected specs:" line in Impact section
    // Match patterns like:
    // - Affected specs: ["auth-flow", "user-model"]
    // - Affected specs: `auth-flow`, `user-model`
    // - Affected specs: auth-flow, user-model

    let re = Regex::new(r"(?mi)^[-*]\s*Affected specs:\s*(.+?)$")?;

    if let Some(caps) = re.captures(content) {
        let specs_str = caps.get(1).unwrap().as_str().trim();

        if specs_str.is_empty() || specs_str == "[]" || specs_str == "none" || specs_str.to_lowercase() == "n/a" {
            // No specs required
            return Ok(vec![]);
        }

        // Remove array brackets, backticks, quotes
        let cleaned = specs_str
            .trim_matches(|c| c == '[' || c == ']')
            .replace('`', "")
            .replace('"', "")
            .replace('\'', "");

        // Split by comma and clean up
        let specs: Vec<AffectedSpec> = cleaned
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && *s != "none" && s.to_lowercase() != "n/a")
            .map(|s| AffectedSpec {
                id: s.to_string(),
                depends: vec![],  // No dependency info in old format
            })
            .collect();

        if specs.is_empty() {
            // If found the line but got empty list after parsing, treat as error
            anyhow::bail!("Affected specs list is empty or invalid in proposal.md. Expected format: `spec-1`, `spec-2`");
        }

        Ok(specs)
    } else {
        anyhow::bail!(
            "Could not find 'Affected specs:' in proposal.md Impact section.\n\
            Expected format:\n\
            ## Impact\n\
            - Affected specs: `spec-1`, `spec-2`, `spec-3`"
        );
    }
}

/// Represents a review block from proposal.md
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewBlock {
    pub status: String,
    pub iteration: u32,
    pub reviewer: String,
    pub content: String,
}

/// Extract latest review from proposal.md
///
/// Parses all `<review>` blocks and returns the one with the highest iteration number.
///
/// # Returns
/// - `Ok(Some(ReviewBlock))` if at least one review is found
/// - `Ok(None)` if no reviews are found
/// - `Err` if parsing fails
pub fn parse_latest_review(content: &str) -> Result<Option<ReviewBlock>> {
    let reviews = crate::parser::extract_xml_blocks(content, "review")?;

    if reviews.is_empty() {
        return Ok(None);
    }

    // Get review with highest iteration number
    let latest = reviews
        .iter()
        .max_by_key(|r| {
            r.attributes
                .get("iteration")
                .and_then(|i| i.parse::<u32>().ok())
                .unwrap_or(0)
        })
        .unwrap();

    Ok(Some(ReviewBlock {
        status: latest
            .attributes
            .get("status")
            .cloned()
            .unwrap_or_default(),
        iteration: latest
            .attributes
            .get("iteration")
            .and_then(|i| i.parse::<u32>().ok())
            .unwrap_or(1),
        reviewer: latest
            .attributes
            .get("reviewer")
            .cloned()
            .unwrap_or_default(),
        content: latest.content.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(id: &str) -> AffectedSpec {
        AffectedSpec {
            id: id.to_string(),
            depends: vec![],
        }
    }

    fn spec_with_deps(id: &str, depends: Vec<&str>) -> AffectedSpec {
        AffectedSpec {
            id: id.to_string(),
            depends: depends.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_parse_affected_specs_backtick_format() {
        let proposal = r#"
# Change: add-oauth

## Summary
Add OAuth authentication

## Impact

- Scope: minor
- Affected specs: `auth-flow`, `user-model`, `api-endpoints`
- Affected code: src/auth/, src/models/
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, vec![spec("auth-flow"), spec("user-model"), spec("api-endpoints")]);
    }

    #[test]
    fn test_parse_affected_specs_array_format() {
        let proposal = r#"
## Impact
- Affected specs: ["auth-flow", "user-model"]
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, vec![spec("auth-flow"), spec("user-model")]);
    }

    #[test]
    fn test_parse_affected_specs_plain_format() {
        let proposal = r#"
## Impact
- Affected specs: auth-flow, user-model
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, vec![spec("auth-flow"), spec("user-model")]);
    }

    #[test]
    fn test_parse_affected_specs_empty() {
        let proposal = r#"
## Impact
- Affected specs: []
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, Vec::<AffectedSpec>::new());
    }

    #[test]
    fn test_parse_affected_specs_none() {
        let proposal = r#"
## Impact
- Affected specs: none
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, Vec::<AffectedSpec>::new());
    }

    #[test]
    fn test_parse_affected_specs_missing() {
        let proposal = r#"
## Impact
- Scope: minor
- Affected code: src/
"#;

        let result = parse_affected_specs(proposal);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Could not find"));
    }

    #[test]
    fn test_parse_affected_specs_single() {
        let proposal = r#"
## Impact
- Affected specs: `main-spec`
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, vec![spec("main-spec")]);
    }

    #[test]
    fn test_parse_affected_specs_from_frontmatter() {
        let proposal = r#"---
id: test-change
affected_specs:
  - id: user-model
    path: specs/user-model.md
    depends: []
  - id: auth-flow
    path: specs/auth-flow.md
    depends: [user-model]
  - id: api-endpoints
    path: specs/api-endpoints.md
    depends: [user-model, auth-flow]
---

<proposal>
## Summary
Test proposal
</proposal>
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs.len(), 3);
        assert_eq!(specs[0], spec("user-model"));
        assert_eq!(specs[1], spec_with_deps("auth-flow", vec!["user-model"]));
        assert_eq!(specs[2], spec_with_deps("api-endpoints", vec!["user-model", "auth-flow"]));
    }

    #[test]
    fn test_topological_sort() {
        let specs = vec![
            spec_with_deps("api-endpoints", vec!["user-model", "auth-flow"]),
            spec("user-model"),
            spec_with_deps("auth-flow", vec!["user-model"]),
        ];

        let sorted = topological_sort_specs(&specs).unwrap();
        assert_eq!(sorted[0].id, "user-model");
        assert_eq!(sorted[1].id, "auth-flow");
        assert_eq!(sorted[2].id, "api-endpoints");
    }

    #[test]
    fn test_topological_sort_cycle_detection() {
        let specs = vec![
            spec_with_deps("a", vec!["b"]),
            spec_with_deps("b", vec!["a"]),
        ];

        let result = topological_sort_specs(&specs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cycle"));
    }

    #[test]
    fn test_topological_sort_missing_dependency() {
        let specs = vec![
            spec_with_deps("a", vec!["nonexistent"]),
            spec("b"),
        ];

        let result = topological_sort_specs(&specs);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("depends on"));
        assert!(err.contains("nonexistent"));
        assert!(err.contains("not in affected_specs"));
    }
}
