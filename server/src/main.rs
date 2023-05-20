use clap::{Arg, Command};
use rocket::*;
use std::path::Path;

mod update_env;

fn export_env() -> Result<(), String> {
    let app = Command::new("").subcommand_negates_reqs(true).arg(
        Arg::new("envfile")
            .help("Export the .env file to environment variables before run")
            .long("envfile")
            .short('e'),
    );
    let matches = app.clone().get_matches();
    let env_file: Option<&String> = matches.get_one("envfile");

    if let Some(env_file) = env_file {
        let env_path = Path::new(env_file);
        if !env_path.exists() {
            return Err(format!("environment file: '{env_file}' does not exist"));
        }
        return update_env::update_env(env_path);
    }
    Ok(())
}

#[launch]
async fn rocket() -> _ {
    let env = export_env();
    if env.is_err() {
        eprintln!("{:?}", env.err());
        std::process::exit(0);
    }

    api::get_rocket()
}
