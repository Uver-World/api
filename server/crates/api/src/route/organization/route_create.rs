use database::{group::Group, organization::Organization, Database};
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{organization_init::OrganizationInit, user_token::UserData},
    RequestError, Server,
};

/// Register a new organization
///
/// Requires 'Website' group
#[openapi(tag = "Organizations")]
#[post("/create", data = "<organization>", format = "application/json")] // <- route attribute
pub async fn create(
    user_data: UserData,
    database: &State<Database>,
    organization: Json<OrganizationInit>,
) -> Custom<Result<Json<String>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }

    let raw_organization = organization.0;

    let user = database
        .user_manager
        .from_id(&raw_organization.owner_id)
        .await;
    if user.is_err() || user.unwrap().is_none() {
        return Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::NotFound,
                "Owner id does not correspond to any exisisting user.".into(),
            ))
            .into()),
        );
    }

    let organization = Organization {
        unique_id: Server::generate_unique_id().to_string(),
        creation_date: Server::current_time().to_string(),
        name: raw_organization.name,
        member_ids: Vec::new(),
        owner_id: raw_organization.owner_id,
        server_ids: Vec::new(),
    };

    match database
        .organization_manager
        .create_organization(&organization)
        .await
    {
        Ok(_) => Custom(Status::Ok, Ok(Json(organization.unique_id))),
        Err(error) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(Status::InternalServerError, error.to_string())).into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{group::Group, Database};
    use rocket::http::{Method, Status};

    use crate::{
        model::organization_init::OrganizationInit,
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_create() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let request_user = testing::get_user(database, Group::Website).await;
            let request_token = request_user.get_token().unwrap();
            let body = OrganizationInit::new("Test organization".to_string(), test_user.unique_id);

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/organization/create"),
                Some(serde_json::to_string(&body).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response_id = response.into_json().await.unwrap();
            let created_org = database
                .organization_manager
                .from_id(&response_id)
                .await
                .unwrap()
                .unwrap();

            assert_eq!(created_org.name, body.name);
            assert_eq!(created_org.owner_id, body.owner_id);
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_unknown_create() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database, Group::Website).await;
            let request_token = request_user.get_token().unwrap();
            let body = OrganizationInit::new("Test organization".to_string(), "NO_ID".to_string());

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/organization/create"),
                Some(serde_json::to_string(&body).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let request_error = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(request_error.code, 404);
            assert_eq!(
                request_error.message,
                format!("Owner id does not correspond to any exisisting user.")
            );

            // No organization should have been created.
            assert!(!database
                .organization_manager
                .organization_exists(body.name)
                .await
                .unwrap());
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_create() {
        _unauthorized_test_create(Group::User).await;
        _unauthorized_test_create(Group::Server).await;
    }

    async fn _unauthorized_test_create(request_group: Group) {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let request_user = testing::get_user(database, request_group).await;
            let request_token = request_user.get_token().unwrap();
            let body = OrganizationInit::new("Test organization".to_string(), test_user.unique_id);

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/organization/create"),
                Some(serde_json::to_string(&body).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);

            // No organization should have been created.
            assert!(!database
                .organization_manager
                .organization_exists(body.name)
                .await
                .unwrap());
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_update() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let body = OrganizationInit::new("Test organization".to_string(), test_user.unique_id);

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/organization/create"),
                Some(serde_json::to_string(&body).unwrap()),
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);

            // No organization should have been created.
            assert!(!database
                .organization_manager
                .organization_exists(body.name)
                .await
                .unwrap());
        })
        .await;
    }
}
