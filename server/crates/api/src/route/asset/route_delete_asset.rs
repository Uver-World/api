// #[openapi(tag = "Assets")]
// #[delete("/<id>")]
// pub async fn delete_asset(
//     database: &State<Database>,
//     id: String,
// ) -> Custom<Result<String, Json<RequestError>>> {
//     let asset_manager = &database.asset_manager;

//     match asset_manager.delete_asset(&id).await {
//         Ok(_) => Custom(Status::Ok, Ok(format!("Asset {} deleted", id))),
//         Err(err) => Custom(
//             Status::InternalServerError,
//             Err(Json(RequestError::from(err.to_string()))),
//         ),
//     }
// }
