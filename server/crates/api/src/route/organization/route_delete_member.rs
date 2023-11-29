use database::{group::Group, Database, organization::Organization};
use rocket::{http::Status, delete, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{user_token::UserData, organization_member::OrganizationMember},
    RequestError,
};

/// Delete a member from the organization
///
/// Requires 'Website' group
#[openapi(tag = "Organizations")]
#[delete(
    "/<id>/members",
    data = "<body>",
    format = "application/json"
)]
pub async fn remove_member(
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

// check_member checks if the member is present in the organization and removes it
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
                    "The user is the owner of the organization.",
                )
            } else if !organization.member_ids.contains(&member.unique_id) {
                error_response(
                    Status::Conflict,
                    "The user is not a member of the organization.",
                )
            } else {
                remove_member_from_organization(database, organization.unique_id, member_id).await
            }
        }
        Ok(None) => error_response(Status::NotFound, "Member was not found."),
        Err(_) => error_response(Status::InternalServerError, "A database error occurred."),
    }
}

// remove_member_from_organization removes the member from the organization
async fn remove_member_from_organization(
    database: &State<Database>,
    organization_id: String,
    member_id: String,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    match database
        .organization_manager
        .remove_from_member_ids(&organization_id, &member_id)
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