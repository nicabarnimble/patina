//! Spec lifecycle management
//!
//! This module follows the dependable-rust pattern:
//! - Public interface (this file): clean API for spec operations
//! - Internal implementation: all logic in internal/

#[allow(dead_code)]
pub(crate) mod internal;

#[allow(unused_imports)]
// Data types and functions re-exported for session integration (Phase 5)
pub(crate) use internal::{get_all_specs, get_blocked_specs, ListFilters};

#[allow(unused_imports)]
// Query data functions re-exported for MCP (Phase 6)
pub(crate) use internal::{
    check_spec_value, get_ready_specs, handoff_spec_value, history_spec_value, next_spec_value,
    packet_spec_value, prompt_spec_value, show_spec_value,
};

#[allow(unused_imports)]
// Mutation _value() functions re-exported for MCP (Phase 6)
pub(crate) use internal::{
    abandon_spec_value, block_spec_value, complete_spec_value, create_spec_value, pause_spec_value,
    promote_spec_value, rename_spec_value, reopen_spec_value, resume_spec_value, set_spec_value,
    split_spec_value,
};

#[cfg(feature = "legacy-runtime")]
mod legacy;
#[cfg(feature = "legacy-runtime")]
pub use legacy::{execute, SpecCommands};
