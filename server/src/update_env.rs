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

#[cfg(test)]
mod tests {
    use std::{env, fs, io::Write, path::Path};

    use super::update_env;

    #[test]
    fn test_update_env() {
        let filename = "test.env";
        let content = "TEST=TEST2\nexport TEST3=TEST4\n";

        let _ = create_and_write_to_file(filename, content);

        let _ = update_env(Path::new(filename));

        // Delete the file
        let _ = fs::remove_file(filename);

        assert_eq!(env::var("TEST").unwrap(), "TEST2");
        assert_eq!(env::var("TEST3").unwrap(), "TEST4");
    }

    fn create_and_write_to_file(filename: &str, content: &str) -> std::io::Result<()> {
        // Create the file
        let path = Path::new(filename);
        let mut file = fs::File::create(path)?;

        // Write the content to the file
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}
