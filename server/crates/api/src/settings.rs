use std::env;

use database::DatabaseSettings;

#[derive(Clone)]
pub struct ApiSettings {
    pub database: DatabaseSettings,
}

impl ApiSettings {
    pub fn retrieve() -> Self {
        Self {
            database: get_database(),
        }
    }
}

fn get_database() -> DatabaseSettings {
    DatabaseSettings {
        hostname: env::var("MONGODB_HOSTNAME").unwrap().trim_end().to_string(),
        port: env::var("MONGODB_PORT")
            .unwrap()
            .trim_end()
            .parse()
            .unwrap_or(0),
        username: env::var("MONGODB_USERNAME").unwrap().trim_end().to_string(),
        password: env::var("MONGODB_PASSWORD").unwrap().trim_end().to_string(),
        database: env::var("MONGODB_DATABASE").unwrap().trim_end().to_string(),
    }
}
