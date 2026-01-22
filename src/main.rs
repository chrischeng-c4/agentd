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
    /// Orchestrate the entire planning phase (proposal → review → revise loop)
    #[command(name = "plan-change")]
    PlanChange {
        /// Change ID
        change_id: String,

        /// Description (required for new changes, optional for existing)
        description: Option<String>,

        /// Skip clarifications.md check
        #[arg(long)]
        skip_clarify: bool,
    },

    /// Refine proposal with additional requirements
    Refine {
        /// Change ID to refine
        change_id: String,

        /// Additional requirements
        requirements: String,
    },

    /// Implement the proposal with Claude (includes automatic review loop)
    #[command(name = "impl-change")]
    ImplChange {
        /// Change ID to implement
        change_id: String,

        /// Filter specific tasks (e.g., "1.1,1.2,2.1")
        #[arg(short, long)]
        tasks: Option<String>,
    },

    /// Merge completed change specs to main agentd/specs/
    #[command(name = "merge-change")]
    MergeChange {
        /// Change ID to merge
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

    /// Unified server management (dashboard + MCP + viewer)
    #[command(subcommand)]
    Server(agentd::cli::server::ServerCommands),

    /// [DEPRECATED] Use 'server' instead. MCP server management.
    #[command(subcommand, hide = true)]
    McpServer(agentd::cli::mcp_server_mgmt::McpServerCommands),

    /// Knowledge base operations
    #[command(subcommand)]
    Knowledge(agentd::cli::knowledge::KnowledgeCommands),

    /// Spec operations
    #[command(subcommand)]
    Spec(agentd::cli::spec::SpecCommands),

    /// File operations
    #[command(subcommand)]
    File(agentd::cli::file::FileCommands),

    /// Proposal operations
    #[command(subcommand)]
    Proposal(agentd::cli::proposal::ProposalCommands),

    /// Tasks operations
    #[command(subcommand)]
    Tasks(agentd::cli::tasks::TasksCommands),

    /// Implementation workflow commands
    #[command(subcommand)]
    Implementation(agentd::cli::implementation::ImplementationCommands),

    /// Create clarifications from Q&A
    Clarifications(agentd::cli::clarifications::ClarificationsArgs),
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
    // Auto-upgrade check for all commands except init, completions, archived, server, and CLI utility commands
    let skip_upgrade = matches!(
        cli.command,
        Commands::Init { .. } | Commands::Completions { .. } | Commands::Archived | Commands::Server(_) | Commands::McpServer(_) | Commands::Knowledge(_) | Commands::Spec(_) | Commands::File(_) | Commands::Proposal(_) | Commands::Tasks(_) | Commands::Implementation(_) | Commands::Clarifications(_)
    );

    #[cfg(feature = "ui")]
    let skip_upgrade = skip_upgrade || matches!(cli.command, Commands::View { .. });

    if !skip_upgrade {
        // Check for updates and auto-upgrade if available
        agentd::cli::init::check_and_auto_upgrade(true);
    }

    match cli.command {
        Commands::PlanChange {
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

        Commands::Refine {
            change_id,
            requirements,
        } => {
            println!("{}", format!("✨ Refining proposal: {}", change_id).cyan());
            agentd::cli::refine::run(&change_id, &requirements).await?;
        }

        Commands::ImplChange { change_id, tasks } => {
            // Implement command now includes automatic review loop
            let _result = agentd::cli::implement::run(&change_id, tasks.as_deref()).await?;
            // Result is used by skills for HITL decisions, CLI just needs success/error
        }

        Commands::MergeChange { change_id } => {
            println!("{}", format!("📦 Merging change: {}", change_id).cyan());
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

        Commands::Server(cmd) => {
            agentd::cli::server::run(cmd).await?;
        }

        Commands::McpServer(cmd) => {
            eprintln!("{}", "⚠ 'mcp-server' is deprecated. Use 'server' instead.".yellow());
            agentd::cli::mcp_server_mgmt::run(cmd).await?;
        }

        Commands::Knowledge(cmd) => {
            agentd::cli::knowledge::run(cmd)?;
        }

        Commands::Spec(cmd) => {
            agentd::cli::spec::run(cmd)?;
        }

        Commands::File(cmd) => {
            agentd::cli::file::run(cmd)?;
        }

        Commands::Proposal(cmd) => {
            agentd::cli::proposal::run(cmd)?;
        }

        Commands::Tasks(cmd) => {
            agentd::cli::tasks::run(cmd)?;
        }

        Commands::Implementation(cmd) => {
            agentd::cli::implementation::run(cmd)?;
        }

        Commands::Clarifications(args) => {
            agentd::cli::clarifications::run(args)?;
        }
    }

    Ok(())
}
