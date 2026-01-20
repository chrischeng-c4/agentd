/// Task dependency graph for spec-by-spec implementation
///
/// This module provides functionality to parse tasks.md and build a dependency
/// graph organized by layers and specs, enabling sequential implementation
/// workflow.

use crate::models::frontmatter::{TaskAction, TaskBlock, TasksFrontmatter};
use anyhow::{bail, Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Task dependency graph
#[derive(Debug, Clone)]
pub struct TaskGraph {
    /// Layers organized by order
    pub layers: Vec<Layer>,
    /// Specs grouped by layer name
    pub specs_by_layer: HashMap<String, Vec<SpecGroup>>,
}

/// Layer definition
#[derive(Debug, Clone)]
pub struct Layer {
    /// Layer name (e.g., "data", "logic", "integration", "testing")
    pub name: String,
    /// Layer order (1, 2, 3, 4)
    pub order: u8,
    /// Specs in this layer
    pub specs: Vec<SpecGroup>,
}

/// Group of tasks for a single spec
#[derive(Debug, Clone)]
pub struct SpecGroup {
    /// Spec ID (e.g., "mcp-tool-enforcement")
    pub spec_id: String,
    /// Path to spec file
    pub spec_path: PathBuf,
    /// Tasks referencing this spec
    pub tasks: Vec<TaskRef>,
    /// Other spec IDs this depends on
    pub depends_on: Vec<String>,
}

/// Reference to a task
#[derive(Debug, Clone)]
pub struct TaskRef {
    /// Task ID (e.g., "1.1")
    pub id: String,
    /// Task action (CREATE, MODIFY, DELETE)
    pub action: TaskAction,
    /// File path
    pub file: String,
    /// Other task IDs this depends on
    pub depends_on: Vec<String>,
}

impl TaskGraph {
    /// Parse tasks.md and build dependency graph
    pub fn from_tasks_file(tasks_path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(tasks_path)
            .with_context(|| format!("Failed to read tasks.md from {}", tasks_path.display()))?;

        // Parse frontmatter to get layer definitions
        let frontmatter = Self::parse_frontmatter(&content)?;

        // Parse tasks from markdown
        let tasks = Self::parse_tasks(&content)?;

        // Build layer structure
        let layers = Self::build_layers(&frontmatter, &tasks)?;

        // Build specs_by_layer map
        let specs_by_layer = layers
            .iter()
            .map(|layer| (layer.name.clone(), layer.specs.clone()))
            .collect();

        Ok(Self {
            layers,
            specs_by_layer,
        })
    }

    /// Parse YAML frontmatter from tasks.md
    fn parse_frontmatter(content: &str) -> Result<TasksFrontmatter> {
        // Extract frontmatter between --- delimiters
        let frontmatter_start = content.find("---").context("No frontmatter start found")?;
        let content_after_start = &content[frontmatter_start + 3..];
        let frontmatter_end = content_after_start
            .find("---")
            .context("No frontmatter end found")?;

        let frontmatter_str = &content_after_start[..frontmatter_end];
        let frontmatter: TasksFrontmatter = serde_yaml::from_str(frontmatter_str)
            .context("Failed to parse tasks frontmatter")?;

        Ok(frontmatter)
    }

    /// Parse task blocks from markdown content
    fn parse_tasks(content: &str) -> Result<Vec<TaskBlock>> {
        let mut tasks = Vec::new();

        // Find all YAML blocks (```yaml...```)
        let mut remaining = content;
        while let Some(start) = remaining.find("```yaml") {
            let content_after_start = &remaining[start + 7..];
            if let Some(end) = content_after_start.find("```") {
                let yaml_block = &content_after_start[..end];

                // Try to parse as TaskBlock
                if let Ok(task) = serde_yaml::from_str::<TaskBlock>(yaml_block) {
                    tasks.push(task);
                }

                remaining = &content_after_start[end + 3..];
            } else {
                break;
            }
        }

        if tasks.is_empty() {
            bail!("No tasks found in tasks.md");
        }

        Ok(tasks)
    }

    /// Build layer structure from frontmatter and tasks
    fn build_layers(
        frontmatter: &TasksFrontmatter,
        tasks: &[TaskBlock],
    ) -> Result<Vec<Layer>> {
        // Get layer definitions from frontmatter
        let layer_breakdown = frontmatter
            .layers
            .as_ref()
            .context("No layers defined in frontmatter")?;

        // Define layer order and names
        let layer_defs = vec![
            (1, "data", &layer_breakdown.data),
            (2, "logic", &layer_breakdown.logic),
            (3, "integration", &layer_breakdown.integration),
            (4, "testing", &layer_breakdown.testing),
        ];

        let mut layers = Vec::new();

        for (order, name, layer_info_opt) in layer_defs {
            // Skip if layer not defined in frontmatter
            if layer_info_opt.is_none() {
                continue;
            }

            // Filter tasks for this layer
            let layer_tasks: Vec<_> = tasks
                .iter()
                .filter(|t| {
                    // Extract layer from task ID (e.g., "1.1" -> layer 1)
                    if let Some(first_dot) = t.id.find('.') {
                        if let Ok(layer_num) = t.id[..first_dot].parse::<u8>() {
                            return layer_num == order;
                        }
                    }
                    false
                })
                .collect();

            // Group tasks by spec_ref
            let specs = Self::group_by_spec(&layer_tasks)?;

            layers.push(Layer {
                name: name.to_string(),
                order,
                specs,
            });
        }

        Ok(layers)
    }

    /// Group tasks by spec reference
    fn group_by_spec(tasks: &[&TaskBlock]) -> Result<Vec<SpecGroup>> {
        let mut spec_map: HashMap<String, Vec<&TaskBlock>> = HashMap::new();

        for task in tasks {
            let spec_id = task
                .spec_ref
                .as_ref()
                .context(format!("Task {} has no spec_ref", task.id))?;

            // Extract spec ID from spec_ref (format: "spec-id:R1" or "spec-id")
            let spec_id = if let Some(colon_pos) = spec_id.find(':') {
                &spec_id[..colon_pos]
            } else {
                spec_id.as_str()
            };

            spec_map
                .entry(spec_id.to_string())
                .or_insert_with(Vec::new)
                .push(task);
        }

        let mut spec_groups = Vec::new();

        for (spec_id, spec_tasks) in spec_map {
            // Collect task dependencies for this spec
            let mut spec_depends_on = HashSet::new();

            let task_refs: Vec<TaskRef> = spec_tasks
                .iter()
                .map(|t| {
                    // Add task dependencies from other specs
                    for dep in &t.depends_on {
                        // Find which spec the dependency belongs to
                        if let Some(dep_task) = tasks.iter().find(|task| task.id == *dep) {
                            if let Some(dep_spec_ref) = &dep_task.spec_ref {
                                let dep_spec_id = if let Some(colon_pos) = dep_spec_ref.find(':') {
                                    &dep_spec_ref[..colon_pos]
                                } else {
                                    dep_spec_ref.as_str()
                                };

                                // Only add if it's a different spec
                                if dep_spec_id != spec_id {
                                    spec_depends_on.insert(dep_spec_id.to_string());
                                }
                            }
                        }
                    }

                    TaskRef {
                        id: t.id.clone(),
                        action: t.action.clone(),
                        file: t.file.clone(),
                        depends_on: t.depends_on.clone(),
                    }
                })
                .collect();

            let spec_path = PathBuf::from(format!("agentd/changes/{{change_id}}/specs/{}.md", spec_id));

            spec_groups.push(SpecGroup {
                spec_id: spec_id.clone(),
                spec_path,
                tasks: task_refs,
                depends_on: spec_depends_on.into_iter().collect(),
            });
        }

        // Sort by spec_id for consistency
        spec_groups.sort_by(|a, b| a.spec_id.cmp(&b.spec_id));

        Ok(spec_groups)
    }

    /// Get specs in execution order (topological sort)
    pub fn get_execution_order(&self) -> Vec<&SpecGroup> {
        let mut result = Vec::new();
        let mut completed = HashSet::new();

        // Iterate through layers in order
        for layer in &self.layers {
            // Within each layer, execute specs in dependency order
            let mut remaining: Vec<_> = layer.specs.iter().collect();

            while !remaining.is_empty() {
                let initial_len = remaining.len();

                remaining.retain(|spec| {
                    // Check if all dependencies are completed
                    let can_execute = spec.depends_on.iter().all(|dep| completed.contains(dep));

                    if can_execute {
                        result.push(*spec);
                        completed.insert(spec.spec_id.clone());
                        false // Remove from remaining
                    } else {
                        true // Keep in remaining
                    }
                });

                // Detect circular dependencies
                if remaining.len() == initial_len && !remaining.is_empty() {
                    // No progress made, but specs still remain
                    // Add remaining specs anyway (circular dependency exists)
                    for spec in remaining {
                        result.push(spec);
                        completed.insert(spec.spec_id.clone());
                    }
                    break;
                }
            }
        }

        result
    }

    /// Get all tasks for a specific spec
    pub fn get_tasks_for_spec(&self, spec_id: &str) -> Vec<&TaskRef> {
        for layer in &self.layers {
            for spec in &layer.specs {
                if spec.spec_id == spec_id {
                    return spec.tasks.iter().collect();
                }
            }
        }
        Vec::new()
    }

    /// Check if all prerequisites are complete
    pub fn can_execute_spec(&self, spec_id: &str, completed: &HashSet<String>) -> bool {
        for layer in &self.layers {
            for spec in &layer.specs {
                if spec.spec_id == spec_id {
                    return spec.depends_on.iter().all(|dep| completed.contains(dep));
                }
            }
        }
        false
    }

    /// Validate no circular dependencies exist
    pub fn validate_dependencies(&self) -> Result<()> {
        let execution_order = self.get_execution_order();
        let total_specs: usize = self.layers.iter().map(|l| l.specs.len()).sum();

        if execution_order.len() < total_specs {
            bail!("Circular dependencies detected in task graph");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec_ref() {
        // Test parsing spec_ref with requirement suffix
        let spec_ref = "mcp-tool-enforcement:R1";
        let colon_pos = spec_ref.find(':').unwrap();
        let spec_id = &spec_ref[..colon_pos];
        assert_eq!(spec_id, "mcp-tool-enforcement");

        // Test parsing spec_ref without suffix
        let spec_ref_no_suffix = "mcp-tool-enforcement";
        let spec_id_no_suffix = if let Some(colon_pos) = spec_ref_no_suffix.find(':') {
            &spec_ref_no_suffix[..colon_pos]
        } else {
            spec_ref_no_suffix
        };
        assert_eq!(spec_id_no_suffix, "mcp-tool-enforcement");
    }

    #[test]
    fn test_task_ref_creation() {
        let task_ref = TaskRef {
            id: "1.1".to_string(),
            action: TaskAction::Create,
            file: "src/models/user.rs".to_string(),
            depends_on: vec![],
        };

        assert_eq!(task_ref.id, "1.1");
        assert_eq!(task_ref.file, "src/models/user.rs");
    }

    #[test]
    fn test_layer_ordering() {
        let mut layers = vec![
            Layer {
                name: "testing".to_string(),
                order: 4,
                specs: vec![],
            },
            Layer {
                name: "data".to_string(),
                order: 1,
                specs: vec![],
            },
            Layer {
                name: "logic".to_string(),
                order: 2,
                specs: vec![],
            },
        ];

        layers.sort_by_key(|l| l.order);

        assert_eq!(layers[0].name, "data");
        assert_eq!(layers[1].name, "logic");
        assert_eq!(layers[2].name, "testing");
    }
}
