use rocket_okapi::openapi_get_routes;

pub mod auth;
pub mod cores;
pub mod games;
pub mod me;
pub mod platforms;
pub mod systems;
pub mod tags;
pub mod teams;
pub mod users;

pub fn routes() -> Vec<rocket::Route> {
    openapi_get_routes![
        auth::github_login,
        auth::google_login,
        auth::patreon_login,
        auth::login,
        auth::logout,
        auth::signup,
        cores::cores_create,
        cores::cores_details,
        cores::cores_list,
        cores::releases::cores_releases_artifacts_download,
        cores::releases::cores_releases_artifacts_download_filename,
        cores::releases::cores_releases_artifacts_list,
        cores::releases::cores_releases_artifacts_upload,
        cores::releases::cores_releases_create,
        cores::releases::cores_releases_list,
        games::games_add_artifact,
        games::games_create,
        games::games_details,
        games::games_images,
        games::games_images_upload,
        games::games_list,
        games::games_update,
        me::me,
        me::me_token,
        me::me_update,
        platforms::platforms_create,
        platforms::platforms_details,
        platforms::platforms_list,
        platforms::platforms_update,
        systems::systems_create,
        systems::systems_details,
        systems::systems_list,
        tags::tags,
        tags::tags_create,
        tags::tags_delete,
        teams::invite,
        teams::invite_accept,
        teams::teams,
        teams::teams_create,
        teams::teams_delete,
        teams::teams_details,
        teams::teams_update,
        users::check_username,
        users::users,
        users::users_details,
        users::users_update,
    ]
}
