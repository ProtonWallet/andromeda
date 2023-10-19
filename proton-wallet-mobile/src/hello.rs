pub use proton_wallet_common::hello::helloworld;
pub struct Hello;
impl Hello {
    pub fn new() -> Self {
        Self
    }

    pub fn helloworld(&self) -> String {
        helloworld()
    }
}
