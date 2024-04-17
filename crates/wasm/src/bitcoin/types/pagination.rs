use andromeda_bitcoin::{transactions::Pagination, utils::SortOrder};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmPagination {
    skip: usize,
    take: usize,
}

impl Into<Pagination> for WasmPagination {
    fn into(self) -> Pagination {
        Pagination::new(self.skip, self.take)
    }
}

#[wasm_bindgen]
pub enum WasmSortOrder {
    Asc,
    Desc,
}

impl Into<SortOrder> for WasmSortOrder {
    fn into(self) -> SortOrder {
        match self {
            WasmSortOrder::Asc => SortOrder::Asc,
            WasmSortOrder::Desc => SortOrder::Desc,
        }
    }
}
