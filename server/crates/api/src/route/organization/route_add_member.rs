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

#[cfg(test)]
mod tests {

    use database::{group::Group, Database};
    use rocket::http::{Method, Status};
    use serde_json::json;

    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn test_unknow_organization() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::Website).await;
            let request_user = testing::get_user(database, Group::Website).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/organization/id/{}/members", "unknow"),
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
            let test_user = testing::get_user(database, Group::Website).await;
            let organization = testing::get_org(database, &test_user).await;
            let request_user = testing::get_user(database, Group::Website).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
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