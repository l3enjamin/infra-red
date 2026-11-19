use std::path::PathBuf;
use thiserror::Error;

/// Convenience result alias for `infra-red` operations.
pub type IrResult<T> = Result<T, IrError>;

/// Actionable domain errors shared across all `infra-red` crates.
#[derive(Debug, Error)]
pub enum IrError {
    #[error("manifest error at {path}: {message}")]
    Manifest { path: PathBuf, message: String },

    #[error("plaintext secret detected in manifest for `{key}`: {hint}")]
    PlaintextSecretDetected { key: String, hint: String },

    #[error(
        "lockfile drift detected: manifest digest {expected} does not match lockfile {actual} (run `ir sync` without `--frozen`)"
    )]
    LockfileDrift { expected: String, actual: String },

    #[error(
        "manual edit drift detected in target file {path} (re-run with `--force` to overwrite)"
    )]
    TargetDrift { path: PathBuf },

    #[error(
        "untrusted lifecycle hook or command in `{package_id}`: run `ir trust {package_id}` after inspecting its commands"
    )]
    UntrustedArtifact { package_id: String },

    #[error(
        "secret `{key}` not found in provider `{provider}` (set it with `ir secret set {key}`)"
    )]
    SecretNotFound { key: String, provider: String },

    #[error("dependency resolution failed:\n{report}")]
    ResolutionConflict { report: String },

    #[error("sandbox backend `{backend}` is unavailable: {reason}")]
    SandboxUnavailable { backend: String, reason: String },

    #[error("skill validation failed at {path}: {reason}")]
    InvalidSkill { path: PathBuf, reason: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
