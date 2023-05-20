use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn update_env(path: &Path) -> Result<(), String> {
    let file = File::open(path).map_err(|err| err.to_string())?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line.map_err(|err| err.to_string())?;
        let mut parts = line.splitn(2, '=');

        if let (Some(mut key), Some(value)) = (parts.next(), parts.next()) {
            if key.starts_with("export ") {
                key = &key[7..];
            }

            env::set_var(key, value);
        }
    }

    Ok(())
}
