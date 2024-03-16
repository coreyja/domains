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
