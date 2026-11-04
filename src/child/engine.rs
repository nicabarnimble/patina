//! Canonical child-engine surface.

pub use crate::child::internal::{
    check_capabilities, ChildKind, ChildManifest, ChildRole, CredentialMapping,
    GrantedCapabilities, InjectionLocation, PipelineEngine, QueryDispatchFn, QueryScope,
};

#[cfg(feature = "legacy-runtime")]
pub use crate::child::internal::{
    ChildEngine, FilesystemAccessMode, FilesystemPreopen, GUEST_PROJECT_ROOT,
};

pub type ChildProvides = crate::child::internal::ChildProvides;
