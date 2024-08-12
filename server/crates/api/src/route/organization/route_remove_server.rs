use database::{Database};
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{organization_server::OrganizationServer, user_token::UserData},
    RequestError,
};

/// Register a new server on the organization
///
/// Requires 'Website' group
#[openapi(tag = "Organizations")]
#[post(
    "/remove_server",
    data = "<organization_server>",
    format = "application/json"
)]
pub async fn remove_server(
    _user_data: UserData,
    database: &State<Database>,
    organization_server: Json<OrganizationServer>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {


    check_organization(database, organization_server).await
}

async fn check_organization(
    database: &State<Database>,
    organization_server: Json<OrganizationServer>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    match database
        .organization_manager
        .from_id(&organization_server.organization_id)
        .await
    {
        Ok(Some(organization))
            if !organization
                .server_ids
                .contains(&organization_server.server_id) =>
        {
            Custom(
                Status::Ok,
                Err(RequestError::from(Custom(
                    Status::NotModified,
                    "Server is not present in the organization.".into(),
                ))
                .into()),
            )
        }
        Ok(Some(_organization)) => check_server(database, organization_server).await,
        Ok(None) => error_response(Status::NotFound, "Organization was not found."),
        Err(_) => error_response(Status::InternalServerError, "A database error occurred."),
    }
}

async fn check_server(
    database: &State<Database>,
    organization_server: Json<OrganizationServer>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    match database
        .user_manager
        .from_id(&organization_server.server_id)
        .await
    {
        Ok(Some(_server)) => {
            match database
                .organization_manager
                .remove_from_server_ids(
                    &organization_server.organization_id,
                    &organization_server.server_id,
                )
                .await
            {
                Ok(_) => Custom(Status::Ok, Ok(Json(true))),
                Err(_) => error_response(Status::InternalServerError, "A database error occurred."),
            }
        }
        Ok(Some(_)) => error_response(Status::UnprocessableEntity, "This is not a server."),
        Ok(None) => error_response(Status::NotFound, "Server not found."),
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

    use crate::{
        model::organization_server::OrganizationServer,
        testing::{self, dispatch_request, run_test},
    };

    #[rocket::async_test]
    async fn test_remove_server() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_server = testing::get_user(database).await;
            let test_user = testing::get_user(database).await;
            let test_org =
                testing::create_org(database, &test_user, vec![test_server.unique_id.clone()])
                    .await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let body = OrganizationServer {
                organization_id: test_org.unique_id.clone(),
                server_id: test_server.unique_id.clone(),
            };

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/organization/remove_server"),
                Some(serde_json::to_string(&body).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert!(response);

            let updated_org = database
                .organization_manager
                .from_id(&test_org.unique_id)
                .await
                .unwrap()
                .unwrap();

            assert!(!updated_org.server_ids.contains(&test_server.unique_id));
        })
        .await;
    }
}
