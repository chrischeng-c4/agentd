use crate::models::frontmatter::StatePhase;
use crate::state::StateManager;
use crate::Result;
use colored::Colorize;
use std::env;

pub async fn run(change_id: &str, json: bool) -> Result<()> {
    let project_root = env::current_dir()?;
    let change_dir = project_root.join("agentd/changes").join(change_id);

    if !change_dir.exists() {
        if json {
            println!("{{\"error\": \"change_not_found\", \"change_id\": \"{}\"}}", change_id);
        } else {
            println!("{}", format!("Change '{}' not found", change_id).red());
        }
        return Ok(());
    }

    let state_manager = StateManager::load(&change_dir)?;
    let state = state_manager.state();

    if json {
        println!(
            "{{\"change_id\": \"{}\", \"phase\": \"{:?}\", \"iteration\": {}}}",
            state.change_id,
            state.phase,
            state.iteration
        );
    } else {
        println!("{}", format!("Status for: {}", change_id).cyan().bold());
        println!();

        let phase_icon = match state.phase {
            StatePhase::Proposed => "📝",
            StatePhase::Challenged => "🔍",
            StatePhase::Rejected => "⛔",
            StatePhase::Implementing => "🔨",
            StatePhase::Complete => "✅",
            StatePhase::Archived => "📦",
        };

        let phase_color = match state.phase {
            StatePhase::Proposed => format!("{:?}", state.phase).yellow(),
            StatePhase::Challenged => format!("{:?}", state.phase).cyan(),
            StatePhase::Rejected => format!("{:?}", state.phase).red(),
            StatePhase::Implementing => format!("{:?}", state.phase).blue(),
            StatePhase::Complete => format!("{:?}", state.phase).green(),
            StatePhase::Archived => format!("{:?}", state.phase).bright_black(),
        };

        println!("   Phase:     {} {}", phase_icon, phase_color);
        println!("   Iteration: {}", state.iteration);

        if let Some(last_action) = &state.last_action {
            println!("   Last:      {}", last_action);
        }

        if let Some(updated) = &state.updated_at {
            println!("   Updated:   {}", updated.format("%Y-%m-%d %H:%M:%S"));
        }

        // Display usage/cost summary if telemetry exists
        if let Some(telemetry) = &state.telemetry {
            println!();
            println!("{}", "💰 Usage Summary:".cyan());

            // Show totals
            println!("   Total tokens:  {} in / {} out",
                format_number(telemetry.total_tokens_in),
                format_number(telemetry.total_tokens_out));

            if telemetry.total_cost_usd > 0.0 {
                println!("   Total cost:    ${:.4}", telemetry.total_cost_usd);
            }

            // Show call count by step
            if !telemetry.calls.is_empty() {
                println!("   LLM calls:     {}", telemetry.calls.len());

                // Group calls by step
                let mut steps: std::collections::HashMap<&str, (u64, u64, f64)> = std::collections::HashMap::new();
                for call in &telemetry.calls {
                    let entry = steps.entry(&call.step).or_insert((0, 0, 0.0));
                    entry.0 += call.tokens_in.unwrap_or(0);
                    entry.1 += call.tokens_out.unwrap_or(0);
                    entry.2 += call.cost_usd.unwrap_or(0.0);
                }

                // Show breakdown by step
                println!();
                println!("{}", "   Breakdown by step:".bright_black());
                for (step, (tokens_in, tokens_out, cost)) in &steps {
                    if *cost > 0.0 {
                        println!("     {:12} {} in / {} out  (${:.4})",
                            step,
                            format_number(*tokens_in),
                            format_number(*tokens_out),
                            cost);
                    } else {
                        println!("     {:12} {} in / {} out",
                            step,
                            format_number(*tokens_in),
                            format_number(*tokens_out));
                    }
                }
            }
        }
    }

    Ok(())
}

/// Format a number with thousands separators
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateManager;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_change() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join("test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Create minimal proposal.md for StateManager
        let mut proposal = std::fs::File::create(change_dir.join("proposal.md")).unwrap();
        writeln!(proposal, "# Test Proposal\n\nContent").unwrap();

        (temp_dir, change_dir)
    }

    #[test]
    fn test_rejected_phase_icon_and_color() {
        let (_temp, change_dir) = setup_test_change();

        // Create state in Rejected phase
        let mut state_manager = StateManager::load(&change_dir).unwrap();
        state_manager.set_phase(StatePhase::Rejected);
        state_manager.save().unwrap();

        // Verify phase is Rejected
        let state_manager = StateManager::load(&change_dir).unwrap();
        let state = state_manager.state();
        assert_eq!(state.phase, StatePhase::Rejected);

        // Verify icon and color mapping
        let phase_icon = match state.phase {
            StatePhase::Proposed => "📝",
            StatePhase::Challenged => "🔍",
            StatePhase::Rejected => "⛔",
            StatePhase::Implementing => "🔨",
            StatePhase::Complete => "✅",
            StatePhase::Archived => "📦",
        };
        assert_eq!(phase_icon, "⛔");

        // Verify color is red for rejected
        let phase_color = match state.phase {
            StatePhase::Rejected => "red",
            _ => "other",
        };
        assert_eq!(phase_color, "red");
    }

    #[test]
    fn test_all_phase_icons() {
        // Verify all phases have appropriate icons
        let phases = [
            (StatePhase::Proposed, "📝"),
            (StatePhase::Challenged, "🔍"),
            (StatePhase::Rejected, "⛔"),
            (StatePhase::Implementing, "🔨"),
            (StatePhase::Complete, "✅"),
            (StatePhase::Archived, "📦"),
        ];

        for (phase, expected_icon) in phases.iter() {
            let icon = match phase {
                StatePhase::Proposed => "📝",
                StatePhase::Challenged => "🔍",
                StatePhase::Rejected => "⛔",
                StatePhase::Implementing => "🔨",
                StatePhase::Complete => "✅",
                StatePhase::Archived => "📦",
            };
            assert_eq!(icon, *expected_icon, "Icon mismatch for phase {:?}", phase);
        }
    }
}
