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
    
    /// Generic internal error.
    Internal(String),
}

impl fmt::Display for RustyRayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ActorNotFound(id) => write!(f, "Actor {} not found", id),
            Self::ActorDead(id) => write!(f, "Actor {} is dead", id),
            Self::MessageSendFailed(msg) => write!(f, "Failed to send message: {}", msg),
            Self::MailboxFull(id) => write!(f, "Actor {} mailbox is full", id),
            Self::Timeout(msg) => write!(f, "Operation timed out: {}", msg),
            Self::TaskExecutionFailed(msg) => write!(f, "Task execution failed: {}", msg),
            Self::InvalidMessage => write!(f, "Invalid message type"),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for RustyRayError {}

/// Result type alias for RustyRay operations.
pub type Result<T> = std::result::Result<T, RustyRayError>;