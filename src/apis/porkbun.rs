use serde::{Deserialize, Serialize};
use tracing::debug;

pub struct Config {
    api_key: String,
    secret_api_key: String,
}

impl Config {
    pub fn from_env() -> color_eyre::Result<Self> {
        Ok(Self {
            api_key: std::env::var("PORKBUN_API_KEY")?,
            secret_api_key: std::env::var("PORKBUN_SECRET_API_KEY")?,
        })
    }
}

#[derive(Debug, Serialize)]
struct Auth {
    apikey: String,
    secretapikey: String,
}

impl Auth {
    pub fn from_config(config: &Config) -> Self {
        Self {
            apikey: config.api_key.clone(),
            secretapikey: config.secret_api_key.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FetchDomainsResponse {
    pub domains: Vec<PorkbunDomain>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PorkbunDomain {
    pub auto_renew: String,
    pub create_date: String,
    pub domain: String,
    pub expire_date: String,
    pub not_local: i32,
    pub security_lock: String,
    pub status: Option<String>,
    pub tld: String,
    pub whois_privacy: String,
}

pub async fn fetch_domains(config: Config) -> color_eyre::Result<FetchDomainsResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.porkbun.com/api/json/v3/domain/listAll")
        .json(&Auth::from_config(&config))
        .send()
        .await?;

    Ok(response.json().await?)
}

#[derive(Serialize, Deserialize)]
pub struct Nameserver(pub String);

#[derive(Serialize, Deserialize)]
pub struct FetchDomainNameserversResponse {
    pub status: String,
    pub ns: Vec<Nameserver>,
}

pub async fn fetch_domain_nameservers(
    config: Config,
    domain: String,
) -> color_eyre::Result<FetchDomainNameserversResponse> {
    let client = reqwest::Client::new();
    let url = format!("https://api.porkbun.com/api/json/v3/domain/getNs/{domain}");
    let response = client
        .post(url)
        .json(&Auth::from_config(&config))
        .send()
        .await?;

    let text = response.text().await?;

    debug!("response: {:?}", text);

    Ok(serde_json::from_str(&text)?)
}
