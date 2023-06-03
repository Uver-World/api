use database::{
    group::Group,
    organization::{Organization, OrganizationUpdate},
    Database,
};
use rocket::{http::Status, response::status::Custom, serde::json::Json, *};
use rocket_okapi::openapi;

use crate::{
    model::{organization_init::OrganizationInit, user_token::UserData},
    Server,
};

mod route_delete_from_id;
mod route_from_id;

pub use route_delete_from_id::*;
pub use route_from_id::*;

/// Update the organization informations from its id
#[openapi(tag = "Organizations")]
#[patch("/<id>", data = "<organization_update>", format = "application/json")] // <- route attribute
pub async fn update(
    user_data: UserData,
    database: &State<Database>,
    id: String,
    organization_update: Json<Vec<OrganizationUpdate>>,
) -> Custom<Result<Json<bool>, String>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website, Group::Server]) {
        return Custom(response.0, Err(response.1));
    }

    match database
        .organization_manager
        .update_organization(id, organization_update.0)
        .await
    {
        Ok(_) => Custom(Status::Ok, Ok(Json(true))),
        Err(err) => Custom(Status::InternalServerError, Err(err.to_string())),
    }
}

/// Register a new organization
///
/// Requires 'Website' group
#[openapi(tag = "Organizations")]
#[post("/create", data = "<organization>", format = "application/json")] // <- route attribute
pub async fn create(
    user_data: UserData,
    database: &State<Database>,
    organization: Json<OrganizationInit>,
) -> Custom<String> {
    if let Err(response) = user_data.matches_group(vec![Group::Website]) {
        return Custom(response.0, response.1);
    }

    let raw_organization = organization.0;

    let user = database
        .user_manager
        .from_id(&raw_organization.owner_id)
        .await;
    if user.is_err() || user.unwrap().is_none() {
        return Custom(
            Status::NotFound,
            "Owner id does not correspond to any exisisting user.".into(),
        );
    }

    let organization = Organization {
        unique_id: Server::generate_unique_id().to_string(),
        creation_date: Server::current_time().to_string(),
        name: raw_organization.name,
        members_id: Vec::new(),
        owner_id: raw_organization.owner_id,
    };

    match database
        .organization_manager
        .create_organization(&organization)
        .await
    {
        Ok(_) => Custom(Status::Ok, organization.unique_id),
        Err(error) => Custom(Status::InternalServerError, error.to_string()),
    }
}
