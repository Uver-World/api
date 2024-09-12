//#![feature(coverage_attribute)]

use clap::{Arg, Command};
use rocket::*;

//#[coverage(off)]
fn export_env() -> Result<(), String> {
    let app = Command::new("").subcommand_negates_reqs(true).arg(
        Arg::new("envfile")
            .help("Export the .env file to environment variables before run")
            .long("envfile")
            .short('e'),
    );
    let matches = app.get_matches();
    let env_file: Option<&String> = matches.get_one("envfile");

    if let Some(env_file) = env_file {
        dotenv::from_filename(env_file).map_err(|err| err.to_string())?;
    }
    Ok(())
}

//#[coverage(off)]
#[launch]
async fn rocket() -> _ {
    let env = export_env();
    
    if env.is_err() {
        eprintln!("{:?}", env.err());
        std::process::exit(0);
    }

    api::get_rocket()
}
