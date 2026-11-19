//! `ir-targets` projects `infra-red.toml` / `infra-red.lock` into target IDE and agent configs
//! with drift detection and platform-aware linking (`CowOrHardlink` for VS Code Copilot).
//!
//! Tickets:
//! - `IR-104`: Claude Code, OpenAI Codex, Google Antigravity adapters + drift tracker
//! - `IR-105`: VS Code Copilot & Cursor adapters
//! - `IR-106`: `ir migrate` config importer
//! - `IR-402`: Zed & JetBrains absolute-path ACP adapters

use ir_core::TargetAdapter;
use std::sync::Arc;

/// Returns all built-in target adapters matching the enabled target IDs from `infra-red.toml`.
pub fn resolve_adapters(_enabled: &[String]) -> Vec<Arc<dyn TargetAdapter>> {
    todo!(
        "IR-104/IR-105/IR-402: Instantiate ClaudeCode, Codex, Antigravity, VsCode, Cursor, Zed, JetBrains adapters"
    )
}
