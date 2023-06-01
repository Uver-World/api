use std::collections::HashMap;
use std::env;

use database::authentication::Authentication;
use database::group::Group;
use database::login::Login;
use database::managers::UserManager;
use database::user::User;
use rocket::{Build, Rocket};
use testcontainers::Container;
use testcontainers::{core::WaitFor, Image};

use crate::{get_rocket, Server};

pub fn get_test_rocket(container: &Container<MongoContainer>) -> Rocket<Build> {
    set_test_env(container.get_host_port_ipv4(27017));

    get_rocket()
}

/// Creates an user with the desired group
/// Adds it to the database
/// Returns it
pub async fn get_user(user_manager: &UserManager, group: Group) -> User {
    let timestamp = Server::current_time();

    let user = User {
        authentication: Authentication::None,
        unique_id: format!("{group:?}-ID"),
        creation_date: timestamp.to_string(),
        logins: vec![Login::new(
            "127.0.0.1".to_string(),
            timestamp,
            Authentication::None,
        )],
        username: format!("{group:?}"),
        group,
    };

    let _ = user_manager.create_user(&user).await;

    user
}

fn set_test_env(mongo_port: u16) {
    env::set_var("MONGODB_HOSTNAME", "127.0.0.1");
    env::set_var("MONGODB_PORT", mongo_port.to_string());
    env::set_var("MONGODB_USERNAME", "test");
    env::set_var("MONGODB_PASSWORD", "test");
    env::set_var("MONGODB_DATABASE", "test");
}

#[derive(Debug, Default)]
pub struct MongoContainer {
    env_vars: HashMap<String, String>,
}

impl MongoContainer {
    pub fn default_env() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("MONGO_INITDB_ROOT_USERNAME".to_string(), "test".to_string());
        env_vars.insert("MONGO_INITDB_ROOT_PASSWORD".to_string(), "test".to_string());

        Self { env_vars }
    }
}

impl Image for MongoContainer {
    type Args = ();

    fn name(&self) -> String {
        "mongo".to_owned()
    }

    fn tag(&self) -> String {
        "latest".to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Waiting for connections")]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![27017]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}
