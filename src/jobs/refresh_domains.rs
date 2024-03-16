use chrono::NaiveDateTime;
use cja::jobs::Job;
use miette::IntoDiagnostic;

use crate::AppState;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RefreshDomains;

#[async_trait::async_trait]
impl Job<AppState> for RefreshDomains {
    const NAME: &'static str = "RefreshDomains";

    async fn run(&self, app_state: AppState) -> miette::Result<()> {
        let config =
            crate::apis::porkbun::Config::from_env().expect("Failed to get porkbun config");

        let resp = crate::apis::porkbun::fetch_domains(config).await.unwrap();

        let format = "%Y-%m-%d %H:%M:%S";

        for domain in resp.domains {
            sqlx::query!("
          INSERT INTO PorkbunDomains
            (porkbun_domain_id, auto_renew, purchase_date, domain, expire_date, not_local, security_lock, status, tld, whois_privacy)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (domain)
            DO UPDATE SET
              auto_renew = excluded.auto_renew,
              purchase_date = excluded.purchase_date,
              expire_date = excluded.expire_date,
              not_local = excluded.not_local,
              security_lock = excluded.security_lock,
              status = excluded.status,
              tld = excluded.tld,
              whois_privacy = excluded.whois_privacy
              ",
              uuid::Uuid::new_v4(),
            domain.auto_renew == "1",
            NaiveDateTime::parse_from_str(&domain.create_date, &format).into_diagnostic()?.and_utc(),
            domain.domain,
            NaiveDateTime::parse_from_str(&domain.expire_date, &format).into_diagnostic()?.and_utc(),
            domain.not_local == 1,
            domain.security_lock == "1",
            domain.status,
            domain.tld,
            domain.whois_privacy == "1"
            ).execute(&app_state.db).await.into_diagnostic()?;
        }

        Ok(())
    }
}
