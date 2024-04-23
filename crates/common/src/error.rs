use std::fmt::Debug;

// errors in common layer
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid script type: {0}")]
    InvalidScriptType(String),
}
