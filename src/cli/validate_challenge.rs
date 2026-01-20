use crate::models::{
    ChallengeVerdict, AgentdConfig, ValidationCounts, ValidationOptions,
};
use crate::parser::parse_challenge_verdict;
use crate::state::StateManager;
use crate::Result;
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Challenge validation result
pub struct ChallengeValidationResult {
    pub verdict: ChallengeVerdict,
    pub has_valid_structure: bool,
    pub issue_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub errors: Vec<String>,
}

impl ChallengeValidationResult {
    pub fn is_valid(&self) -> bool {
        self.has_valid_structure && self.verdict != ChallengeVerdict::Unknown
    }

    /// Convert to JSON output format
    pub fn to_json_output(&self) -> ChallengeJsonOutput {
        ChallengeJsonOutput {
            valid: self.is_valid(),
            verdict: format!("{:?}", self.verdict).to_uppercase(),
            counts: ValidationCounts {
                high: self.high_count,
                medium: self.medium_count,
                low: self.low_count,
            },
            issue_count: self.issue_count,
            errors: self.errors.clone(),
        }
    }
}

/// JSON output format for challenge validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeJsonOutput {
    pub valid: bool,
    pub verdict: String,
    pub counts: ValidationCounts,
    pub issue_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

/// Run validate-challenge command
pub async fn run(change_id: &str, options: &ValidationOptions) -> Result<()> {
    let project_root = env::current_dir()?;

    if !options.json {
        println!(
            "{}",
            format!("🔍 Validating challenge: {}", change_id).cyan()
        );
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    }

    let result = validate_challenge(change_id, &project_root, options)?;

    // JSON output mode
    if options.json {
        let json_output = result.to_json_output();
        println!("{}", serde_json::to_string_pretty(&json_output)?);
        return Ok(());
    }

    println!();
    println!("{}", "📊 Summary:".cyan());

    if result.is_valid() {
        println!("{}", "✅ Challenge format validation passed!".green().bold());
        println!();
        println!("   Verdict: {}", format_verdict(&result.verdict));
        println!(
            "   Issues: {} total ({} HIGH, {} MEDIUM, {} LOW)",
            result.issue_count,
            result.high_count,
            result.medium_count,
            result.low_count
        );

        println!();
        match result.verdict {
            ChallengeVerdict::Approved => {
                println!("{}", "⏭️  Next steps:".yellow());
                println!("   agentd implement {}", change_id);
            }
            ChallengeVerdict::NeedsRevision => {
                println!("{}", "⏭️  Next steps:".yellow());
                println!("   agentd reproposal {}", change_id);
            }
            ChallengeVerdict::Rejected => {
                println!("{}", "⚠️  Proposal was rejected. Review and recreate:".yellow());
                println!("   cat agentd/changes/{}/CHALLENGE.md", change_id);
            }
            ChallengeVerdict::Unknown => {}
        }
    } else {
        println!("{}", "❌ Challenge format validation failed!".red().bold());
        println!();
        println!("{}", "📝 Errors:".yellow());
        for error in &result.errors {
            println!("   • {}", error);
        }
        println!();
        println!("   Re-run challenge to regenerate CHALLENGE.md:");
        println!("   agentd challenge {}", change_id);
    }

    Ok(())
}

/// Validate challenge result and return structured result (used by other commands)
pub fn validate_challenge(
    change_id: &str,
    project_root: &PathBuf,
    options: &ValidationOptions,
) -> Result<ChallengeValidationResult> {
    // Load config
    let _config = AgentdConfig::load(project_root)?;

    // Check if change exists
    let change_dir = project_root.join("agentd/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Run 'agentd proposal {}' first.",
            change_id,
            change_id
        );
    }

    // Check proposal.md for review blocks (new format only)
    let proposal_path = change_dir.join("proposal.md");

    if !proposal_path.exists() {
        return Ok(ChallengeValidationResult {
            verdict: ChallengeVerdict::Unknown,
            has_valid_structure: false,
            issue_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            errors: vec!["proposal.md not found.".to_string()],
        });
    }

    if !options.json {
        println!("   Checking review in proposal.md...");
    }

    let proposal_content = std::fs::read_to_string(&proposal_path)?;
    let latest_review = crate::parser::parse_latest_review(&proposal_content)?;

    let content = match latest_review {
        Some(review) => review.content,
        None => {
            return Ok(ChallengeValidationResult {
                verdict: ChallengeVerdict::Unknown,
                has_valid_structure: false,
                issue_count: 0,
                high_count: 0,
                medium_count: 0,
                low_count: 0,
                errors: vec!["No review found in proposal.md. Run 'agentd challenge' first.".to_string()],
            });
        }
    };

    let mut errors = Vec::new();
    let mut has_valid_structure = true;

    // Parse verdict from proposal.md
    let verdict = parse_challenge_verdict(&proposal_path).unwrap_or(ChallengeVerdict::Unknown);
    if !options.json {
        println!("   Checking verdict...");
    }

    if verdict == ChallengeVerdict::Unknown {
        has_valid_structure = false;
        errors.push("Could not parse verdict. Expected: APPROVED, NEEDS_REVISION, or REJECTED".to_string());
        if !options.json {
            println!("      {} Could not parse verdict", "HIGH:".red());
        }
    } else if !options.json {
        println!("      {} Verdict: {:?}", "✓".green(), verdict);
    }

    // Count issues
    let high_count = content.matches("**Severity**: High").count();
    let medium_count = content.matches("**Severity**: Medium").count();
    let low_count = content.matches("**Severity**: Low").count();
    let issue_count = high_count + medium_count + low_count;

    if !options.json {
        println!("   Checking issue format...");
    }

    // Validate issue format (each issue should have required fields)
    // Support both old format "#### Issue \d+" and new format "### Issue:"
    let issue_regex = Regex::new(r"###\s+Issue[:\s]").ok();
    if let Some(re) = issue_regex {
        let issue_headers: Vec<_> = re.find_iter(&content).collect();

        for (i, m) in issue_headers.iter().enumerate() {
            let issue_num = i + 1;
            let issue_start = m.start();
            let issue_end = if i + 1 < issue_headers.len() {
                issue_headers[i + 1].start()
            } else {
                // Find next ## section or end of content
                content[issue_start + 1..]
                    .find("\n## ")
                    .map(|pos| issue_start + 1 + pos)
                    .unwrap_or(content.len())
            };
            let issue_content = &content[issue_start..issue_end];

            // Check required fields (note: **Severity** may have - prefix)
            let has_severity = issue_content.contains("**Severity**:");
            let has_description = issue_content.contains("**Description**:");
            let has_location = issue_content.contains("**Location**:");

            if !has_severity {
                errors.push(format!("Issue {} missing field: Severity", issue_num));
                if !options.json {
                    println!("      {} Issue {} missing: Severity", "MEDIUM:".yellow(), issue_num);
                }
            }
            if !has_description {
                errors.push(format!("Issue {} missing field: Description", issue_num));
                if !options.json {
                    println!("      {} Issue {} missing: Description", "MEDIUM:".yellow(), issue_num);
                }
            }
            // Location is optional for some issues
            if !has_location && options.verbose {
                if !options.json {
                    println!("      {} Issue {} missing optional: Location", "LOW:".bright_black(), issue_num);
                }
            }
        }

        if issue_headers.is_empty() && verdict == ChallengeVerdict::NeedsRevision {
            errors.push("NEEDS_REVISION verdict but no issues documented".to_string());
            if !options.json {
                println!("      {} NEEDS_REVISION but no issues found", "MEDIUM:".yellow());
            }
        }
    }

    if has_valid_structure && errors.is_empty() && !options.json {
        println!("      {}", "✓ OK".green());
    }

    // Record validation to STATE.yaml
    let mut state_manager = StateManager::load(&change_dir)?;

    // Record challenge validation
    let verdict_str = match verdict {
        ChallengeVerdict::Approved => "APPROVED",
        ChallengeVerdict::NeedsRevision => "NEEDS_REVISION",
        ChallengeVerdict::Rejected => "REJECTED",
        ChallengeVerdict::Unknown => "UNKNOWN",
    };

    state_manager.record_challenge_validation(
        verdict_str,
        issue_count as u32,
        high_count as u32,
        medium_count as u32,
        low_count as u32,
    );

    // Update phase based on verdict
    state_manager.update_phase_from_verdict(verdict_str);

    // Update proposal.md checksum (contains review blocks)
    state_manager.update_checksum("proposal.md")?;
    state_manager.set_last_action("validate-challenge");

    // Save state
    state_manager.save()?;

    if !options.json {
        println!();
        println!(
            "   {} STATE.yaml updated",
            "💾".bright_black()
        );
    }

    Ok(ChallengeValidationResult {
        verdict,
        has_valid_structure,
        issue_count,
        high_count,
        medium_count,
        low_count,
        errors,
    })
}

fn format_verdict(verdict: &ChallengeVerdict) -> String {
    match verdict {
        ChallengeVerdict::Approved => "APPROVED".green().bold().to_string(),
        ChallengeVerdict::NeedsRevision => "NEEDS_REVISION".yellow().bold().to_string(),
        ChallengeVerdict::Rejected => "REJECTED".red().bold().to_string(),
        ChallengeVerdict::Unknown => "UNKNOWN".bright_black().to_string(),
    }
}
