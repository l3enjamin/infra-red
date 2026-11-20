//! Ticket `IR-104`: Target file SHA-256 drift tracker stored in `.infrared/state/target-hashes.json`.

use ir_core::IrResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TargetHashState {
    pub files: BTreeMap<String, String>,
}

impl TargetHashState {
    pub fn state_path(project_root: &Path) -> std::path::PathBuf {
        project_root
            .join(".infrared")
            .join("state")
            .join("target-hashes.json")
    }

    pub fn load(project_root: &Path) -> IrResult<Self> {
        let path = Self::state_path(project_root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    }

    pub fn save(&self, project_root: &Path) -> IrResult<()> {
        let path = Self::state_path(project_root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json =
            serde_json::to_string_pretty(self).map_err(|e| ir_core::IrError::Other(e.into()))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Returns `true` if `target_file` currently exists on disk and its SHA-256 differs from
    /// the last recorded hash emitted by `ir sync`.
    pub fn has_manual_drift(&self, project_root: &Path, target_file: &Path) -> IrResult<bool> {
        if !target_file.exists() {
            return Ok(false);
        }
        let key = relative_key(project_root, target_file);
        let Some(recorded) = self.files.get(&key) else {
            return Ok(false);
        };
        let current = ir_store::sha256_path(target_file)?;
        Ok(&current != recorded)
    }

    pub fn record(&mut self, project_root: &Path, target_file: &Path) -> IrResult<()> {
        let key = relative_key(project_root, target_file);
        let digest = ir_store::sha256_path(target_file)?;
        self.files.insert(key, digest);
        Ok(())
    }
}

pub fn relative_key(project_root: &Path, target_file: &Path) -> String {
    target_file
        .strip_prefix(project_root)
        .unwrap_or(target_file)
        .to_string_lossy()
        .replace('\\', "/")
}
