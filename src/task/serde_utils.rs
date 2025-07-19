//! Serialization utilities for the task system.
//!
//! This module wraps bincode 2.0 functionality to provide a simpler API
//! for our use case.

use crate::error::{Result, RustyRayError};
use serde::{de::DeserializeOwned, Serialize};

/// Configuration for bincode serialization
fn config() -> bincode::config::Configuration {
    bincode::config::standard()
}

/// Serialize a value to bytes
pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serde::encode_to_vec(value, config())
        .map_err(|e| RustyRayError::Internal(format!("Failed to serialize: {:?}", e)))
}

/// Deserialize bytes to a value
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let (value, _) = bincode::serde::decode_from_slice(bytes, config())
        .map_err(|e| RustyRayError::Internal(format!("Failed to deserialize: {:?}", e)))?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let value = 42i32;
        let bytes = serialize(&value).unwrap();
        let decoded: i32 = deserialize(&bytes).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_struct_roundtrip() {
        #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
        struct TestStruct {
            x: i32,
            y: String,
        }

        let value = TestStruct {
            x: 100,
            y: "hello".to_string(),
        };

        let bytes = serialize(&value).unwrap();
        let decoded: TestStruct = deserialize(&bytes).unwrap();
        assert_eq!(value, decoded);
    }
}
