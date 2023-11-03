pub use proton_wallet_common::keys::gen_mnemonic;
pub use proton_wallet_common::WordCount;
pub struct Keys;
impl Keys {
    pub fn new() -> Self {
        Self
    }

    pub fn gen_gnemonic(&self, count: WordCount) -> String {
        gen_mnemonic(count)
    }
}
