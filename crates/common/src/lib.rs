use serde::{Deserialize, Serialize};

pub const SATOSHI: u64 = 1;
pub const BITCOIN: u64 = 100_000_000 * SATOSHI;
pub const MILLI_BITCOIN: u64 = BITCOIN / 1000;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum BitcoinUnit {
    /// 100,000,000 sats
    BTC,
    /// 100,000 sats
    MBTC,
    /// 1 sat
    SAT,
}
