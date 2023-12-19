const ONE_BTC_IN_SATS: f64 = 100_000_000f64;

pub fn sats_to_btc(sats: u64) -> f64 {
    sats as f64 / ONE_BTC_IN_SATS
}

pub fn btc_to_sats(btc: f64) -> u64 {
    (btc * ONE_BTC_IN_SATS).round() as u64
}
