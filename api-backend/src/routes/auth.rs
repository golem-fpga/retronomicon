use crate::db::Db;
use crate::guards::users::UserGuard;
use crate::models;
use crate::RetronomiconConfig;
use anyhow::{Context, Error};
use diesel::OptionalExtension;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use retronomicon_dto::AuthTokenResponse;
use rocket::http::CookieJar;
use rocket::response::{Debug, Redirect};
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route, State};
use rocket_db_pools::diesel::AsyncConnection;
use rocket_oauth2::{OAuth2, TokenResponse};
use scoped_futures::ScopedFutureExt;
use serde_json::{self, Value};

#[post("/me/token")]
async fn me_token(user: UserGuard) -> Result<Json<AuthTokenResponse>, String> {
    user.create_jwt()
        .map(|token| Json(AuthTokenResponse { token }))
        .map_err(|e| e.to_string())
}

async fn login_(
    mut db: Db,
    cookies: &CookieJar<'_>,
    config: &State<RetronomiconConfig>,
    username: Option<String>,
    email: &str,
    auth_provider: &str,
) -> Result<Redirect, Debug<Error>> {
    let add_to_root = config.root_team.iter().any(|u| u == email);

    db.transaction::<Redirect, diesel::result::Error, _>(|db| {
        async move {
            let maybe_user =
                UserGuard::login_from_auth(db, username, email, auth_provider.to_string(), None)
                    .await
                    .optional()?;

            let user = if let Some((created, model, user)) = maybe_user {
                if created && add_to_root {
                    model
                        .add_team(db, config.root_team_id, models::UserTeamRole::Owner)
                        .await?;
                }
                user
            } else {
                let maybe_user =
                    UserGuard::login_from_auth(db, None, email, auth_provider.to_string(), None)
                        .await
                        .optional()?;
                if let Some((created, model, user)) = maybe_user {
                    if created && add_to_root {
                        model
                            .add_team(db, config.root_team_id, models::UserTeamRole::Owner)
                            .await?;
                    }
                    user
                } else {
                    return Err(diesel::result::Error::NotFound);
                }
            };

            user.update_cookie(cookies);

            let base_url = config.base_url.clone();
            Ok(Redirect::to(base_url))
        }
        .scope_boxed()
    })
    .await
    .context("Adding team")
    .map_err(Debug)
}

/// User information to be retrieved from the GitHub API.
#[derive(serde::Deserialize)]
pub struct GitHubUserInfo {
    login: String,
    email: String,
}

// NB: Here we are using the same struct as a type parameter to OAuth2 and
// TokenResponse as we use for the user's GitHub login details. For
// `TokenResponse` and `OAuth2` the actual type does not matter; only that they
// are matched up.
#[get("/login/github")]
fn github_login(oauth2: OAuth2<GitHubUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2.get_redirect(cookies, &["user:read"]).unwrap()
}

#[get("/auth/github")]
async fn github_callback(
    db: Db,
    token: TokenResponse<GitHubUserInfo>,
    cookies: &CookieJar<'_>,
    config: &State<RetronomiconConfig>,
) -> Result<Redirect, Debug<Error>> {
    let json: Value = reqwest::Client::builder()
        .build()
        .context("failed to build reqwest client")?
        .get("https://api.github.com/user")
        .header(AUTHORIZATION, format!("token {}", token.access_token()))
        .header(ACCEPT, "application/vnd.github.v3+json")
        .header(USER_AGENT, "rocket_oauth2 demo application")
        .send()
        .await
        .context("failed to complete request")?
        .json()
        .await
        .context("failed to deserialize response")?;

    // Use the token to retrieve the user's GitHub account information.
    let user_info: GitHubUserInfo = serde_json::from_str(&json.to_string()).unwrap();

    login_(
        db,
        cookies,
        config,
        Some(user_info.login),
        &user_info.email,
        "github",
    )
    .await
}

/// User information to be retrieved from the Google People API.
#[derive(serde::Deserialize)]
pub struct GoogleUserInfo {
    #[allow(unused)]
    names: Vec<Value>,

    #[serde(default, rename = "emailAddresses")]
    email_addresses: Vec<Value>,
}

#[get("/login/google")]
fn google_login(oauth2: OAuth2<GoogleUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect(cookies, &["profile", "email", "openid"])
        .unwrap()
}

#[get("/auth/google")]
async fn google_callback(
    db: Db,
    token: TokenResponse<GoogleUserInfo>,
    cookies: &CookieJar<'_>,
    frontend_config: &State<RetronomiconConfig>,
) -> Result<Redirect, Debug<Error>> {
    let json: serde_json::Value = reqwest::Client::builder()
        .build()
        .context("failed to build reqwest client")?
        .get("https://people.googleapis.com/v1/people/me?personFields=names,emailAddresses")
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
        .send()
        .await
        .context("failed to complete request")?
        .json()
        .await
        .context("failed to deserialize response")?;

    // Use the token to retrieve the user's Google account information.
    let user_info: GoogleUserInfo = serde_json::from_str(&json.to_string()).unwrap();
    let email = user_info.email_addresses[0]
        .get("value")
        .and_then(|e| e.as_str());

    if let Some(email) = email {
        login_(db, cookies, frontend_config, None, email, "google").await
    } else {
        Err(Debug(Error::msg("failed to get email")))
    }
}

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>, user: UserGuard) -> Redirect {
    user.remove_cookie(cookies);
    Redirect::to("/")
}

pub fn routes() -> Vec<Route> {
    routes![
        me_token,
        logout,
        github_callback,
        google_callback,
        github_login,
        google_login,
    ]
}
