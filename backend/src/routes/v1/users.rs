use crate::db::Db;
use crate::{guards, models, schema};
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, put};
use rocket_db_pools::diesel::prelude::*;
use rocket_okapi::openapi;

/// Check availability of a username. This is easier and less resource intensive than
/// getting the user's details and checking for 404.
#[openapi(tag = "Users", ignore = "db")]
#[post("/users/check?<username>")]
pub async fn check_username(
    mut db: Db,
    username: &str,
) -> Result<Json<dto::user::UserCheckResponse>, (Status, String)> {
    // Validate the username.
    dto::user::Username::new(username).map_err(|e| (Status::BadRequest, e.to_string()))?;

    let exists = schema::users::table
        .filter(schema::users::username.eq(username))
        .first::<models::User>(&mut db)
        .await
        .is_ok();

    // Negate the existence because we want to return `true` if the username is available.
    Ok(Json(dto::user::UserCheckResponse {
        username: username.to_string(),
        available: !exists,
    }))
}

/// List all users.
#[openapi(tag = "Users", ignore = "db")]
#[get("/users?<paging..>")]
pub async fn users(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::user::User>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    models::User::list(&mut db, page, limit)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))
        .map(|u| Json(u.into_iter().map(Into::into).collect()))
}

#[openapi(tag = "Users", ignore = "db")]
#[get("/users/<id>")]
pub async fn users_details(
    mut db: Db,
    id: dto::user::UserIdOrUsername<'_>,
) -> Result<Json<dto::user::UserDetails>, (Status, String)> {
    let (user, teams) = models::User::get_user_with_teams(&mut db, id)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "User not found".to_string()))?;

    if user.username.is_none() {
        return Err((Status::NotFound, "User not found".to_string()));
    }

    let teams = teams
        .into_iter()
        .map(|(id, name, slug, role)| dto::user::UserTeamRef {
            team: dto::teams::TeamRef { id, name, slug },
            role: role.into(),
        })
        .collect();

    Ok(Json(dto::user::UserDetails {
        teams,
        user: dto::user::UserDetailsInner {
            id: user.id,
            username: user.username.unwrap(),
            description: user.description,
            links: user.links,
            metadata: user.metadata,
        },
    }))
}

/// Only root users can update other users.
#[openapi(tag = "Users", ignore = "db")]
#[put("/users/<id>", rank = 1, format = "application/json", data = "<form>")]
pub async fn users_update(
    mut db: Db,
    _root_user: guards::users::RootUserGuard,
    id: dto::user::UserIdOrUsername<'_>,
    form: Json<dto::user::UserUpdate<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let user = models::User::from_userid(&mut db, id)
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;
    let user_guard = guards::users::UserGuard::from_model(user);

    user_guard
        .update(&mut db, form.into_inner())
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;

    Ok(Json(dto::Ok))
}
