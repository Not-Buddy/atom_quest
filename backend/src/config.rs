use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry: i64,
    pub smtp_host: String,
    #[allow(dead_code)]
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub base_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env file");
        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set in .env file");
        let jwt_expiry = std::env::var("JWT_EXPIRY")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .expect("JWT_EXPIRY must be a valid number");
        let smtp_host = std::env::var("SMTP_HOST")
            .expect("SMTP_HOST must be set in .env file");
        let smtp_port = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .expect("SMTP_PORT must be a valid number");
        let smtp_username = std::env::var("SMTP_USERNAME")
            .expect("SMTP_USERNAME must be set in .env file");
        let smtp_password = std::env::var("SMTP_PASSWORD")
            .expect("SMTP_PASSWORD must be set in .env file");
        let from_email = std::env::var("FROM_EMAIL")
            .expect("FROM_EMAIL must be set in .env file");
        let base_url = std::env::var("BASE_URL")
            .expect("BASE_URL must be set in .env file");

        Ok(Config {
            database_url,
            jwt_secret,
            jwt_expiry,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            base_url,
        })
    }
}
