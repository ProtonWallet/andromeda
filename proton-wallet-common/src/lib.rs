// pub mod 
pub mod hello;
// pub mod keys;
pub fn library_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
