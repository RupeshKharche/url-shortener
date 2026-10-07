use crate::rate_limit::RateLimitConfig;

#[derive(Clone)]
pub struct AppConfig {
    pub db_url: String,
    pub base_url: String,
    pub code_length: usize,
    pub default_code_ttl_seconds: u64,
    pub rate_limit_config: RateLimitConfig
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let db_url = std::env::var("DATABASE_URL")?;
        let base_url = std::env::var("BASE_URL")?;
        let code_length = std::env::var("CODE_LENGTH")?.trim().parse()?;
        let default_code_ttl_seconds = std::env::var("DEFAULT_CODE_TTL_SECONDS")?.trim().parse()?;
        let rate_limit_config = RateLimitConfig::from_env()?;

        Ok(Self {
            db_url,
            base_url,
            code_length,
            default_code_ttl_seconds,
            rate_limit_config
        })
    }
}
