use crate::models::{
    Change, AgentdConfig, JsonValidationError, Severity, ValidationCounts,
    ValidationError, ValidationJsonOutput, ValidationOptions, ValidationResult, ValidationRules,
    frontmatter::ValidationMode,
};
use crate::parser::has_frontmatter;
use crate::state::StateManager;
use crate::validator::{AutoFixer, ConsistencyValidator, SchemaValidator, SpecFormatValidator, SemanticValidator};
use crate::Result;
use colored::Colorize;
use std::env;
use std::path::PathBuf;

/// Validation result summary
pub struct ValidationSummary {
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub errors: Vec<String>,
    pub validation_errors: Vec<ValidationError>,
    pub stale_files: Vec<String>,
}

impl ValidationSummary {
    /// Check if valid (no blocking errors)
    pub fn is_valid(&self) -> bool {
        self.high_count == 0
    }

    /// Check if valid with strict mode (no errors at all)
    pub fn is_valid_strict(&self) -> bool {
        self.high_count == 0 && self.medium_count == 0 && self.low_count == 0
    }

    pub fn has_warnings(&self) -> bool {
        self.medium_count > 0 || self.low_count > 0
    }

    /// Convert to JSON output format
    pub fn to_json_output(&self, strict: bool) -> ValidationJsonOutput {
        ValidationJsonOutput {
            valid: if strict { self.is_valid_strict() } else { self.is_valid() },
            counts: ValidationCounts {
                high: self.high_count,
                medium: self.medium_count,
                low: self.low_count,
            },
            errors: self.validation_errors.iter().map(JsonValidationError::from).collect(),
            stale_files: self.stale_files.clone(),
        }
    }
}

/// Helper struct to accumulate validation errors
struct ErrorAccumulator {
    high_count: usize,
    medium_count: usize,
    low_count: usize,
    errors: Vec<String>,
    validation_errors: Vec<ValidationError>,
}

impl ErrorAccumulator {
    fn new() -> Self {
        Self {
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            errors: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    fn process_result(
        &mut self,
        result: &ValidationResult,
        file_label: &str,
        indent: &str,
        options: &ValidationOptions,
    ) {
        for error in &result.errors {
            self.process_error(error, file_label, indent, options);
        }
    }

    fn process_error(
        &mut self,
        error: &ValidationError,
        file_label: &str,
        indent: &str,
        options: &ValidationOptions,
    ) {
        self.validation_errors.push(error.clone());
        match error.severity {
            Severity::High => {
                self.high_count += 1;
                self.errors.push(format!("[{}] {}", file_label, error.message));
                if !options.json {
                    print_error(error, indent, options.verbose);
                }
            }
            Severity::Medium => {
                self.medium_count += 1;
                if options.strict {
                    self.errors.push(format!("[{}] {}", file_label, error.message));
                }
                if !options.json {
                    print_error(error, indent, options.verbose);
                }
            }
            Severity::Low => {
                self.low_count += 1;
                if options.strict {
                    self.errors.push(format!("[{}] {}", file_label, error.message));
                }
                if !options.json {
                    print_error(error, indent, options.verbose);
                }
            }
        }
    }

    fn process_errors_slice(
        &mut self,
        errors: &[ValidationError],
        file_label: &str,
        indent: &str,
        options: &ValidationOptions,
    ) {
        for error in errors {
            self.process_error(error, file_label, indent, options);
        }
    }
}

/// Run validate-proposal command
pub async fn run(change_id: &str, options: &ValidationOptions) -> Result<()> {
    let project_root = env::current_dir()?;

    // For JSON output, suppress all other output
    if !options.json {
        println!(
            "{}",
            format!("🔍 Validating proposal: {}", change_id).cyan()
        );
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    }

    let mut summary = validate_proposal(change_id, &project_root, options)?;

    // If --fix is enabled and there are fixable errors, attempt to fix them
    if options.fix && summary.high_count > 0 {
        let fixable_count = summary.validation_errors.iter()
            .filter(|e| e.category.is_fixable())
            .count();

        if fixable_count > 0 {
            if !options.json {
                println!();
                println!("{}", "🔧 Attempting auto-fix...".cyan());
            }

            let fixer = AutoFixer::new(&project_root);
            let fix_result = fixer.fix_errors(&summary.validation_errors)?;

            if !options.json {
                for detail in &fix_result.fix_details {
                    println!("   ✓ {}", detail.green());
                }

                if fix_result.errors_fixed > 0 {
                    println!(
                        "   {} {} error(s) fixed in {} file(s)",
                        "✨".green(),
                        fix_result.errors_fixed,
                        fix_result.files_modified
                    );

                    // Re-run validation after fixes
                    println!();
                    println!("{}", "🔄 Re-validating after fixes...".cyan());
                    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());

                    // Create new options without fix to avoid infinite loop
                    let revalidate_options = ValidationOptions::new()
                        .with_strict(options.strict)
                        .with_verbose(options.verbose)
                        .with_json(false)
                        .with_fix(false);

                    summary = validate_proposal(change_id, &project_root, &revalidate_options)?;
                }

                if !fix_result.unfixable_errors.is_empty() {
                    println!(
                        "   {} {} error(s) could not be auto-fixed",
                        "⚠️".yellow(),
                        fix_result.unfixable_errors.len()
                    );
                }
            }
        }
    }

    // JSON output mode
    if options.json {
        let json_output = summary.to_json_output(options.strict);
        println!("{}", serde_json::to_string_pretty(&json_output)?);
        return Ok(());
    }

    // Determine if validation passed based on strict mode
    let passed = if options.strict {
        summary.is_valid_strict()
    } else {
        summary.is_valid()
    };

    if passed {
        println!();
        println!("{}", "✅ Proposal validation passed!".green().bold());

        if summary.has_warnings() && !options.strict {
            println!(
                "   {} warnings (MEDIUM: {}, LOW: {})",
                summary.medium_count + summary.low_count,
                summary.medium_count,
                summary.low_count
            );
        }

        println!();
        println!("{}", "⏭️  Next steps:".yellow());
        println!("   agentd challenge {}", change_id);
    } else {
        println!();
        println!("{}", "❌ Proposal validation failed!".red().bold());

        if options.strict {
            println!(
                "   {} errors in strict mode (HIGH: {}, MEDIUM: {}, LOW: {})",
                summary.high_count + summary.medium_count + summary.low_count,
                summary.high_count,
                summary.medium_count,
                summary.low_count
            );
        } else {
            println!(
                "   {} HIGH severity errors must be fixed before challenge",
                summary.high_count
            );
        }

        println!();
        println!("{}", "📝 Errors:".yellow());
        for error in &summary.errors {
            println!("   • {}", error);
        }
        println!();
        println!("   Fix the errors and run validation again:");
        println!("   agentd validate-proposal {}", change_id);
    }

    Ok(())
}

/// Validate a proposal and return summary (used by other commands)
pub fn validate_proposal(
    change_id: &str,
    project_root: &PathBuf,
    options: &ValidationOptions,
) -> Result<ValidationSummary> {
    // Load config for validation rules
    let config = AgentdConfig::load(project_root)?;

    // Check if change exists
    let change_dir = project_root.join("agentd/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Run 'agentd proposal {}' first.",
            change_id,
            change_id
        );
    }

    // Create Change object and validate structure
    let change = Change::new(change_id, "");
    change.validate_structure(project_root)?;

    // Create validators with type-specific rules
    // PRD validator for proposal.md (lenient rules)
    let prd_rules = ValidationRules::for_prd();
    let prd_format_validator = SpecFormatValidator::new(prd_rules);

    // Task validator for tasks.md (lenient rules)
    let task_rules = ValidationRules::for_task();
    let task_format_validator = SpecFormatValidator::new(task_rules);

    // Spec validator for specs/*.md (strict rules - use central format rules)
    let spec_rules = ValidationRules::for_spec();
    let spec_format_validator = SpecFormatValidator::new(spec_rules.clone());
    let semantic_validator = SemanticValidator::new(spec_rules);

    // Schema validator for frontmatter validation
    let schemas_dir = project_root.join("agentd/schemas");
    let mut schema_validator = if schemas_dir.exists() {
        Some(SchemaValidator::new(&schemas_dir))
    } else {
        if !options.json {
            println!(
                "   {}",
                "⚠ Schema directory not found, skipping frontmatter validation".yellow()
            );
        }
        None
    };

    let mut acc = ErrorAccumulator::new();

    // Validate proposal.md
    let proposal_path = change_dir.join("proposal.md");
    if proposal_path.exists() {
        if !options.json {
            println!("   Checking proposal.md...");
        }

        // Check if file has frontmatter
        let content = std::fs::read_to_string(&proposal_path).unwrap_or_default();
        let has_fm = has_frontmatter(&content);

        // Schema validation (only if frontmatter exists and schemas available)
        let mut schema_valid = true;
        if has_fm {
            if let Some(ref mut validator) = schema_validator {
                let schema_result = validator.validate_file(&proposal_path);
                if !schema_result.is_valid() {
                    schema_valid = false;
                    if !options.json {
                        println!("      {} Frontmatter schema validation:", "📋".cyan());
                    }
                    acc.process_result(&schema_result, "proposal.md", "      ", options);
                }
            }
        } else if !options.json && options.verbose {
            println!(
                "      {} No frontmatter found (optional for now)",
                "ℹ".bright_black()
            );
        }

        // Format validation (PRD rules - lenient)
        let result = prd_format_validator.validate(&proposal_path);
        acc.process_result(&result, "proposal.md", "      ", options);

        if result.is_valid() && schema_valid && !options.json {
            println!("      {}", "✓ OK".green());
        }
    }

    // Validate tasks.md
    let tasks_path = change_dir.join("tasks.md");
    if tasks_path.exists() {
        if !options.json {
            println!("   Checking tasks.md...");
        }

        // Check if file has frontmatter
        let content = std::fs::read_to_string(&tasks_path).unwrap_or_default();
        let has_fm = has_frontmatter(&content);

        // Schema validation
        let mut schema_valid = true;
        if has_fm {
            if let Some(ref mut validator) = schema_validator {
                let schema_result = validator.validate_file(&tasks_path);
                if !schema_result.is_valid() {
                    schema_valid = false;
                    if !options.json {
                        println!("      {} Frontmatter schema validation:", "📋".cyan());
                    }
                    acc.process_result(&schema_result, "tasks.md", "      ", options);
                }
            }
        } else if !options.json && options.verbose {
            println!(
                "      {} No frontmatter found (optional for now)",
                "ℹ".bright_black()
            );
        }

        // Format validation (Task rules - lenient)
        let result = task_format_validator.validate(&tasks_path);
        acc.process_result(&result, "tasks.md", "      ", options);

        if result.is_valid() && schema_valid && !options.json {
            println!("      {}", "✓ OK".green());
        }
    }

    // Validate spec files in specs/
    let specs_dir = change_dir.join("specs");
    if specs_dir.exists() {
        if !options.json {
            println!("   Checking specs/...");
        }
        let spec_files: Vec<_> = walkdir::WalkDir::new(&specs_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
            // Skip template/skeleton files (files starting with underscore)
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map_or(true, |name| !name.starts_with('_'))
            })
            .collect();

        for entry in spec_files {
            let rel_path = entry.path().strip_prefix(&change_dir).unwrap_or(entry.path());
            let file_label = rel_path.display().to_string();
            if !options.json {
                println!("      {}...", rel_path.display());
            }

            // Check if file has frontmatter
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let has_fm = has_frontmatter(&content);

            // Schema validation
            let mut schema_valid = true;
            if has_fm {
                if let Some(ref mut validator) = schema_validator {
                    let schema_result = validator.validate_file(entry.path());
                    if !schema_result.is_valid() {
                        schema_valid = false;
                        acc.process_result(&schema_result, &file_label, "         ", options);
                    }
                }
            }

            // Format validation (Spec rules - strict)
            let format_result = spec_format_validator.validate(entry.path());
            // Semantic validation
            let semantic_result = semantic_validator.validate(entry.path());

            // Combine and process errors
            let combined_errors: Vec<_> = format_result.errors.iter()
                .chain(semantic_result.errors.iter())
                .cloned()
                .collect();
            acc.process_errors_slice(&combined_errors, &file_label, "         ", options);

            if format_result.is_valid() && semantic_result.is_valid() && schema_valid && !options.json {
                println!("         {}", "✓ OK".green());
            }
        }
    }

    // Cross-file consistency validation
    if !options.json {
        println!("   Checking cross-file consistency...");
    }
    let consistency_validator = ConsistencyValidator::new(&change_dir);

    // Validate task spec_refs
    if let Ok(spec_ref_errors) = consistency_validator.validate_task_spec_refs() {
        acc.process_errors_slice(&spec_ref_errors, "consistency", "      ", options);
        if spec_ref_errors.is_empty() && !options.json {
            println!("      {} Task spec_refs", "✓".green());
        }
    }

    // Validate proposal spec alignment
    if let Ok(alignment_errors) = consistency_validator.validate_proposal_spec_alignment() {
        acc.process_errors_slice(&alignment_errors, "consistency", "      ", options);
        if alignment_errors.is_empty() && !options.json {
            println!("      {} Proposal spec alignment", "✓".green());
        }
    }

    // Validate task dependencies
    if let Ok(dep_errors) = consistency_validator.validate_task_dependencies() {
        acc.process_errors_slice(&dep_errors, "consistency", "      ", options);
        if dep_errors.is_empty() && !options.json {
            println!("      {} Task dependencies", "✓".green());
        }
    }

    if !options.json {
        println!();
        println!("{}", "📊 Summary:".cyan());
        println!(
            "   {} HIGH, {} MEDIUM, {} LOW",
            if acc.high_count > 0 { acc.high_count.to_string().red().to_string() } else { "0".to_string() },
            if acc.medium_count > 0 { acc.medium_count.to_string().yellow().to_string() } else { "0".to_string() },
            acc.low_count
        );
    }

    // Record validation to STATE.yaml
    let mut state_manager = StateManager::load(&change_dir)?;

    // Check staleness first
    let staleness = state_manager.check_staleness()?;
    let stale_files = staleness.stale_files.clone();

    if staleness.has_stale() && !options.json {
        println!();
        println!("{}", "⚠️  Stale files detected:".yellow());
        for file in &staleness.stale_files {
            println!("   • {} (modified since last validation)", file);
        }
    }

    // Record validation result
    let validation_mode = if options.strict {
        ValidationMode::Strict
    } else {
        ValidationMode::Normal
    };

    let warnings: Vec<String> = Vec::new();
    state_manager.record_validation(
        "validate-proposal",
        validation_mode,
        acc.high_count == 0,
        acc.high_count as u32,
        acc.medium_count as u32,
        acc.low_count as u32,
        acc.errors.clone(),
        warnings,
    );

    // Update checksums if validation passed
    if acc.high_count == 0 {
        state_manager.update_all_checksums()?;
        state_manager.set_last_action("validate-proposal");
    }

    // Save state
    state_manager.save()?;

    if !options.json {
        println!();
        println!(
            "   {} STATE.yaml updated",
            "💾".bright_black()
        );
    }

    Ok(ValidationSummary {
        high_count: acc.high_count,
        medium_count: acc.medium_count,
        low_count: acc.low_count,
        errors: acc.errors,
        validation_errors: acc.validation_errors,
        stale_files,
    })
}

/// Print a validation error with appropriate formatting
fn print_error(error: &ValidationError, indent: &str, verbose: bool) {
    let severity_label = match error.severity {
        Severity::High => "HIGH:".red(),
        Severity::Medium => "MEDIUM:".yellow(),
        Severity::Low => "LOW:".bright_black(),
    };

    if verbose {
        // Verbose output includes file path and line number
        let location = if let Some(line) = error.line {
            format!("{}:{}", error.file.display(), line)
        } else {
            error.file.display().to_string()
        };
        println!("{}{} {} ({})", indent, severity_label, error.message, location.bright_black());
    } else {
        println!("{}{} {}", indent, severity_label, error.message);
    }
}
