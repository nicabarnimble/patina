extern crate self as patina;

pub mod beliefs;
pub mod child;
pub mod commands;
#[cfg(feature = "legacy-runtime")]
pub mod connect;
pub mod embeddings;
pub mod environment;
pub mod eventlog;
pub mod git;
#[cfg(feature = "legacy-runtime")]
pub mod interface;
#[cfg(not(feature = "legacy-runtime"))]
#[path = "interface/core.rs"]
pub mod interface;
pub mod lake;
pub mod layer;
pub mod measure;
pub mod migration;
pub mod models;
pub mod mother;
pub mod paths;
pub mod project;
pub mod release;
pub mod retrieval;
pub mod scanner;
#[cfg(feature = "legacy-runtime")]
pub mod secrets;
pub mod session;
pub mod slate;
pub mod spec;
pub mod version;
#[cfg(feature = "legacy-runtime")]
pub mod workspace;

pub mod test_support;

// Re-export commonly used types
pub use environment::Environment;
pub use layer::Layer;
pub use session::SessionManager;
