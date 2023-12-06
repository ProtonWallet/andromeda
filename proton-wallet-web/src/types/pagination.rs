use proton_wallet_common::account::Pagination;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmPagination {
    pub skip: u32,
    pub take: u32,
}

#[wasm_bindgen]
impl WasmPagination {
    #[wasm_bindgen(constructor)]
    pub fn new(skip: u32, take: u32) -> Self {
        WasmPagination { skip, take }
    }
}

impl Into<Pagination> for WasmPagination {
    fn into(self) -> Pagination {
        Pagination::new(self.skip, self.take)
    }
}
