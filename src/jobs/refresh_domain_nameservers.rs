use cja::{app_state::AppState as _, jobs::Job};
use uuid::Uuid;

use crate::{apis::porkbun::fetch_domain_nameservers, routes::domains::PorkbunDomain, AppState};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RefreshDomainsNameservers;

#[async_trait::async_trait]
impl Job<AppState> for RefreshDomainsNameservers {
    const NAME: &'static str = "RefreshDomainsNameservers";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        let domains = sqlx::query_as!(
            PorkbunDomain,
            "SELECT * FROM PorkbunDomains ORDER BY purchase_date DESC"
        )
        .fetch_all(app_state.db())
        .await?;

        for domain in domains {
            RefreshDomainNameservers {
                porkbun_domain_id: domain.porkbun_domain_id,
            }
            .enqueue(
                app_state.clone(),
                "RefreshDomainsNameservers bulk".to_string(),
            )
            .await?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RefreshDomainNameservers {
    pub porkbun_domain_id: Uuid,
}
#[async_trait::async_trait]
impl Job<AppState> for RefreshDomainNameservers {
    const NAME: &'static str = "RefreshDomainNameservers";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        let config =
            crate::apis::porkbun::Config::from_env().expect("Failed to get porkbun config");

        let db_domain = sqlx::query_as!(
            PorkbunDomain,
            "SELECT * FROM PorkbunDomains WHERE porkbun_domain_id = $1",
            self.porkbun_domain_id
        )
        .fetch_one(app_state.db())
        .await?;

        let resp = fetch_domain_nameservers(config, db_domain.domain).await?;
        let nameservers: Vec<_> = resp.ns.into_iter().map(|ns| ns.0).collect();

        sqlx::query!(
            "UPDATE PorkbunDomains SET nameservers = $1 WHERE porkbun_domain_id = $2",
            &nameservers,
            self.porkbun_domain_id
        )
        .execute(app_state.db())
        .await?;

        Ok(())
    }
}
