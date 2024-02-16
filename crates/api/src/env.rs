use muon::{environment::ApiEnv, Product};

/// A local environment
#[derive(Debug, Default)]
pub struct LocalEnv {
    name: String,
    base: String,
}

const DEFAULT_LOCAL_PORT: u32 = 8080;

#[cfg(feature = "local")]
impl LocalEnv {
    const PINS: &'static [&'static str] = &[];

    /// Create a new atlas enviroment, possibly with a scientist name
    #[must_use]
    pub fn new(maybe_port: Option<u32>) -> Self {
        let port = maybe_port.unwrap_or(DEFAULT_LOCAL_PORT);

        Self {
            name: format!("localhost:{}", port),
            base: format!("http://localhost:{}", port),
        }
    }
}

#[cfg(feature = "local")]
impl ApiEnv for LocalEnv {
    fn name(&self) -> &str {
        &self.name
    }

    fn base(&self, _: &Product) -> &str {
        &self.base
    }

    fn pins(&self) -> &[&'static str] {
        Self::PINS
    }
}
