//! Optional guest-runtime integration around local session records.
use super::internal::current_interface_name;
use anyhow::Result;
use patina::interface::interface as load_interface;
use std::path::Path;

pub(super) fn teardown_tmux_lane_best_effort(
    project_root: &Path,
    interface_name: &str,
    session_target: Option<&str>,
) {
    match patina::interface::teardown_interface_tmux_lane(
        project_root,
        interface_name,
        session_target,
    ) {
        Ok(true) => {
            eprintln!(
                "Patina: closed tmux lane for interface '{}' after session end.",
                interface_name
            );
        }
        Ok(false) => {}
        Err(error) => {
            eprintln!(
                "Patina: warning: failed to close tmux lane for interface '{}': {}",
                interface_name, error
            );
        }
    }
}

pub(super) fn resolve_native_session_interface(
    project_root: &Path,
    interface: Option<&str>,
) -> Result<String> {
    let resolved = interface
        .map(ToOwned::to_owned)
        .or_else(current_interface_name)
        .unwrap_or(patina::interface::resolve_preferred_ai_interface(
            project_root,
        )?);
    patina::interface::ensure_ai_project_config(project_root, None)?;
    let _ = load_interface(&resolved).map_err(|_| {
        anyhow::anyhow!(
            "Unsupported Patina AI interface '{}'. Choose one of: {}.",
            resolved,
            patina::interface::supported_ai_interfaces().join(", ")
        )
    })?;
    Ok(resolved)
}
