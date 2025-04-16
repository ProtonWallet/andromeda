use andromeda_bitcoin::{
    transactions::Pagination,
    utils::{SortOrder, TransactionFilter},
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmPagination {
    skip: usize,
    take: usize,
}

impl From<WasmPagination> for Pagination {
    fn from(val: WasmPagination) -> Self {
        Pagination::new(val.skip, val.take)
    }
}

#[wasm_bindgen]
pub enum WasmSortOrder {
    Asc,
    Desc,
}

impl From<WasmSortOrder> for SortOrder {
    fn from(val: WasmSortOrder) -> Self {
        match val {
            WasmSortOrder::Asc => SortOrder::Asc,
            WasmSortOrder::Desc => SortOrder::Desc,
        }
    }
}

#[wasm_bindgen]
pub enum WasmTransactionFilter {
    All,
    Receive,
    Send,
}

impl From<WasmTransactionFilter> for TransactionFilter {
    fn from(val: WasmTransactionFilter) -> Self {
        match val {
            WasmTransactionFilter::All => TransactionFilter::All,
            WasmTransactionFilter::Receive => TransactionFilter::Receive,
            WasmTransactionFilter::Send => TransactionFilter::Send,
        }
    }
}
