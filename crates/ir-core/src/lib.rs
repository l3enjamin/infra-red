//! `ir-core` is the Layer 0 foundation crate of the `infra-red` workspace.
//!
//! It defines:
//! - Shared domain types ([`types`])
//! - Actionable typed errors ([`error`])
//! - The six extensibility trait seams ([`traits`])
//! - In-memory mock implementations ([`testing`]) so Layer 1–3 crates can be developed
//!   and unit-tested in parallel without network, OS keyring, or sandbox privileges.

pub mod error;
pub mod testing;
pub mod traits;
pub mod types;

pub use error::{IrError, IrResult};
pub use traits::{
    AgentRuntime, ArtifactSource, ProtocolBridge, SandboxBackend, SecretProvider, TargetAdapter,
};
pub use types::{
    ArtifactKind, ExecutionOutcome, NetworkMode, ReconcileReport, ResolvedArtifact, SandboxPolicy,
    SpawnSpec,
};
