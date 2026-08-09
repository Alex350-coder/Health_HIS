//! Tauri event emission (StateManagement.md — cross-module propagation channel). Events follow
//! the `<module>:<entity>:<action>` naming convention (IPC.md Section 3, CLAUDE.md Section 8) and
//! are emitted from the command layer after a service call succeeds, never from services
//! themselves, so services stay testable without a live Tauri runtime.

use serde::Serialize;
use tauri::Emitter;

use crate::errors::AppError;

/// Emits `event` with `payload` to every listening window. A failure here is a technical error
/// (no listener state to react to, not a data problem), so it maps to `AppError::Unexpected`
/// with the same correlation-id convention as other technical errors.
pub fn emit<T: Serialize + Clone>(
    app: &tauri::AppHandle,
    event: &str,
    payload: &T,
) -> Result<(), AppError> {
    app.emit(event, payload).map_err(|error| {
        let correlation_id = crate::errors::app_error::correlation_id();
        tracing::error!(correlation_id, error = %error, event, "event emission failed");
        AppError::Unexpected {
            message: "an event emission error occurred".to_string(),
            correlation_id,
        }
    })
}
