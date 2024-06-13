// use andromeda_api::Product;

// /// A local environment
// #[derive(Debug, Default)]
// pub struct BrowserOriginEnv {
//     name: String,
//     base: String,
// }

// impl BrowserOriginEnv {
//     const PINS: &'static [&'static str] = &[];

//     /// Create a new atlas enviroment, possibly with a scientist name
//     #[must_use]
//     pub fn new(origin: String) -> Self {
//         Self {
//             name: origin.clone(),
//             base: origin.clone(),
//         }
//     }
// }
// impl ApiEnv for BrowserOriginEnv {
//     fn name(&self) -> &str {
//         &self.name
//     }

//     fn base(&self, _: &Product) -> &str {
//         &self.base
//     }

//     fn pins(&self) -> &[&'static str] {
//         Self::PINS
//     }
// }
