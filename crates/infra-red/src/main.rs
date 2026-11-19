use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "ir",
    version,
    about = "Infra-Red: Unified package manager, environment isolator, and target projector for AI coding tooling."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize a new infra-red.toml manifest in the current repository (Ticket IR-107).
    Init,
    /// Import existing .mcp.json, .codex/config.toml, .agents/, .vscode/, and .cursor/ configs (Ticket IR-106).
    Migrate,
    /// Audit MCP context-window token bloat, target drift, plaintext secrets, and deprecations (Ticket IR-106).
    Doctor,
    /// Project infra-red.toml / infra-red.lock into all enabled IDE and agent targets (Ticket IR-107).
    Sync {
        #[arg(long)]
        frozen: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        profile: Option<String>,
    },
    /// Add an MCP server, skill, plugin, or runtime to infra-red.toml (Ticket IR-110).
    Add { spec: String },
    /// Review and approve quarantined repository lifecycle hooks or stdio commands (Ticket IR-110).
    Trust { package: Option<String> },
    /// Manage OS-keyring secrets without writing plaintext credentials to disk (Ticket IR-202).
    Secret {
        #[command(subcommand)]
        action: SecretAction,
    },
    /// Internal trampoline invoked by IDEs to inject keyring secrets into stdio MCP servers (Ticket IR-202).
    ExecMcp { server: String },
    /// Create or inspect the isolated .infrared/ project environment (Ticket IR-204).
    Venv,
    /// Run a coding agent inside the isolated .infrared/ environment (Ticket IR-204).
    Run {
        agent: Option<String>,
        #[arg(long)]
        profile: Option<String>,
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Start the ACP v1 Conductor proxy for Zed and JetBrains IDEs (Ticket IR-401).
    AcpServe {
        #[arg(long)]
        project: Option<std::path::PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum SecretAction {
    Set { key: String },
    List,
    Delete { key: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Init => todo!("IR-107: Implement `ir init`"),
        Commands::Migrate => todo!("IR-106: Implement `ir migrate`"),
        Commands::Doctor => todo!("IR-106: Implement `ir doctor`"),
        Commands::Sync { .. } => todo!("IR-107: Implement `ir sync`"),
        Commands::Add { .. } => todo!("IR-110: Implement `ir add`"),
        Commands::Trust { .. } => todo!("IR-110: Implement `ir trust`"),
        Commands::Secret { .. } => todo!("IR-202: Implement `ir secret`"),
        Commands::ExecMcp { .. } => todo!("IR-202: Implement `ir exec-mcp`"),
        Commands::Venv => todo!("IR-204: Implement `ir venv`"),
        Commands::Run { .. } => todo!("IR-204: Implement `ir run`"),
        Commands::AcpServe { .. } => todo!("IR-401: Implement `ir acp-serve`"),
    }
}
