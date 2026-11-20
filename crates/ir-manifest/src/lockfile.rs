//! Ticket `IR-101`: Deterministic `infra-red.lock` serialization and manifest digest verification.

use crate::manifest::ProjectManifest;
use ir_core::{ArtifactKind, IrError, IrResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lockfile {
    pub lockfile_version: u32,
    pub manifest_digest: String,
    #[serde(default, rename = "package")]
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedPackage {
    pub id: String,
    pub kind: ArtifactKind,
    pub version: String,
    pub source: String,
    pub subdir: Option<String>,
    pub integrity: String,
    pub mcp_protocol: Option<String>,
    #[serde(default)]
    pub requires: Vec<String>,
}

impl Lockfile {
    /// Reads and parses `infra-red.lock` from disk.
    pub fn from_file(path: &Path) -> IrResult<Self> {
        let raw = std::fs::read_to_string(path)?;
        toml::from_str(&raw).map_err(|err| IrError::Manifest {
            path: path.to_path_buf(),
            message: err.to_string(),
        })
    }

    /// Writes a deterministic, canonically sorted `infra-red.lock` with a managed header.
    pub fn write_to_file(&self, path: &Path) -> IrResult<()> {
        let mut sorted = self.clone();
        sorted.packages.sort_by(|a, b| a.id.cmp(&b.id));
        let body = toml::to_string_pretty(&sorted).map_err(|e| IrError::Other(e.into()))?;
        let content = format!(
            "# Managed by infra-red. Do not edit manually. Commit to version control.\n{body}"
        );
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Verifies that the lockfile's `manifest_digest` matches the given `ProjectManifest`.
    pub fn verify_matches_manifest(&self, manifest: &ProjectManifest) -> IrResult<()> {
        let expected = manifest.canonical_digest()?;
        if self.manifest_digest != expected {
            return Err(IrError::LockfileDrift {
                expected,
                actual: self.manifest_digest.clone(),
            });
        }
        Ok(())
    }

    /// Builds an initial lockfile snapshot directly from a [`ProjectManifest`].
    pub fn from_manifest(manifest: &ProjectManifest) -> IrResult<Self> {
        let manifest_digest = manifest.canonical_digest()?;
        let mut packages = Vec::new();

        for (name, spec) in &manifest.mcp {
            packages.push(LockedPackage {
                id: format!("mcp:{name}"),
                kind: ArtifactKind::McpServer,
                version: spec.version.clone().unwrap_or_else(|| "0.0.0".to_string()),
                source: spec.source.clone(),
                subdir: None,
                integrity: "sha256:pending".to_string(),
                mcp_protocol: spec.mcp_spec.clone(),
                requires: Vec::new(),
            });
        }
        for (name, spec) in &manifest.skills {
            packages.push(LockedPackage {
                id: format!("skill:{name}"),
                kind: ArtifactKind::Skill,
                version: spec.rev.clone().unwrap_or_else(|| "0.0.0".to_string()),
                source: spec.source.clone(),
                subdir: spec.subdir.clone(),
                integrity: "sha256:pending".to_string(),
                mcp_protocol: None,
                requires: Vec::new(),
            });
        }
        for (name, spec) in &manifest.plugins {
            packages.push(LockedPackage {
                id: format!("plugin:{name}"),
                kind: ArtifactKind::Plugin,
                version: spec.rev.clone().unwrap_or_else(|| "0.0.0".to_string()),
                source: spec.source.clone(),
                subdir: None,
                integrity: "sha256:pending".to_string(),
                mcp_protocol: None,
                requires: Vec::new(),
            });
        }

        packages.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(Self {
            lockfile_version: 1,
            manifest_digest,
            packages,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ProjectMeta;
    use tempfile::tempdir;

    #[test]
    fn lockfile_roundtrip_and_frozen_verification() {
        let dir = tempdir().unwrap();
        let lock_path = dir.path().join("infra-red.lock");

        let manifest = ProjectManifest {
            project: ProjectMeta {
                name: "demo".to_string(),
                schema_version: 1,
            },
            ..Default::default()
        };

        let lock = Lockfile::from_manifest(&manifest).unwrap();
        lock.write_to_file(&lock_path).unwrap();

        let reloaded = Lockfile::from_file(&lock_path).unwrap();
        reloaded.verify_matches_manifest(&manifest).unwrap();

        let mut modified = manifest;
        modified.project.name = "changed".to_string();
        assert!(matches!(
            reloaded.verify_matches_manifest(&modified),
            Err(IrError::LockfileDrift { .. })
        ));
    }
}
