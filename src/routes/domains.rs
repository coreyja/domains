use crate::{auth::AdminSession, AppState};
use axum::{extract::State, response::IntoResponse};
use cja::app_state::AppState as _;
use maud::html;
use uuid::Uuid;

pub(crate) struct PorkbunDomain {
    pub(crate) porkbun_domain_id: Uuid,
    pub(crate) auto_renew: bool,
    pub(crate) purchase_date: chrono::DateTime<chrono::Utc>,
    pub(crate) domain: String,
    pub(crate) expire_date: chrono::DateTime<chrono::Utc>,
    pub(crate) not_local: bool,
    pub(crate) security_lock: bool,
    pub(crate) status: Option<String>,
    pub(crate) tld: String,
    pub(crate) whois_privacy: bool,
    pub(crate) nameservers: Vec<String>,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    pub(crate) updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum DnsProvider {
    Porkbun,
    Cloudflare,
    GoogleDomains,
    Vercel,
    Route53,
    Unknown,
}

impl std::fmt::Display for DnsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsProvider::Porkbun => write!(f, "Porkbun"),
            DnsProvider::Cloudflare => write!(f, "Cloudflare"),
            DnsProvider::GoogleDomains => write!(f, "Google Domains"),
            DnsProvider::Vercel => write!(f, "Vercel"),
            DnsProvider::Route53 => write!(f, "AWS Route 53"),
            DnsProvider::Unknown => write!(f, "Unknown"),
        }
    }
}

impl PorkbunDomain {
    pub fn using_porkbun_dns(&self) -> bool {
        self.nameservers
            .iter()
            .all(|ns| ns.ends_with(".porkbun.com"))
    }

    pub fn using_cloudflare_dns(&self) -> bool {
        self.nameservers
            .iter()
            .all(|ns| ns.ends_with(".cloudflare.com"))
    }

    pub fn using_google_domains_dns(&self) -> bool {
        self.nameservers
            .iter()
            .all(|ns| ns.ends_with(".googledomains.com"))
    }

    pub fn using_vercel_dns(&self) -> bool {
        self.nameservers
            .iter()
            .all(|ns| ns.ends_with(".vercel-dns.com"))
    }

    pub fn using_route53_dns(&self) -> bool {
        self.nameservers.iter().all(|ns| ns.contains("awsdns"))
    }

    pub fn dns_provider(&self) -> DnsProvider {
        if self.using_porkbun_dns() {
            return DnsProvider::Porkbun;
        }

        if self.using_cloudflare_dns() {
            return DnsProvider::Cloudflare;
        }

        if self.using_google_domains_dns() {
            return DnsProvider::GoogleDomains;
        }

        if self.using_vercel_dns() {
            return DnsProvider::Vercel;
        }

        if self.using_route53_dns() {
            return DnsProvider::Route53;
        }

        DnsProvider::Unknown
    }
}

pub(crate) async fn show(_: AdminSession, State(app_state): State<AppState>) -> impl IntoResponse {
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
                    th { "DNS Provider" }
                }
            }

            tbody {
                @for domain in domains {
                    @let dns_provider = domain.dns_provider();
                    tr {
                        td { (domain.domain) }
                        td {
                            (dns_provider)
                            br;
                            @if dns_provider == DnsProvider::Unknown {
                                (domain.nameservers.join(", "))
                            }
                         }
                    }
                }
            }
        }
    }
}
