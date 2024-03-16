use color_eyre::config;
use serde::Serialize;
use serde_json::Value;

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

pub async fn fetch_domains(config: Config) -> color_eyre::Result<Value> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://porkbun.com/api/json/v3/domain/listAll")
        .json(&Auth::from_config(&config))
        .send()
        .await?;

    Ok(response.json().await?)
}
