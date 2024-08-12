use database::{Database, organization::Organization};
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
    _user_data: UserData,
    database: &State<Database>,
    id: String,
    body: Json<OrganizationMember>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {


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

#[cfg(test)]
mod tests {

    use database::{Database};
    use rocket::http::{Method, Status};
    use serde_json::json;

    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn test_unknow_organization() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Delete,
                format!("/organization/id/{}/members", "unknown"),
                Some(serde_json::to_string(&json!({
                    "member_id": test_user.unique_id
                })).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_unknow_member() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let organization = testing::get_org(database, &test_user).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Delete,
                format!("/organization/id/{}/members", organization.unique_id),
                Some(serde_json::to_string(&json!({
                    "member_id": "unknow"
                })).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        })
        .await;
    }
}