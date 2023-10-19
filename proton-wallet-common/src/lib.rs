// pub mod 

pub fn library_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
