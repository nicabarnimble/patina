//! Mother - Cross-project awareness layer
//!
//! This module consolidates mother functionality:
//! - **Client**: HTTP client for communicating with `patina serve` daemon
//! - **Graph**: Local SQLite storage for project relationships
//!
//! # Client Usage (containers → mother)
//!
//! Set `PATINA_MOTHER=host:port` to enable remote queries.
//! ```ignore
//! use patina::mother;
//!
//! if mother::is_configured() {
//!     let response = mother::scry(request)?;
//! }
//! ```
//!
//! # Graph Usage (local relationship storage)
//!
//! ```ignore
//! use patina::mother::{Graph, EdgeType};
//!
//! let graph = Graph::open()?;
//! graph.add_edge("patina", "dojo", EdgeType::TestsWith, Some("benchmark"))?;
//! let related = graph.get_related("patina", &[EdgeType::Uses, EdgeType::TestsWith])?;
//! ```

#[cfg(feature = "legacy-runtime")]
pub mod broker;
#[cfg(feature = "legacy-runtime")]
pub mod doctor_runtime;
#[cfg(feature = "legacy-runtime")]
mod internal;
#[cfg(feature = "legacy-runtime")]
pub mod skills;

// Bridge module: state logic now lives in the mother crate.
// Re-export so existing `crate::mother::state::*` paths continue to compile.
#[cfg(feature = "legacy-runtime")]
pub(crate) mod state {
    pub use mother_crate::state::*;
}

// Child trait exports
pub use crate::child::runtime::{
    CallCorrelation, Child, ChildCallRequest, ChildHealth, ChildRequest, ChildResponse, MotherHost,
    PendingEvent, TaskIntent, TaskIntentKind, Toy,
};
#[cfg(feature = "legacy-runtime")]
pub use mother_crate::child_registry::{
    ChildRegistryProvider, ChildRegistrySyncEngine, DiscoveredChildRelease,
    GitHubChildRegistryProvider, SourceSyncReport,
};
pub use mother_crate::state::{
    ChildInstallRecord, ChildInstallUpdate, ChildRegistryAuditEventUpdate,
    ChildRegistryAuditRecord, ChildRegistryEntryRecord, ChildRegistryEntryUpdate,
    ChildRegistrySourceRecord, ChildRegistrySourceUpdate, InterfaceKindId, LakeCursorUpdate,
    MotherRuntimeStore, MotherSessionParticipant, MotherSessionRecord, MotherSessionStatus,
    ProjectBeliefStateRecord, ProjectBeliefStateUpdate, ProjectChildAssignmentRecord,
    ProjectChildAssignmentUpdate, ProjectUid, QueuedTask, RunStatus, TaskStatus, VoiceUid,
};
pub use mother_crate::toys::{GrantedIngressSource, GrantedToys};

// Graph exports
pub use crate::beliefs::{
    BeliefEntry, BeliefStatus, Edge, EdgeType, EdgeUsageStats, Graph, Node, NodeType, WeightChange,
    WeightLearningReport, DEFAULT_ALPHA, MIN_SAMPLES, WEIGHT_MAX, WEIGHT_MIN,
};

#[cfg(feature = "legacy-runtime")]
mod client;
#[cfg(feature = "legacy-runtime")]
pub use client::*;
