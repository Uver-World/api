use database::{Database, project::Project};
use rocket::{http::Status, get, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;
use crate::RequestError;
use crate::model::user_token::UserData;

/// Retrieve the organization informations from its unique identifier
#[openapi(tag = "Organizations")]
#[get("/<id>/projects/<project_id>", format = "application/json")]
pub async fn project_from_id(
    user_data: UserData,
    database: &State<Database>,
    project_id: String,
    id: String,
) -> Custom<Result<Json<Project>, Json<RequestError>>> {


    // If organization not found
    let organization = match database.organization_manager.from_id(&id).await {
        Ok(Some(organization)) => organization,
        Ok(None) => {
            return Custom(
                Status::NotFound,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    "Organization not found.".into(),
                ))
                .into()),
            )
        }
        Err(err) => {
            return Custom(
                Status::InternalServerError,
                Err(RequestError::from(Custom(
                    Status::InternalServerError,
                    err.to_string(),
                ))
                .into()),
            )
        }
    };

    // check if organization contains project
    if !organization.projects_ids.contains(&project_id) {
        return Custom(
            Status::NotFound,
            Err(RequestError::from(Custom(
                Status::NotFound,
                "Project not found.".into(),
            ))
            .into()),
        );
    }

    match database.project_manager.from_id(&project_id).await {
        Ok(Some(project)) => Custom(Status::Ok, Ok(Json(project))),
        Ok(None) => Custom(
            Status::NotFound,
            Err(RequestError::from(Custom(
                Status::NotFound,
                "Project not found.".into(),
            ))
            .into()),
        ),
        Err(err) => Custom(
            Status::InternalServerError,
            Err(RequestError::from(Custom(
                Status::InternalServerError,
                err.to_string(),
            ))
            .into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{Database};
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_from_unknow_organization() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let test_org = testing::get_org(database, &test_user).await;

            let response = dispatch_request(
                &client,
                Method::Get,
                format!(
                    "/organization/{}/projects/{}",
                    test_org.unique_id, "cdcdc"
                ),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
            let err = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(err.message, "Project not found.");
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_from_unknown_project() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let test_org = testing::get_org(database, &test_user).await;

            let response = dispatch_request(
                &client,
                Method::Get,
                format!(
                    "/organization/{}/projects/{}",
                    test_org.unique_id, "unknown_project"
                ),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
            let err = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(err.message, "Project not found.");
        })
        .await;
    }
}