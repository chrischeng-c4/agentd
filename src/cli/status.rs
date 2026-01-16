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
            StatePhase::Implementing => "🔨",
            StatePhase::Testing => "🧪",
            StatePhase::Complete => "✅",
            StatePhase::Archived => "📦",
        };

        let phase_color = match state.phase {
            StatePhase::Proposed => format!("{:?}", state.phase).yellow(),
            StatePhase::Challenged => format!("{:?}", state.phase).cyan(),
            StatePhase::Implementing => format!("{:?}", state.phase).blue(),
            StatePhase::Testing => format!("{:?}", state.phase).magenta(),
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
    }

    Ok(())
}
