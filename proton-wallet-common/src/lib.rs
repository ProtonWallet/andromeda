// pub mod 
pub mod hello;
pub fn library_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
