use std::time::Duration;

use super::bitcoin::{BitcoinUnit, BITCOIN, MILLI_BITCOIN, SATOSHI};

pub fn now() -> Duration {
    #[cfg(target_arch = "wasm32")]
    return instant::SystemTime::now()
        .duration_since(instant::SystemTime::UNIX_EPOCH)
        .unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    return std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
}

pub fn spawn<F>(future: F)
where
    F: core::future::Future<Output = ()> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(future);
    }
    // #[cfg(not(target_arch = "wasm32"))]
    // {
    //     tokio::task::LocalSet::new().spawn_local(future);
    // }
}

pub fn convert_amount(value: f64, from: BitcoinUnit, to: BitcoinUnit) -> f64 {
    match from {
        BitcoinUnit::BTC => match to {
            BitcoinUnit::BTC => value,
            BitcoinUnit::MBTC => value * (BITCOIN / MILLI_BITCOIN) as f64,
            BitcoinUnit::SAT => value * (BITCOIN / SATOSHI) as f64,
        },
        BitcoinUnit::MBTC => match to {
            BitcoinUnit::BTC => value / (BITCOIN / MILLI_BITCOIN) as f64,
            BitcoinUnit::MBTC => value,
            BitcoinUnit::SAT => value * (MILLI_BITCOIN / SATOSHI) as f64,
        },
        BitcoinUnit::SAT => match to {
            BitcoinUnit::BTC => value / (BITCOIN / SATOSHI) as f64,
            BitcoinUnit::MBTC => value / (MILLI_BITCOIN / SATOSHI) as f64,
            BitcoinUnit::SAT => value,
        },
    }
}

pub fn max_f64(a: f64, b: f64) -> f64 {
    let max = a.max(b);
    if max.is_nan() {
        0f64
    } else {
        max
    }
}

pub fn min_f64(a: f64, b: f64) -> f64 {
    let min = a.min(b);
    if min.is_nan() {
        0f64
    } else {
        min
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        bitcoin::BitcoinUnit,
        utils::{convert_amount, max_f64, min_f64},
    };

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
        assert_eq!(convert_amount(0.00089377, BitcoinUnit::BTC, BitcoinUnit::SAT), 89377f64)
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
        assert_eq!(convert_amount(97.897, BitcoinUnit::MBTC, BitcoinUnit::SAT), 9789700.0)
    }

    #[test]
    fn should_convert_sat_to_btc() {
        assert_eq!(
            convert_amount(1527463f64, BitcoinUnit::SAT, BitcoinUnit::BTC),
            0.01527463
        )
    }

    #[test]
    fn should_convert_sat_to_mbtc() {
        assert_eq!(
            convert_amount(8867354f64, BitcoinUnit::SAT, BitcoinUnit::MBTC),
            88.67354
        )
    }

    #[test]
    fn should_do_nothing_sat_to_sat() {
        assert_eq!(
            convert_amount(9928764f64, BitcoinUnit::SAT, BitcoinUnit::SAT),
            9928764f64
        )
    }
}
