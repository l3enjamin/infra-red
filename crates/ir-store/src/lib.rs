//! `ir-store` manages the versioned global content-addressable store (`v1`),
//! Copy-on-Write / hardlink materialization (`IR-103`), and remote artifact fetchers (`IR-108`).

use ir_core::IrResult;
use std::path::Path;

/// Controls how files from the CAS are projected into target directories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStrategy {
    Symlink,
    /// Uses CoW `reflink-copy` where supported, falling back to `hard_link` and `copy`.
    /// Required for targets like VS Code Copilot that cannot traverse symlinks.
    CowOrHardlink,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MaterializeStats {
    pub files_reflinked: usize,
    pub files_hardlinked: usize,
    pub files_copied: usize,
    pub symlinks_created: usize,
}

pub fn materialize_tree(
    _src: &Path,
    _dst: &Path,
    _strategy: LinkStrategy,
) -> IrResult<MaterializeStats> {
    todo!(
        "IR-103: Implement CoW reflink -> hardlink -> copy ladder with zero symlinks when CowOrHardlink"
    )
}

pub fn sha256_path(_path: &Path) -> IrResult<String> {
    todo!("IR-103: Compute deterministic SHA-256 digest of a file or sorted directory tree")
}
