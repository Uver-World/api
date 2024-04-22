use database::{group::Group, Database};
use rocket::{http::Status, delete, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;
use crate::RequestError;
use crate::model::user_token::UserData;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, JsonSchema, Deserialize)]
pub struct DeleteProject {
    pub project_id: String,
}

/// Delete project of an organization.
/// 
/// Requires 'Website' group
#[openapi(tag = "Organizations")]
#[delete("/<id>/projects", data = "<project_data>", format = "application/json")]
pub async fn delete_project(
    user_data: UserData,
    database: &State<Database>,
    project_data: Json<DeleteProject>,
    id: String,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }

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

    let project = match database.project_manager.from_id(&project_data.project_id).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            return Custom(
                Status::NotFound,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    "Project not found.".into(),
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

    if project.organization_id != organization.unique_id {
        return Custom(
            Status::BadRequest,
            Err(RequestError::from(Custom(
                Status::BadRequest,
                "Project does not belong to this organization.".into(),
            ))
            .into()),
        );
    }

    match database
        .organization_manager
        .remove_from_projects_ids(&organization.unique_id, &project.unique_id)
        .await
    {
        Ok(_) => {}
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
    }

    match database.project_manager.delete_from_id(&project_data.project_id).await {
        Ok(Some(_)) => Custom(Status::Ok, Ok(Json(true))),
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
                Method::Delete,
                format!("/organization/id/{}/projects", "unknown"),
                Some(serde_json::to_string(&json!({
                    "project_id": test_user.unique_id
                })).unwrap()),
                Some(request_token.to_string()),
            )
            .await;
            
            assert_eq!(response.status(), Status::NotFound);
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_unknow_project() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::Website).await;
            let request_user = testing::get_user(database, Group::Website).await;
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
}