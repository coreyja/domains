use crate::{auth::AdminSession, AppState};
use axum::{extract::State, response::IntoResponse};
use cja::app_state::AppState as _;
use maud::html;

pub(crate) struct PorkbunDomain {
    pub(crate) porkbun_domain_id: String,
    pub(crate) auto_renew: bool,
    pub(crate) purchase_date: chrono::DateTime<chrono::Utc>,
    pub(crate) domain: String,
    pub(crate) expire_date: chrono::DateTime<chrono::Utc>,
    pub(crate) not_local: bool,
    pub(crate) security_lock: bool,
    pub(crate) status: Option<String>,
    pub(crate) tld: String,
    pub(crate) whois_privacy: bool,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    pub(crate) updated_at: chrono::DateTime<chrono::Utc>,
}

pub(crate) async fn show(
    admin_session: AdminSession,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let domains = sqlx::query_as!(
        PorkbunDomain,
        "SELECT * FROM PorkbunDomains ORDER BY purchase_date DESC"
    )
    .fetch_all(app_state.db())
    .await
    .unwrap();

    html! {
        h1 { "Domains" }

        h2 { "Porkbun Domains" }

        table {
            thead {
                tr {
                    th { "Domain" }
                }
            }

            tbody {
                @for domain in domains {
                    tr {
                        td { (domain.domain) }
                    }
                }
            }
        }
    }
}
