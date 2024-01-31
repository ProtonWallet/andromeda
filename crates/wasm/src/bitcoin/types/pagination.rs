use proton_wallet_bitcoin::transactions::Pagination;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmPagination {
    pub skip: usize,
    pub take: usize,
}

#[wasm_bindgen]
impl WasmPagination {
    #[wasm_bindgen(constructor)]
    pub fn new(skip: usize, take: usize) -> Self {
        WasmPagination { skip, take }
    }
}

impl Into<Pagination> for WasmPagination {
    fn into(self) -> Pagination {
        Pagination::new(self.skip, self.take)
    }
}
