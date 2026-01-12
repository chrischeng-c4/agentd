use clap::{Parser, Subcommand};
use colored::Colorize;
use specter::Result;

#[derive(Parser)]
#[command(name = "specter")]
#[command(author = "Chris Cheng <chris.cheng@shopee.com>")]
#[command(version = "0.1.0")]
#[command(about = "Spec-driven Development Orchestrator", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new proposal with Gemini (2M context)
    Proposal {
        /// Change ID (e.g., "add-oauth")
        change_id: String,

        /// Description of the change
        description: String,
    },

    /// Challenge the proposal with Codex (code analysis)
    Challenge {
        /// Change ID to challenge
        change_id: String,
    },

    /// Regenerate proposal based on challenge feedback
    Reproposal {
        /// Change ID to regenerate
        change_id: String,
    },

    /// Refine proposal with additional requirements
    Refine {
        /// Change ID to refine
        change_id: String,

        /// Additional requirements
        requirements: String,
    },

    /// Implement the proposal with Claude
    Implement {
        /// Change ID to implement
        change_id: String,

        /// Filter specific tasks (e.g., "1.1,1.2,2.1")
        #[arg(short, long)]
        tasks: Option<String>,
    },

    /// Verify implementation with Codex (generate and run tests)
    Verify {
        /// Change ID to verify
        change_id: String,
    },

    /// Fix issues found during verification with Claude
    Fix {
        /// Change ID to fix
        change_id: String,
    },

    /// Archive completed change
    Archive {
        /// Change ID to archive
        change_id: String,
    },

    /// Initialize Specter in current directory
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Show status of a change
    Status {
        /// Change ID to show status
        change_id: String,

        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// List all changes
    List {
        /// Show archived changes
        #[arg(short, long)]
        archived: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Proposal {
            change_id,
            description,
        } => {
            println!(
                "{}",
                "🤖 Generating proposal with Gemini (2M context)...".cyan()
            );
            specter::cli::proposal::run(&change_id, &description).await?;
        }

        Commands::Challenge { change_id } => {
            println!(
                "{}",
                format!("🔍 Challenging proposal: {}", change_id).cyan()
            );
            specter::cli::challenge::run(&change_id).await?;
        }

        Commands::Reproposal { change_id } => {
            println!("{}", format!("🤖 Reproposing: {}", change_id).cyan());
            specter::cli::reproposal::run(&change_id).await?;
        }

        Commands::Refine {
            change_id,
            requirements,
        } => {
            println!("{}", format!("✨ Refining proposal: {}", change_id).cyan());
            specter::cli::refine::run(&change_id, &requirements).await?;
        }

        Commands::Implement { change_id, tasks } => {
            println!("{}", format!("🎨 Implementing: {}", change_id).cyan());
            specter::cli::implement::run(&change_id, tasks.as_deref()).await?;
        }

        Commands::Verify { change_id } => {
            println!(
                "{}",
                format!("🧪 Verifying implementation: {}", change_id).cyan()
            );
            specter::cli::verify::run(&change_id).await?;
        }

        Commands::Fix { change_id } => {
            println!("{}", format!("🔧 Fixing issues: {}", change_id).cyan());
            specter::cli::fix::run(&change_id).await?;
        }

        Commands::Archive { change_id } => {
            println!("{}", format!("📦 Archiving: {}", change_id).cyan());
            specter::cli::archive::run(&change_id).await?;
        }

        Commands::Init { name } => {
            println!("{}", "🚀 Initializing Specter...".cyan());
            specter::cli::init::run(name.as_deref()).await?;
        }

        Commands::Status { change_id, json } => {
            specter::cli::status::run(&change_id, json).await?;
        }

        Commands::List { archived } => {
            specter::cli::list::run(archived).await?;
        }
    }

    Ok(())
}
