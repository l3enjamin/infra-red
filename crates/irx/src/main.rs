use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "irx",
    version,
    about = "Infra-Red Ephemeral Runner: Stateless, OS-sandboxed one-shot agent execution & MCP 2026-07-28 server."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Target coding agent runtime (e.g., claude, codex, agy).
    #[arg(long)]
    agent: Option<String>,

    /// Explicit minimal artifacts to attach (e.g., mcp:github, skill:rust-security-audit).
    #[arg(long = "with")]
    with_artifacts: Vec<String>,

    /// Mount workspace read-only inside the ephemeral sandbox.
    #[arg(long)]
    ro: Option<std::path::PathBuf>,

    /// Prompt passed to the one-shot agent execution.
    #[arg(last = true)]
    prompt: Vec<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run `irx` as a stateless MCP 2026-07-28 server exposing `irx_list_catalog` and `irx_run_isolated` (Ticket IR-304/305).
    ServeMcp {
        #[arg(long, default_value = "stdio")]
        transport: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Some(Commands::ServeMcp { .. }) => {
            todo!("IR-305: Start ir_mcp_server::serve_stdio with platform default sandbox")
        }
        None => {
            todo!("IR-305: Execute ephemeral one-shot sandboxed agent task with --with artifacts")
        }
    }
}
