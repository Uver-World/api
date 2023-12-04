use database::{group::Group, project::ProjectUpdateData, Database};
use rocket::{http::Status, patch, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Update the project informations from its id
#[openapi(tag = "Organizations")]
#[patch("/<id>/projects", data = "<project_update>", format = "application/json")] // <- route attribute
pub async fn update_project(
    user_data: UserData,
    database: &State<Database>,
    id: String,
    project_update: Json<ProjectUpdateData>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }

    // If organization not found 
    match database
        .organization_manager
        .from_id(id.as_str())
        .await
    {
        Ok(Some(_)) => (),
        Ok(None) => {
            return Custom(
                Status::Ok,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    format!("Organization not found with id: {id}"),
                ))
                .into()),
            )
        }
        Err(err) => {
            return Custom(
                Status::Ok,
                Err(RequestError::from(Custom(Status::InternalServerError, err.to_string())).into()),
            )
        }
    }

    // if project not found
    match database
        .project_manager
        .from_id(project_update.0.project_id.as_str())
        .await
    {
        Ok(Some(_)) => (),
        Ok(None) => {
            return Custom(
                Status::Ok,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    format!("Project not found with id: {id}", id = project_update.0.project_id),
                ))
                .into()),
            )
        }
        Err(err) => {
            return Custom(
                Status::Ok,
                Err(RequestError::from(Custom(Status::InternalServerError, err.to_string())).into()),
            )
        }
    }

    match database
        .project_manager
        .update_project(&project_update.0.project_id, project_update.0.project_update)
        .await
    {
        Ok(_)=> Custom(Status::Ok, Ok(Json(true))),
        Err(err) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::InternalServerError,
                err.to_string(),
            ))
            .into()),
        ),
    }

}