pub use andromeda_bitcoin::error::Error as AndromedaBitcoinError;
pub use andromeda_common::error::Error as AndromedaCommonError;
use std::fmt::Debug;
// errors in common layer
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An Andromeda Bitcoin Error: \n\t{0}")]
    AndromedaBitcoinError(#[from] AndromedaBitcoinError),
    #[error("An Andromeda Common Error: \n\t{0}")]
    AndromedaCommonError(#[from] AndromedaCommonError),
    #[error("An Account Export Datetime Error")]
    AccountExportDatetimeError,
}
