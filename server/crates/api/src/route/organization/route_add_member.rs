use database::{group::Group, Database, organization::Organization};
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{user_token::UserData, organization_member::OrganizationMember},
    RequestError,
};

/// Register a new member on the organization
///
/// Requires 'Website' group
#[openapi(tag = "Organizations")]
#[post(
    "/<id>/members",
    data = "<body>",
    format = "application/json"
)]
pub async fn add_member(
    user_data: UserData,
    database: &State<Database>,
    id: String,
    body: Json<OrganizationMember>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }

    // Check if the organization exists
    match database.organization_manager.from_id(&id).await {
        Ok(Some(organization)) => check_member(database, organization, body.member_id.clone()).await,
        Ok(None) => error_response(Status::NotFound, "Organization was not found."),
        Err(_) => error_response(Status::InternalServerError, "A database error occurred."),
    }
}

// check_member checks if the member exists, if is not already a member or the owner and adds it to the organization
async fn check_member(
    database: &State<Database>,
    organization: Organization,
    member_id: String,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    match database.user_manager.from_id(&member_id).await {
        Ok(Some(member)) => {
            if organization.owner_id == member.unique_id {
                error_response(
                    Status::Conflict,
                    "The user is already the owner of the organization.",
                )
            } else if organization.member_ids.contains(&member.unique_id) {
                error_response(
                    Status::Conflict,
                    "The user is already a member of the organization.",
                )
            } else {
                add_member_to_organization(database, organization.unique_id, member_id).await
            }
        }
        Ok(None) => error_response(Status::NotFound, "Member was not found."),
        Err(_) => error_response(Status::InternalServerError, "A database error occurred."),
    }
}

// add_member_to_organization adds the member to the organization
async fn add_member_to_organization(
    database: &State<Database>,
    organization_id: String,
    member_id: String,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    match database
        .organization_manager
        .add_member(&organization_id, &member_id)
        .await
    {
        Ok(_) => Custom(Status::Ok, Ok(Json(true))),
        Err(_) => error_response(Status::InternalServerError, "A database error occurred."),
    }
}

fn error_response(status: Status, message: &str) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    Custom(
        Status::Ok,
        Err(RequestError::from(Custom(status, message.into())).into()),
    )
}