use clap::{Parser, Subcommand};
use colored::Colorize;
use agentd::Result;

#[derive(Parser)]
#[command(name = "agentd")]
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

    /// Validate proposal format (local validation, no AI)
    ValidateProposal {
        /// Change ID to validate
        change_id: String,

        /// Treat warnings (MEDIUM/LOW) as errors
        #[arg(short, long)]
        strict: bool,

        /// Show verbose output with additional details
        #[arg(short, long)]
        verbose: bool,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Challenge the proposal with Codex (code analysis)
    Challenge {
        /// Change ID to challenge
        change_id: String,
    },

    /// Validate challenge format (local validation, no AI)
    ValidateChallenge {
        /// Change ID to validate
        change_id: String,

        /// Treat warnings (MEDIUM/LOW) as errors
        #[arg(short, long)]
        strict: bool,

        /// Show verbose output with additional details
        #[arg(short, long)]
        verbose: bool,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
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

    /// Implement the proposal with Claude (includes automatic review loop)
    Implement {
        /// Change ID to implement
        change_id: String,

        /// Filter specific tasks (e.g., "1.1,1.2,2.1")
        #[arg(short, long)]
        tasks: Option<String>,
    },

    /// Review implementation with Codex (run tests and code review)
    Review {
        /// Change ID to review
        change_id: String,
    },

    /// Resolve issues found during code review with Claude
    ResolveReviews {
        /// Change ID to resolve
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

    /// Initialize Agentd in current directory
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
            agentd::cli::proposal::run(&change_id, &description).await?;
        }

        Commands::ValidateProposal { change_id, strict, verbose, json } => {
            let options = agentd::models::ValidationOptions::new()
                .with_strict(strict)
                .with_verbose(verbose)
                .with_json(json);
            agentd::cli::validate_proposal::run(&change_id, &options).await?;
        }

        Commands::Challenge { change_id } => {
            println!(
                "{}",
                format!("🔍 Challenging proposal: {}", change_id).cyan()
            );
            agentd::cli::challenge_proposal::run(&change_id).await?;
        }

        Commands::ValidateChallenge { change_id, strict, verbose, json } => {
            let options = agentd::models::ValidationOptions::new()
                .with_strict(strict)
                .with_verbose(verbose)
                .with_json(json);
            agentd::cli::validate_challenge::run(&change_id, &options).await?;
        }

        Commands::Reproposal { change_id } => {
            println!("{}", format!("🤖 Reproposing: {}", change_id).cyan());
            agentd::cli::reproposal::run(&change_id).await?;
        }

        Commands::Refine {
            change_id,
            requirements,
        } => {
            println!("{}", format!("✨ Refining proposal: {}", change_id).cyan());
            agentd::cli::refine::run(&change_id, &requirements).await?;
        }

        Commands::Implement { change_id, tasks } => {
            // Implement command now includes automatic review loop
            agentd::cli::implement::run(&change_id, tasks.as_deref()).await?;
        }

        Commands::Review { change_id } => {
            println!("{}", format!("🔍 Reviewing: {}", change_id).cyan());
            agentd::cli::review::run(&change_id).await?;
        }

        Commands::ResolveReviews { change_id } => {
            println!("{}", format!("🔧 Resolving reviews: {}", change_id).cyan());
            agentd::cli::resolve_reviews::run(&change_id).await?;
        }

        Commands::Fix { change_id } => {
            println!("{}", format!("🔧 Fixing issues: {}", change_id).cyan());
            agentd::cli::fix::run(&change_id).await?;
        }

        Commands::Archive { change_id } => {
            println!("{}", format!("📦 Archiving: {}", change_id).cyan());
            agentd::cli::archive::run(&change_id).await?;
        }

        Commands::Init { name } => {
            println!("{}", "🚀 Initializing Agentd...".cyan());
            agentd::cli::init::run(name.as_deref()).await?;
        }

        Commands::Status { change_id, json } => {
            agentd::cli::status::run(&change_id, json).await?;
        }

        Commands::List { archived } => {
            agentd::cli::list::run(archived).await?;
        }
    }

    Ok(())
}
