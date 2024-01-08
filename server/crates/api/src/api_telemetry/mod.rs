mod system_usage;
mod fairing;

use database::Database;
use rocket::tokio::runtime::Runtime;
use system_usage::*;
use std::thread;
use opentelemetry::global;

pub use fairing::*;

pub fn start(database: Database) {
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        
        rt.block_on(async {
            loop {
                cpu_telemetry();
                ram_telemetry();
                user_telemetry(&database).await;
                peer_telemetry(&database).await;
            }
        });
    });
}

async fn user_telemetry(database: &Database) {
    let number_of_users = database.user_manager.users.count_documents(None, None).await.unwrap();
    let meter = global::meter("api");
    let cpu_gauge = meter.u64_observable_gauge("users")
        .with_description("Number of users")
        .init();
    cpu_gauge.observe(number_of_users, [].as_ref());
}

async fn peer_telemetry(database: &Database) {
    let number_of_peers = database.peers_manager.peers.count_documents(None, None).await.unwrap();
    let meter = global::meter("api");
    let cpu_gauge = meter.u64_observable_gauge("peers")
        .with_description("Number of peers")
        .init();
    cpu_gauge.observe(number_of_peers, [].as_ref());
}

fn cpu_telemetry() {
    let cpu_usage = get_cpu_usage();
    let meter = global::meter("api");
    let cpu_gauge = meter.f64_observable_gauge("cpu_usage")
        .with_description("CPU usage percentage")
        .init();
    cpu_gauge.observe(cpu_usage, [].as_ref());
}

fn ram_telemetry() {
    let ram_usage = get_ram_usage();
    let meter = global::meter("api");
    let ram_gauge = meter.f64_observable_gauge("ram_usage")
        .with_description("RAM usage in megabytes")
        .init();
    ram_gauge.observe(ram_usage, [].as_ref());
}