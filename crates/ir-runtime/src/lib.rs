//! `ir-runtime` manages `.infrared/` project-scoped isolated environments,
//! runtime binary discovery (`global` vs `venv` modes), and normalized stream-JSON events.
//!
//! Primary ticket: `IR-203`.

use ir_core::IrResult;
use std::path::{Path, PathBuf};

/// Ensures `.infrared/` directory layout exists (`config/{claude,codex,agy}`, `state/sessions`, `bin`).
pub fn ensure_project_venv(_project_root: &Path) -> IrResult<PathBuf> {
    todo!("IR-203: Create .infrared/ layout and env.toml descriptor")
}
