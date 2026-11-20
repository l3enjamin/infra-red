//! `ir-store` manages the versioned global content-addressable store (`v1`),
//! Copy-on-Write / hardlink materialization (`IR-103`), and remote artifact fetchers (`IR-108`).

use ir_core::IrResult;
use sha2::{Digest, Sha256};
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

/// Projects a file or directory tree from `src` to `dst` using `strategy`.
///
/// When `LinkStrategy::CowOrHardlink` is selected, **zero symlinks** are ever created:
/// 1. On Unix (macOS APFS, Linux Btrfs/XFS), attempts `reflink_copy::reflink`.
/// 2. Falls back to `std::fs::hard_link`.
/// 3. Falls back to `std::fs::copy` across filesystems or permission boundaries.
pub fn materialize_tree(
    src: &Path,
    dst: &Path,
    strategy: LinkStrategy,
) -> IrResult<MaterializeStats> {
    let mut stats = MaterializeStats::default();

    if strategy == LinkStrategy::Symlink {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        remove_existing_path(dst)?;
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(src, dst)?;
            stats.symlinks_created += 1;
            return Ok(stats);
        }
        #[cfg(windows)]
        {
            if src.is_dir() {
                std::os::windows::fs::symlink_dir(src, dst)?;
            } else {
                std::os::windows::fs::symlink_file(src, dst)?;
            }
            stats.symlinks_created += 1;
            return Ok(stats);
        }
    }

    materialize_cow_recursive(src, dst, &mut stats)?;
    Ok(stats)
}

fn materialize_cow_recursive(src: &Path, dst: &Path, stats: &mut MaterializeStats) -> IrResult<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)?;
        let mut entries: Vec<_> = std::fs::read_dir(src)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let child_src = entry.path();
            let child_dst = dst.join(entry.file_name());
            materialize_cow_recursive(&child_src, &child_dst, stats)?;
        }
    } else {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        remove_existing_path(dst)?;

        // On Windows NTFS, skip untested ReFS reflink assumptions and go straight to hard_link -> copy.
        #[cfg(not(windows))]
        {
            if reflink_copy::reflink(src, dst).is_ok() {
                stats.files_reflinked += 1;
                return Ok(());
            }
        }

        if std::fs::hard_link(src, dst).is_ok() {
            stats.files_hardlinked += 1;
        } else {
            std::fs::copy(src, dst)?;
            stats.files_copied += 1;
        }
    }
    Ok(())
}

fn remove_existing_path(path: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.file_type().is_symlink() || meta.is_file() {
                std::fs::remove_file(path)
            } else {
                std::fs::remove_dir_all(path)
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

/// Computes a deterministic SHA-256 digest (`sha256:<hex>`) over a file or sorted directory tree.
pub fn sha256_path(path: &Path) -> IrResult<String> {
    let mut hasher = Sha256::new();
    hash_path_recursive(path, path, &mut hasher)?;
    let digest = hasher.finalize();
    Ok(format!("sha256:{}", hex_encode(&digest)))
}

fn hash_path_recursive(root: &Path, current: &Path, hasher: &mut Sha256) -> IrResult<()> {
    if current.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(current)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            hash_path_recursive(root, &entry.path(), hasher)?;
        }
    } else {
        let rel = current
            .strip_prefix(root)
            .unwrap_or(current)
            .to_string_lossy()
            .replace('\\', "/");
        hasher.update(rel.as_bytes());
        hasher.update(b"\0");
        let bytes = std::fs::read(current)?;
        hasher.update(&bytes);
        hasher.update(b"\0");
    }
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cow_or_hardlink_materializes_tree_with_zero_symlinks_and_matching_digest() {
        let root = tempdir().unwrap();
        let src = root.path().join("cas-skill");
        let dst = root.path().join("vscode-skill");

        std::fs::create_dir_all(src.join("references")).unwrap();
        std::fs::write(src.join("SKILL.md"), "---\nname: demo\n---\n").unwrap();
        std::fs::write(src.join("references/guide.md"), "Guide content").unwrap();

        let stats = materialize_tree(&src, &dst, LinkStrategy::CowOrHardlink).unwrap();
        assert_eq!(stats.symlinks_created, 0);
        assert_eq!(
            stats.files_reflinked + stats.files_hardlinked + stats.files_copied,
            2
        );

        assert!(
            !std::fs::symlink_metadata(&dst)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(
            !std::fs::symlink_metadata(dst.join("SKILL.md"))
                .unwrap()
                .file_type()
                .is_symlink()
        );

        assert_eq!(sha256_path(&src).unwrap(), sha256_path(&dst).unwrap());
    }
}
