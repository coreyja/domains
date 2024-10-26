use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{self},
};
use cja::{
    app_state::AppState as _,
    server::session::{DBSession, SessionRedirect},
};
use uuid::Uuid;

use crate::AppState;

#[allow(dead_code)]
pub(crate) struct User {
    pub(crate) user_id: Uuid,
    pub(crate) coreyja_user_id: Uuid,
    pub(crate) is_active_sponsor: bool,
    pub(crate) is_admin: bool,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    pub(crate) updated_at: chrono::DateTime<chrono::Utc>,
}

#[allow(dead_code)]
pub(crate) struct AdminSession {
    pub(crate) user: User,
    pub(crate) session: DBSession,
}

#[async_trait]
impl FromRequestParts<AppState> for AdminSession {
    type Rejection = SessionRedirect;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = DBSession::from_request_parts(parts, state).await?;
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM Users WHERE user_id = $1",
            session.user_id
        )
        .fetch_one(state.db())
        .await
        .map_err(|_| SessionRedirect::temporary("/"))?;

        if !user.is_admin {
            return Err(SessionRedirect::temporary("/"));
        }

        Ok(AdminSession { user, session })
    }
}
