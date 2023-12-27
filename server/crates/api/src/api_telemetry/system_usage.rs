use sys_info::mem_info;
use sys_info::{cpu_num, loadavg};

pub fn get_cpu_usage() -> f64 {
    let load = match loadavg() {
        Ok(load) => load.one, // 1-minute load average
        Err(_) => return 0.0,
    };

    let num_cpus = match cpu_num() {
        Ok(num) => num as f64,
        Err(_) => return 0.0,
    };

    // Calculate the average CPU usage per CPU
    (load / num_cpus * 100.0).min(100.0)
}

pub fn get_ram_usage() -> f64 {
    match mem_info() {
        Ok(mem) => mem.total as f64 - mem.free as f64, // Total - Free memory
        Err(_) => 0.0,
    }
}
