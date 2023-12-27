mod system_usage;
mod fairing;

use system_usage::*;
use std::thread;
use opentelemetry::global;

pub use fairing::*;

pub fn start() {
    thread::spawn(move || {
        loop {
            cpu_telemetry();
            ram_telemetry();
        }    
    });
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