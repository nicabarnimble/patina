//! Passive interface identity; no launcher, discovery subprocesses or tmux.
#[path = "internal/bundle.rs"]
mod bundle;
#[path = "metadata.rs"]
mod metadata;

pub use bundle::{
    builtin_registry_toml, canonical_interface_name, canonicalize_required_interface,
    interface_bundle, interface_bundle_catalog, is_supported_ai_interface, supported_ai_interfaces,
    BundleTmuxPolicy, InterfaceBundle,
};
pub use metadata::{CLAUDE_INTERFACE_VERSION, GEMINI_INTERFACE_VERSION};
