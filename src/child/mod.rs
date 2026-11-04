//! Child runtime boundary surface.

pub mod engine;
pub(crate) mod internal;
pub mod runtime;
#[cfg(feature = "legacy-runtime")]
pub mod scaffold;
#[cfg(feature = "legacy-runtime")]
pub(crate) mod toy_host;

// Re-exports for integration testing. Not a public API commitment.
#[doc(hidden)]
pub mod testing {
    pub use super::internal::{
        check_capabilities, ChildIngressMode, ChildKind, ChildManifest, ChildProvides,
        FilesystemAccessMode, PipelineEngine,
    };
    #[cfg(feature = "legacy-runtime")]
    pub use super::internal::{ChildEngine, FilesystemPreopen};
    #[cfg(feature = "legacy-runtime")]
    pub use super::toy_host::v2::{events_subscribe, EventRecord};
}
