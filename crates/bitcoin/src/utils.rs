use andromeda_common::{BitcoinUnit, BITCOIN, MILLI_BITCOIN, SATOSHI};

use super::transactions::Pagination;
use crate::transactions::TransactionDetails;

#[derive(PartialEq, PartialOrd)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(PartialEq, PartialOrd)]
pub enum TransactionFilter {
    All,
    Receive,
    Send,
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(future: F)
where
    F: core::future::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(_future: F)
where
    F: core::future::Future<Output = ()> + 'static,
{
    //TODO:: why this is commented out?
    // tokio::task::LocalSet::new().spawn_local(future);
}

pub fn convert_amount(value: f64, from: BitcoinUnit, to: BitcoinUnit) -> f64 {
    match from {
        BitcoinUnit::BTC => match to {
            BitcoinUnit::BTC => value,
            BitcoinUnit::MBTC => value * (BITCOIN / MILLI_BITCOIN) as f64,
            BitcoinUnit::SATS => value * (BITCOIN / SATOSHI) as f64,
        },
        BitcoinUnit::MBTC => match to {
            BitcoinUnit::BTC => value / (BITCOIN / MILLI_BITCOIN) as f64,
            BitcoinUnit::MBTC => value,
            BitcoinUnit::SATS => value * (MILLI_BITCOIN / SATOSHI) as f64,
        },
        BitcoinUnit::SATS => match to {
            BitcoinUnit::BTC => value / (BITCOIN / SATOSHI) as f64,
            BitcoinUnit::MBTC => value / (MILLI_BITCOIN / SATOSHI) as f64,
            BitcoinUnit::SATS => value,
        },
    }
}

/// Returns the maximum value between two `f64` numbers, or 0.0 if both are NaN.
///
/// # Notes
///
/// If either of the input values is NaN (Not a Number), the function returns
/// 0.0.
///
/// # Examples
///
/// ```
/// use andromeda_bitcoin::utils::max_f64;
///
/// let result = max_f64(3.5, 2.0);
/// assert_eq!(result, 3.5);
///
/// let nan_result = max_f64(f64::NAN, f64::NAN);
/// assert_eq!(nan_result, 0.0);
/// ```
pub fn max_f64(a: f64, b: f64) -> f64 {
    let max = a.max(b);
    if max.is_nan() {
        0.0
    } else {
        max
    }
}

/// Returns the minimum value between two `f64` numbers, or 0.0 if both are NaN.
///
/// # Notes
///
/// If either of the input values is NaN (Not a Number), the function returns
/// 0.0.
///
/// # Examples
///
/// ```
/// use andromeda_bitcoin::utils::min_f64;
///
/// let result = min_f64(3.5, 2.0);
/// assert_eq!(result, 2.0);
///
/// let nan_result = min_f64(f64::NAN, f64::NAN);
/// assert_eq!(nan_result, 0.0);
/// ```
pub fn min_f64(a: f64, b: f64) -> f64 {
    let min = a.min(b);
    if min.is_nan() {
        0f64
    } else {
        min
    }
}

pub fn sort_txs(mut txs: Vec<TransactionDetails>, sort_order: Option<SortOrder>) -> Vec<TransactionDetails> {
    if let Some(sort_order) = sort_order {
        // we only sort by time for now
        txs.sort_by(|a, b| {
            if sort_order == SortOrder::Desc {
                b.time.cmp(&a.time)
            } else {
                a.time.cmp(&b.time)
            }
        });
    }

    txs
}

pub fn sort_and_paginate_txs(
    mut txs: Vec<TransactionDetails>,
    pagination: Pagination,
    sort_order: Option<SortOrder>,
) -> Vec<TransactionDetails> {
    txs = sort_txs(txs, sort_order);

    // We paginated sorted vector
    txs.into_iter()
        .skip(pagination.skip)
        .take(pagination.take)
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use andromeda_common::BitcoinUnit;

    use super::super::utils::{convert_amount, max_f64, min_f64};

    #[test]
    fn should_return_max_value() {
        assert_eq!(max_f64(78.8, -97.4), 78.8)
    }

    #[test]
    fn should_return_max_value_0() {
        assert_eq!(max_f64(f64::NAN, f64::NAN), 0.0)
    }

    #[test]
    fn should_return_min_value() {
        assert_eq!(min_f64(78.8, -97.4), -97.4)
    }

    #[test]
    fn should_return_min_value_0() {
        assert_eq!(min_f64(f64::NAN, f64::NAN), 0.0)
    }

    #[test]
    fn should_do_nothing_for_btc_to_btc() {
        assert_eq!(convert_amount(0.0075634, BitcoinUnit::BTC, BitcoinUnit::BTC), 0.0075634)
    }

    #[test]
    fn should_convert_btc_to_mbtc() {
        assert_eq!(convert_amount(0.0056342, BitcoinUnit::BTC, BitcoinUnit::MBTC), 5.6342)
    }

    #[test]
    fn should_convert_btc_to_sat() {
        assert_eq!(
            convert_amount(0.00089377, BitcoinUnit::BTC, BitcoinUnit::SATS),
            89377f64
        )
    }

    #[test]
    fn should_convert_mbtc_to_btc() {
        assert_eq!(convert_amount(7.89, BitcoinUnit::MBTC, BitcoinUnit::BTC), 0.00789)
    }

    #[test]
    fn should_do_nothing_for_mbtc_to_mbtc() {
        assert_eq!(convert_amount(5.13, BitcoinUnit::MBTC, BitcoinUnit::MBTC), 5.13)
    }

    #[test]
    fn should_convert_mbtc_to_sat() {
        assert_eq!(convert_amount(97.897, BitcoinUnit::MBTC, BitcoinUnit::SATS), 9789700.0)
    }

    #[test]
    fn should_convert_sat_to_btc() {
        assert_eq!(
            convert_amount(1527463f64, BitcoinUnit::SATS, BitcoinUnit::BTC),
            0.01527463
        )
    }

    #[test]
    fn should_convert_sat_to_mbtc() {
        assert_eq!(
            convert_amount(8867354f64, BitcoinUnit::SATS, BitcoinUnit::MBTC),
            88.67354
        )
    }

    #[test]
    fn should_do_nothing_sat_to_sat() {
        assert_eq!(
            convert_amount(9928764f64, BitcoinUnit::SATS, BitcoinUnit::SATS),
            9928764f64
        )
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! read_mock_file {
    ($filename:expr) => {{
        use std::{fs::File, io::Read};

        let mut file = File::open(format!("./src/tests/mocks/{}.json", $filename)).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        contents
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! read_mock_raw_file {
    ($filename:expr) => {{
        use std::{fs::File, io::Read};

        let mut file = File::open(format!("./src/tests/mocks/{}.raw", $filename)).unwrap();
        let mut contents = Vec::new();

        file.read_to_end(&mut contents).unwrap();
        contents
    }};
}
