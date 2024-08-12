use database::{Database, project::Project};
use rocket::{http::Status, get, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;
use crate::RequestError;
use crate::model::user_token::UserData;

/// Get all the projects of a specific organization.
/// 
/// Requires 'Website' group
#[openapi(tag = "Organizations")]
#[get("/<id>/projects", format = "application/json")]
pub async fn get_projects_from_organization(
    _user_data: UserData,
    database: &State<Database>,
    id: String,
) -> Custom<Result<Json<Vec<Project>>, Json<RequestError>>> {


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

    let projects = match database.project_manager.from_organization_id(&organization.unique_id).await {
        Ok(projects) => projects,
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

    Custom(Status::Ok, Ok(Json(projects)))
}

#[cfg(test)]
mod tests {

    use database::{Database};
    use rocket::http::{Method, Status};
    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn get_projects_from_organization_not_found() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;

            let response = dispatch_request(
                &client,
                Method::Get,
                "/organizations/123/projects".to_string(),
                None,
                Some(test_user.get_token().unwrap().to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        })
        .await;
    }
}