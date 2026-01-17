//! IPC types for viewer (legacy, kept for test compatibility)
//!
//! Note: The actual API handlers are now in mod.rs using axum.

use super::manager::ViewerError;

/// Errors that can occur during IPC handling
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("Failed to parse IPC message: {0}")]
    ParseError(String),

    #[error("Annotation error: {0}")]
    AnnotationError(String),

    #[error("Viewer error: {0}")]
    ViewerError(#[from] ViewerError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_error_display() {
        let err = IpcError::ParseError("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }
}
