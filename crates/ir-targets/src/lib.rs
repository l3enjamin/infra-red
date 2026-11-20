//! `ir-targets` projects `infra-red.toml` / `infra-red.lock` into target IDE and agent configs
//! with drift detection and platform-aware linking (`CowOrHardlink` for VS Code Copilot).
//!
//! Tickets:
//! - `IR-104`: Claude Code, OpenAI Codex, Google Antigravity adapters + drift tracker
//! - `IR-105`: VS Code Copilot & Cursor adapters
//! - `IR-106`: `ir migrate` config importer & `ir doctor` auditor
//! - `IR-402`: Zed & JetBrains absolute-path ACP adapters

pub mod doctor;
pub mod drift;
pub mod migrate;
pub mod projectors;

pub use doctor::{DoctorReport, audit_repository};
pub use migrate::{MigrateReport, migrate_repository};
pub use projectors::{
    AntigravityAdapter, ClaudeCodeAdapter, CodexAdapter, CursorAdapter, ReconcileOptions,
    VsCodeAdapter, project_manifest_to_targets, resolve_adapters,
};
