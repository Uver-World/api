use std::collections::HashMap;
use std::env;
use std::future::Future;

use database::authentication::Authentication;
use database::group::Group;
use database::login::Login;
use database::organization::Organization;
use database::user::User;
use database::Database;
use rocket::http::{Header, Method};
use rocket::local::asynchronous::{Client, LocalResponse};
use testcontainers::clients::Cli;
use testcontainers::{core::WaitFor, Image};

use crate::{get_rocket, Server};

/// Creates an user with the desired group
/// Adds it to the database
/// Returns it
pub async fn create_user(
    database: &Database,
    group: Group,
    authentication: Authentication,
) -> User {
    let timestamp = Server::current_time();

    let user = User {
        authentication: authentication.clone(),
        unique_id: Server::generate_unique_id().to_string(),
        creation_date: timestamp.to_string(),
        logins: vec![Login::new(
            "127.0.0.1".to_string(),
            timestamp,
            authentication,
        )],
        username: format!("{group:?}"),
        group,
    };

    let _ = database.user_manager.create_user(&user).await;

    user
}

/// Creates an user with the desired group
/// Adds it to the database
/// Returns it
pub async fn get_user(database: &Database, group: Group) -> User {
    create_user(database, group, Authentication::None).await
}

pub async fn create_org(database: &Database, user: &User, server_ids: Vec<String>) -> Organization {
    let timestamp = Server::current_time();
    let unique_id = Server::generate_unique_id().to_string();

    let organization = Organization {
        unique_id: unique_id.clone(),
        creation_date: timestamp.to_string(),
        member_ids: Vec::new(),
        name: format!("name-{unique_id}"),
        owner_id: user.unique_id.clone(),
        server_ids,
    };

    let _ = database
        .organization_manager
        .create_organization(&organization)
        .await;

    organization
}

pub async fn get_org(database: &Database, user: &User) -> Organization {
    create_org(database, user, Vec::new()).await
}

pub async fn run_test<F, Fut>(lambda_func: F)
where
    F: Fn(Client) -> Fut,
    Fut: Future,
{
    let docker = Cli::docker();
    let container = &docker.run(MongoContainer::default_env());
    set_test_env(container.get_host_port_ipv4(27017));
    let client = Client::tracked(get_rocket()).await.unwrap();

    lambda_func(client).await;
}

pub async fn dispatch_request(
    client: &Client,
    method: Method,
    uri: String,
    body: Option<String>,
    token: Option<String>,
) -> LocalResponse {
    let mut request = match method {
        Method::Get => client.get(uri),
        Method::Post => client
            .post(uri)
            .body(body.unwrap_or_default())
            .header(Header::new("content-type", "application/json")),
        Method::Patch => client
            .patch(uri)
            .body(body.unwrap_or_default())
            .header(Header::new("content-type", "application/json")),
        Method::Delete => client.delete(uri),
        _ => panic!("Unsupported HTTP method"),
    };

    if let Some(token) = token {
        request = request.header(Header::new("X-User-Token", token))
    }
    request.dispatch().await
}

fn set_test_env(mongo_port: u16) {
    env::set_var("MONGODB_HOSTNAME", "127.0.0.1");
    env::set_var("MONGODB_PORT", mongo_port.to_string());
    env::set_var("MONGODB_USERNAME", "test");
    env::set_var("MONGODB_PASSWORD", "test");
    env::set_var("MONGODB_DATABASE", "test");
    env::set_var("OTEL_RESOURCE_ATTRIBUTES", "test");
    env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "test");
    env::set_var("OTEL_EXPORTER_OTLP_TOKEN", "test");
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
