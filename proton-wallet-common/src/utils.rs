use crate::transactions::{Pagination, SimpleTransaction};

const ONE_BTC_IN_SATS: f64 = 100_000_000f64;

pub fn sats_to_btc(sats: u64) -> f64 {
    sats as f64 / ONE_BTC_IN_SATS
}

pub fn btc_to_sats(btc: f64) -> u64 {
    (btc * ONE_BTC_IN_SATS).round() as u64
}

pub fn sort_and_paginate_txs(
    mut simple_txs: Vec<SimpleTransaction>,
    pagination: Pagination,
    sorted: bool,
) -> Vec<SimpleTransaction> {
    if sorted {
        simple_txs.sort_by(|a, b| b.get_time().partial_cmp(&a.get_time()).unwrap());
    }

    // We paginated sorted vector
    let paginated = simple_txs
        .into_iter()
        .skip(pagination.skip)
        .take(pagination.take)
        .collect::<Vec<_>>();

    paginated
}
