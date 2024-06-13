use std::sync::{Arc, RwLock};

use muon::Client;

pub struct QuarkCommand {
    pub session: Arc<RwLock<Client>>,
}
