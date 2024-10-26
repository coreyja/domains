use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use cja::{app_state::AppState as _, server::session::DBSession};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_cookies::Cookie;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginCallback {
    state: String,
}

pub async fn show(session: Option<DBSession>) -> axum::response::Response {
    if session.is_none() {
        let idp_url =
            std::env::var("COREYJA_IDP_URL").unwrap_or_else(|_| "https://coreyja.com".into());
        let login_url = format!("{}/login/domains", idp_url);
        Redirect::temporary(&login_url).into_response()
    } else {
        Redirect::temporary("/").into_response()
    }
}

pub async fn callback(
    cookies: tower_cookies::Cookies,
    Query(query): Query<LoginCallback>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let idp_url = std::env::var("COREYJA_IDP_URL").unwrap_or_else(|_| "https://coreyja.com".into());
    let client = reqwest::Client::new();

    #[derive(Debug, Serialize, Deserialize)]
    struct Claim {
        sub: String,
        exp: usize,
    }

    let key = std::env::var("AUTH_PRIVATE_KEY").unwrap();
    let token = jsonwebtoken::encode(
        &Header::new(Algorithm::RS256),
        &Claim {
            sub: query.state,
            exp: (chrono::Utc::now() + chrono::Duration::minutes(1)).timestamp() as usize,
        },
        &EncodingKey::from_rsa_pem(key.as_bytes()).unwrap(),
    )
    .unwrap();

    let claim_url = format!("{}/login/domains", idp_url);

    let resp = client
        .post(claim_url)
        .json(&json!({ "jwt": token }))
        .send()
        .await
        .unwrap();

    #[derive(Debug, Serialize, Deserialize)]
    struct ClaimResponse {
        user_id: String,
        is_active_sponsor: bool,
        is_admin: bool,
    }
    let json = resp.json::<ClaimResponse>().await.unwrap();

    let user = sqlx::query!(
      "INSERT INTO Users (user_id, coreyja_user_id, is_active_sponsor, is_admin) VALUES ($1, $2, $3, $4) ON CONFLICT (coreyja_user_id) DO UPDATE SET is_active_sponsor = excluded.is_active_sponsor, is_admin = excluded.is_admin RETURNING *",
      uuid::Uuid::new_v4(),
      uuid::Uuid::parse_str(&json.user_id).unwrap(),
      json.is_active_sponsor,
      json.is_admin
  )
  .fetch_one(app_state.db())
  .await
  .unwrap();

    DBSession::create(user.user_id, &app_state, &cookies)
        .await
        .unwrap();

    Redirect::temporary("/").into_response()
}

pub async fn logout(
    cookies: tower_cookies::Cookies,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let private = cookies.private(app_state.cookie_key());
    private.remove(Cookie::new("session_id", ""));

    Redirect::to("/").into_response()
}
