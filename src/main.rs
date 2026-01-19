use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::Colorize;
use agentd::Result;
use std::io;

#[derive(Parser)]
#[command(name = "agentd")]
#[command(author = "Chris Cheng <chris.cheng@shopee.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Spec-driven Development Orchestrator", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Orchestrate the entire planning phase (proposal → challenge → reproposal loop)
    Plan {
        /// Change ID
        change_id: String,

        /// Description (required for new changes, optional for existing)
        description: Option<String>,

        /// Skip clarifications.md check
        #[arg(long)]
        skip_clarify: bool,
    },

    /// Generate a new proposal with Gemini (2M context)
    Proposal {
        /// Change ID (e.g., "add-oauth")
        change_id: String,

        /// Description of the change
        description: String,

        /// Skip clarifications.md check (use when clarification is not needed)
        #[arg(long)]
        skip_clarify: bool,
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

        /// Auto-fix fixable errors (missing headings, WHEN/THEN)
        #[arg(short, long)]
        fix: bool,
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

        /// Auto-fix fixable errors (missing headings)
        #[arg(short, long)]
        fix: bool,
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

    /// Archive completed change
    Archive {
        /// Change ID to archive
        change_id: String,
    },

    /// Migrate files to XML format
    MigrateXml {
        /// Change ID to migrate (optional, migrates all if not specified)
        change_id: Option<String>,
    },

    /// Initialize Agentd in current directory
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,

        /// Force upgrade: update scripts and skills, preserve user data
        #[arg(short, long)]
        force: bool,
    },

    /// Show status of a change
    Status {
        /// Change ID to show status
        change_id: String,

        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// List all changes (for detailed archived view, use 'agentd archived')
    List {
        /// Show archived changes
        #[arg(short, long)]
        archived: bool,
    },

    /// Show detailed list of archived changes
    Archived,

    /// Update agentd to the latest version
    Update {
        /// Check for updates without installing
        #[arg(short, long)]
        check: bool,
    },

    /// Bootstrap Agentd specs from existing codebase using AST analysis
    Fillback {
        /// Path to source directory to analyze (default: current directory)
        #[arg(short, long)]
        path: Option<String>,

        /// Only generate spec for a specific module name
        #[arg(short, long)]
        module: Option<String>,

        /// Overwrite existing specs without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell type (bash, zsh, fish, powershell)
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Revise proposal based on review annotations
    Revise {
        /// Change ID to revise
        change_id: String,
    },

    /// Open plan viewer UI for a change (requires ui feature)
    #[cfg(feature = "ui")]
    View {
        /// Change ID to view
        change_id: String,
    },

    /// Start MCP server for structured proposal generation
    McpServer,
}

fn main() {
    let cli = Cli::parse();

    // Handle View command specially - it needs to run on the main thread
    // without initializing the tokio runtime first (tao/wry requirement on macOS)
    #[cfg(feature = "ui")]
    if let Commands::View { change_id } = &cli.command {
        agentd::cli::view::run(change_id);
        return;
    }

    // For all other commands, run with tokio runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    if let Err(e) = runtime.block_on(run_async(cli)) {
        eprintln!("{}", format!("Error: {}", e).red());
        std::process::exit(1);
    }
}

async fn run_async(cli: Cli) -> Result<()> {
    // Auto-upgrade check for all commands except init, completions, archived, and mcp-server
    let skip_upgrade = matches!(
        cli.command,
        Commands::Init { .. } | Commands::Completions { .. } | Commands::Archived | Commands::McpServer
    );

    #[cfg(feature = "ui")]
    let skip_upgrade = skip_upgrade || matches!(cli.command, Commands::View { .. });

    if !skip_upgrade {
        // Check for updates and auto-upgrade if available
        agentd::cli::init::check_and_auto_upgrade(true);
    }

    match cli.command {
        Commands::Plan {
            change_id,
            description,
            skip_clarify,
        } => {
            println!(
                "{}",
                "🤖 Orchestrating planning workflow...".cyan()
            );
            agentd::cli::plan::run(&change_id, description, skip_clarify).await?;
        }

        Commands::Proposal {
            change_id,
            description,
            skip_clarify,
        } => {
            println!(
                "{}",
                "🤖 Generating proposal with Gemini (2M context)...".cyan()
            );
            agentd::cli::proposal::run(&change_id, &description, skip_clarify).await?;
        }

        Commands::ValidateProposal { change_id, strict, verbose, json, fix } => {
            let options = agentd::models::ValidationOptions::new()
                .with_strict(strict)
                .with_verbose(verbose)
                .with_json(json)
                .with_fix(fix);
            agentd::cli::validate_proposal::run(&change_id, &options).await?;
        }

        Commands::Challenge { change_id } => {
            println!(
                "{}",
                format!("🔍 Challenging proposal: {}", change_id).cyan()
            );
            agentd::cli::challenge_proposal::run(&change_id).await?;
        }

        Commands::ValidateChallenge { change_id, strict, verbose, json, fix } => {
            let options = agentd::models::ValidationOptions::new()
                .with_strict(strict)
                .with_verbose(verbose)
                .with_json(json)
                .with_fix(fix);
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

        Commands::Archive { change_id } => {
            println!("{}", format!("📦 Archiving: {}", change_id).cyan());
            agentd::cli::archive::run(&change_id).await?;
        }

        Commands::MigrateXml { change_id } => {
            agentd::cli::migrate_xml::run(change_id.as_deref()).await?;
        }

        Commands::Init { name, force } => {
            if force {
                println!("{}", "🔄 Upgrading Agentd...".cyan());
            } else {
                println!("{}", "🚀 Initializing Agentd...".cyan());
            }
            agentd::cli::init::run(name.as_deref(), force).await?;
        }

        Commands::Status { change_id, json } => {
            agentd::cli::status::run(&change_id, json).await?;
        }

        Commands::List { archived } => {
            agentd::cli::list::run(archived)?;
        }

        Commands::Archived => {
            agentd::cli::list::run_archived_detailed()?;
        }

        Commands::Update { check } => {
            agentd::cli::update::run(check).await?;
        }

        Commands::Fillback {
            path,
            module,
            force,
        } => {
            agentd::cli::fillback::run(path.as_deref(), module.as_deref(), force).await?;
        }

        Commands::Completions { shell } => {
            generate(shell, &mut Cli::command(), "agentd", &mut io::stdout());
        }

        Commands::Revise { change_id } => {
            println!("{}", format!("📝 Reviewing annotations: {}", change_id).cyan());
            agentd::cli::revise::run(&change_id).await?;
        }

        // View command is handled before the async runtime is created
        #[cfg(feature = "ui")]
        Commands::View { .. } => {
            unreachable!("View command should be handled before runtime creation");
        }

        Commands::McpServer => {
            agentd::cli::mcp_server::run().await?;
        }
    }

    Ok(())
}
