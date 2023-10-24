pub use proton_wallet_common::keys::gen_mnemonic;
pub struct Keys;
impl Keys {
    pub fn new() -> Self {
        Self
    }

    pub fn gen_gnemonic(&self) -> String {
        gen_mnemonic()
    }
}
