use std::time::SystemTime;

use rand::Rng;
use serde::Serialize;

#[derive(Serialize)]
pub struct Host {
    host: String,
}

/// Project server structure
/// All services are registered here
pub struct Server {}

impl Server {
    pub fn current_time() -> u128 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }

    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_unique_id() -> u64 {
        let now = Self::current_time;
        let random_number = rand::thread_rng().gen_range(0..1_000_000_000_000_000_000);
        now as u64 + random_number
    }
}
