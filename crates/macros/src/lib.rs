use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(WasmJson)]
pub fn derive_wasm_json(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let gen = quote! {
        #[wasm_bindgen]
        impl #name {
            #[wasm_bindgen(js_name = fromJson)]
            pub fn from_json(js: &wasm_bindgen::JsValue) -> Result<Self, wasm_bindgen::JsValue> {
                serde_wasm_bindgen::from_value(js.clone()).map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
            }

            #[wasm_bindgen(js_name = toJson)]
            pub fn to_json(&self) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                serde_wasm_bindgen::to_value(self).map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
            }
        }
    };

    gen.into()
}
