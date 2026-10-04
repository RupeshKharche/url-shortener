#[derive(Clone)]
pub struct AppConfig {
    pub db_url: String,
    pub base_url: String,
    pub code_length: usize,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let db_url = std::env::var("DATABASE_URL")?;
        let base_url = std::env::var("BASE_URL")?;
        let code_length = std::env::var("CODE_LENGTH")?.trim().parse()?;

        Ok(Self {
            db_url,
            base_url,
            code_length,
        })
    }
}
