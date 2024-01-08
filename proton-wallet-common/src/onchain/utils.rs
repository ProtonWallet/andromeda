use super::transactions::{Pagination, SimpleTransaction};

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
