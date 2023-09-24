use database::{group::Group, peer::Peer, Database};
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{server_id::ServerId, user_token::UserData},
    RequestError,
};

#[openapi(tag = "Users")]
#[post("/access_server", data = "<server_id>", format = "application/json")] // <- route attribute
pub async fn access_server(
    user_data: UserData,
    database: &State<Database>,
    server_id: Json<ServerId>,
) -> Custom<Result<Json<Peer>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::User]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }
    let server_id = server_id.0;
    match database.user_manager.from_id(&server_id.0).await {
        Ok(user) if user.is_some() => {
            match database
                .organization_manager
                .has_access_to_server(&server_id.0, &user_data.id.unwrap())
                .await
            {
                Ok(true) => match database.peers_manager.from_server_id(&server_id.0).await {
                    Ok(Some(peer)) => Custom(Status::Ok, Ok(Json(peer))),
                    Ok(None) => Custom(
                        Status::Ok,
                        Err(RequestError::from(Custom(
                            Status::NotFound,
                            format!("The server is currently offline."),
                        ))
                        .into()),
                    ),
                    Err(_) => Custom(
                        Status::Ok,
                        Err(RequestError::from(Custom(
                            Status::InternalServerError,
                            format!("A database error occured."),
                        ))
                        .into()),
                    ),
                },
                Ok(false) => Custom(
                    Status::Ok,
                    Err(RequestError::from(Custom(
                        Status::Unauthorized,
                        format!("You are not part of any organization of this server."),
                    ))
                    .into()),
                ),
                Err(_) => Custom(
                    Status::Ok,
                    Err(RequestError::from(Custom(
                        Status::InternalServerError,
                        format!("A database error occured."),
                    ))
                    .into()),
                ),
            }
        }
        _ => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("Server not found with id: {}", server_id.0),
            ))
            .into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{group::Group, peer::Peer, Database};
    use rocket::http::{Method, Status};

    use crate::{
        model::server_id::ServerId,
        testing::{self, dispatch_request, run_test},
        Server,
    };

    #[rocket::async_test]
    async fn test_access_server() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_server = testing::get_user(database, Group::Server).await;
            let test_user = testing::get_user(database, Group::User).await;
            let _test_org =
                testing::create_org(database, &test_user, vec![test_server.unique_id.clone()])
                    .await;

            let server_peer = Peer {
                room_id: Server::generate_unique_id().to_string(),
                creation_date: Server::current_time().to_string(),
                signaling_hostname: "127.0.0.1".to_string(),
                signaling_port: 3536,
                server_unique_id: test_server.unique_id.clone(),
            };

            let _ = database
                .peers_manager
                .create_peer(&server_peer)
                .await
                .unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/access_server"),
                Some(serde_json::to_string(&ServerId(test_server.unique_id)).unwrap()),
                Some(test_user.get_token().unwrap().to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let _ = response.into_json::<Peer>().await.unwrap();
        })
        .await;
    }
}
