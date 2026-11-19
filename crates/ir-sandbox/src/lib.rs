//! `ir-sandbox` implements OS-native confinement (Linux `bwrap`+`seccomp`,
//! macOS `/usr/bin/sandbox-exec`, Windows Restricted Token/Job Objects, opt-in OCI)
//! and deterministic process-tree reaping via `process-wrap`.
//!
//! Tickets: `IR-301`, `IR-302`, `IR-303`.

use ir_core::SandboxBackend;
use std::sync::Arc;

/// Selects the default OS-native [`SandboxBackend`] for the current platform.
pub fn default_backend() -> anyhow::Result<Arc<dyn SandboxBackend>> {
    todo!(
        "IR-301/IR-302/IR-303: Select BubblewrapBackend (Linux), SeatbeltBackend (macOS), or WindowsJobBackend (Windows)"
    )
}
