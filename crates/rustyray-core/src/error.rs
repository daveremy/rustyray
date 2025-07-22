//! Error types for RustyRay operations.
//!
//! This module defines the various error conditions that can occur
//! in RustyRay, inspired by Ray's error handling but adapted for Rust.

use std::fmt;

/// The main error type for RustyRay operations.
#[derive(Debug, Clone)]
pub enum RustyRayError {
    /// Actor with the given ID was not found.
    ActorNotFound(crate::types::ActorId),

    /// Actor has been killed/terminated.
    ActorDead(crate::types::ActorId),

    /// Failed to send message to actor.
    MessageSendFailed(String),

    /// Actor mailbox is full (for bounded channels).
    MailboxFull(crate::types::ActorId),

    /// Timeout waiting for actor response.
    Timeout(String),

    /// Task execution failed.
    TaskExecutionFailed(String),

    /// Message type not recognized by actor.
    InvalidMessage,

    /// Runtime not initialized.
    RuntimeNotInitialized,

    /// Generic internal error.
    Internal(String),

    /// Error with additional context information.
    WithContext {
        /// Additional context about where/why the error occurred
        context: String,
        /// The underlying error
        source: Box<RustyRayError>,
    },
}

impl fmt::Display for RustyRayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ActorNotFound(id) => write!(f, "Actor {id} not found"),
            Self::ActorDead(id) => write!(f, "Actor {id} is dead"),
            Self::MessageSendFailed(msg) => write!(f, "Failed to send message: {msg}"),
            Self::MailboxFull(id) => write!(f, "Actor {id} mailbox is full"),
            Self::Timeout(msg) => write!(f, "Operation timed out: {msg}"),
            Self::TaskExecutionFailed(msg) => write!(f, "Task execution failed: {msg}"),
            Self::InvalidMessage => write!(f, "Invalid message type"),
            Self::RuntimeNotInitialized => write!(
                f,
                "Runtime not initialized. Call runtime::init() or use #[rustyray::main]"
            ),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
            Self::WithContext { context, source } => write!(f, "{context}: {source}"),
        }
    }
}

impl std::error::Error for RustyRayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::WithContext { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl RustyRayError {
    /// Add context to this error.
    ///
    /// This is useful for adding information about where or why an error occurred.
    ///
    /// # Example
    /// ```
    /// # use rustyray_core::error::RustyRayError;
    /// let err = RustyRayError::Internal("failed to connect".into());
    /// let err_with_context = err.context("while initializing actor system");
    /// ```
    pub fn context(self, context: impl Into<String>) -> Self {
        Self::WithContext {
            context: context.into(),
            source: Box::new(self),
        }
    }
}

/// Result type alias for RustyRay operations.
pub type Result<T> = std::result::Result<T, RustyRayError>;

/// Extension trait for Result types to easily add context to errors.
pub trait ResultExt<T> {
    /// Add context to an error.
    fn context(self, context: impl Into<String>) -> Result<T>;
}

impl<T> ResultExt<T> for Result<T> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.context(context))
    }
}
