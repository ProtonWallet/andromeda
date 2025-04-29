use std::fmt::Debug;

// errors in common layer
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid script type: {0}")]
    InvalidScriptType(String),
    #[error("Invalid network: {0}")]
    InvalidNetwork(String),
    #[error("Error compiling typst")]
    CompileTypst,
    #[error("Error exporting PDF")]
    ExportPDF,
}
