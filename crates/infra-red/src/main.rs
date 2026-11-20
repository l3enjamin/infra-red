use anyhow::Context;
use clap::{Parser, Subcommand};
use ir_manifest::lockfile::Lockfile;
use ir_manifest::manifest::ProjectManifest;
use ir_targets::{
    ReconcileOptions, audit_repository, migrate_repository, project_manifest_to_targets,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "ir",
    version,
    about = "Infra-Red: Unified package manager, environment isolator, and target projector for AI coding tooling."
)]
struct Cli {
    /// Override project directory (defaults to current working directory).
    #[arg(long, global = true)]
    project_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize a new infra-red.toml manifest in the current repository (Ticket IR-107).
    Init {
        /// Optional project name (defaults to directory name).
        #[arg(long)]
        name: Option<String>,
    },
    /// Import existing .mcp.json, .codex/config.toml, .agents/, .vscode/, and .cursor/ configs (Ticket IR-106).
    Migrate {
        /// Overwrite existing infra-red.toml if present.
        #[arg(long)]
        force: bool,
    },
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
        project: Option<PathBuf>,
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
    let project_root = match cli.project_dir {
        Some(dir) => dir,
        None => std::env::current_dir().context("failed to determine current directory")?,
    };

    match cli.command {
        Commands::Init { name } => cmd_init(&project_root, name.as_deref())?,
        Commands::Migrate { force } => cmd_migrate(&project_root, force)?,
        Commands::Doctor => cmd_doctor(&project_root)?,
        Commands::Sync {
            frozen,
            dry_run,
            force,
            profile,
        } => cmd_sync(&project_root, frozen, dry_run, force, profile.as_deref())?,
        Commands::Add { .. } => todo!("IR-110: Implement `ir add`"),
        Commands::Trust { .. } => todo!("IR-110: Implement `ir trust`"),
        Commands::Secret { .. } => todo!("IR-202: Implement `ir secret`"),
        Commands::ExecMcp { .. } => todo!("IR-202: Implement `ir exec-mcp`"),
        Commands::Venv => todo!("IR-204: Implement `ir venv`"),
        Commands::Run { .. } => todo!("IR-204: Implement `ir run`"),
        Commands::AcpServe { .. } => todo!("IR-401: Implement `ir acp-serve`"),
    }

    Ok(())
}

fn cmd_init(project_root: &Path, name_override: Option<&str>) -> anyhow::Result<()> {
    let manifest_path = project_root.join("infra-red.toml");
    if manifest_path.exists() {
        anyhow::bail!(
            "infra-red.toml already exists at {}",
            manifest_path.display()
        );
    }

    let project_name = name_override
        .map(ToString::to_string)
        .or_else(|| {
            project_root
                .file_name()
                .and_then(|s| s.to_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "infra-red-project".to_string());

    let starter = format!(
        r#"[project]
name = {project_name:?}
schema_version = 1

[env]
isolation = "project"
default_runtime = "claude"
sandbox_profile = "workspace-write"

[targets]
enabled = ["claude-code", "codex", "antigravity", "vscode", "cursor"]
secret_delivery = "trampoline"
"#
    );

    std::fs::write(&manifest_path, starter)?;
    ensure_gitignore_entries(project_root)?;
    println!("Initialized {}", manifest_path.display());
    Ok(())
}

fn cmd_migrate(project_root: &Path, force: bool) -> anyhow::Result<()> {
    let manifest_path = project_root.join("infra-red.toml");
    if manifest_path.exists() && !force {
        anyhow::bail!(
            "infra-red.toml already exists at {} (pass --force to overwrite)",
            manifest_path.display()
        );
    }

    let report = migrate_repository(project_root)?;
    let serialized = toml::to_string_pretty(&report.manifest)?;
    std::fs::write(&manifest_path, serialized)?;
    ensure_gitignore_entries(project_root)?;

    println!(
        "Migrated {} server(s) from {} existing config file(s) into {}",
        report.servers_imported,
        report.sources_imported.len(),
        manifest_path.display()
    );
    if !report.extracted_secrets.is_empty() {
        println!("Extracted plaintext credentials into logical keyring references:");
        for (source_field, keyring_key) in &report.extracted_secrets {
            println!("  - {source_field} -> run `ir secret set {keyring_key}`");
        }
    }
    Ok(())
}

fn cmd_sync(
    project_root: &Path,
    frozen: bool,
    dry_run: bool,
    force: bool,
    profile: Option<&str>,
) -> anyhow::Result<()> {
    let manifest = ProjectManifest::load_with_local_override(project_root)?;
    let lock_path = project_root.join("infra-red.lock");

    if frozen {
        if !lock_path.exists() {
            anyhow::bail!("--frozen specified but infra-red.lock does not exist");
        }
        let lock = Lockfile::from_file(&lock_path)?;
        lock.verify_matches_manifest(&manifest)?;
    } else if !dry_run {
        let lock = Lockfile::from_manifest(&manifest)?;
        lock.write_to_file(&lock_path)?;
    }

    let report = project_manifest_to_targets(
        project_root,
        &manifest,
        profile,
        ReconcileOptions { dry_run, force },
    )?;

    println!(
        "Synced {} target path(s) (dry_run={dry_run}).",
        report.files_written.len()
    );
    if !report.files_skipped_due_to_drift.is_empty() {
        println!("Skipped manually edited file(s) (pass --force to overwrite):");
        for skipped in &report.files_skipped_due_to_drift {
            println!("  - {}", skipped.display());
        }
    }
    for warn in &report.capability_warnings {
        println!("Warning: {warn}");
    }
    Ok(())
}

fn cmd_doctor(project_root: &Path) -> anyhow::Result<()> {
    let report = audit_repository(project_root)?;
    println!("Infra-Red Doctor Audit");
    println!("  MCP servers declared: {}", report.mcp_server_count);
    println!(
        "  Estimated upfront tool tokens/turn: ~{}",
        report.estimated_upfront_tokens
    );
    for finding in &report.plaintext_secret_findings {
        println!("  [SECRET] {finding}");
    }
    for drifted in &report.drifted_files {
        println!("  [DRIFT] {drifted}");
    }
    for dep in &report.deprecation_warnings {
        println!("  [DEPRECATED] {dep}");
    }
    for rec in &report.recommendations {
        println!("  [TIP] {rec}");
    }
    Ok(())
}

fn ensure_gitignore_entries(project_root: &Path) -> anyhow::Result<()> {
    let gitignore_path = project_root.join(".gitignore");
    let mut content = if gitignore_path.exists() {
        std::fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    let mut changed = false;
    for entry in [".infrared/", "infra-red.local.toml"] {
        if !content.lines().any(|line| line.trim() == entry) {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(entry);
            content.push('\n');
            changed = true;
        }
    }
    if changed {
        std::fs::write(gitignore_path, content)?;
    }
    Ok(())
}
