//! An extensible async Esplora client
//!
//! This library provides an extensible async Esplora client to query Esplora's
//! backend.
//!
//! The library provides the possibility to build an async client using
//! [`andromeda-api`]. The library supports communicating to Esplora via a proxy
//! and also using TLS (SSL) for secure communication.
//!
//!
//! ## Usage
//!
//! Here is an example of how to create an asynchronous client.
//!
//! ```no_run
//! # #[cfg(feature = "async")]
//! # {
//! use esplora_client::Builder;
//! let builder = Builder::new("https://blockstream.info/testnet/api");
//! let async_client = builder.build_async();
//! # Ok::<(), esplora_client::Error>(());
//! # }
//! ```

#![allow(clippy::result_large_err)]

use std::collections::HashMap;

pub mod api;
pub mod r#async;
pub mod async_ext;
pub mod error;

pub use api::*;
pub use async_ext::EsploraAsyncExt;
use error::Error;
pub use r#async::AsyncClient;

/// Get a fee value in sats/vbytes from the estimates
/// that matches the confirmation target set as parameter.
pub fn convert_fee_rate(target: usize, estimates: HashMap<String, f64>) -> Result<f32, Error> {
    let fee_val = {
        let mut pairs = estimates
            .into_iter()
            .filter_map(|(k, v)| Some((k.parse::<usize>().ok()?, v)))
            .collect::<Vec<_>>();
        pairs.sort_unstable_by_key(|(k, _)| std::cmp::Reverse(*k));
        pairs
            .into_iter()
            .find(|(k, _)| k <= &target)
            .map(|(_, v)| v)
            .unwrap_or(1.0)
    };
    Ok(fee_val as f32)
}
