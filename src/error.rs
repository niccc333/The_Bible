//! Typed errors.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BibleError {
    #[error("unknown book in reference: '{0}'")]
    UnknownBook(String),
    #[error("invalid reference: '{0}'")]
    InvalidReference(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("failed to load bible data: {0}")]
    Load(String),
}
