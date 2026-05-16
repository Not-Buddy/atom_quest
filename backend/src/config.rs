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
    #[allow(dead_code)]
    pub admin_seed_password: Option<String>,

    // ── 5.1 Azure AD / Entra ID ──────────────────────────────────────────────
    /// Azure App Registration client ID
    pub azure_client_id: Option<String>,
    /// Azure App Registration client secret
    pub azure_client_secret: Option<String>,
    /// Tenant ID (use "common" for multi-tenant)
    pub azure_tenant_id: Option<String>,
    /// Redirect URI registered in the App Registration
    pub azure_redirect_uri: Option<String>,
    /// Azure AD group Object ID whose members are granted the "admin" role
    pub azure_group_admin: Option<String>,
    /// Azure AD group Object ID whose members are granted the "manager" role
    pub azure_group_manager: Option<String>,

    // ── 5.2 Microsoft Teams Notifications ────────────────────────────────────
    /// Default incoming-webhook URL for a Teams channel (used as a fallback
    /// when individual users have not set a personal webhook)
    pub teams_default_webhook: Option<String>,
    /// Frontend base URL used to build deep-links in notifications
    pub frontend_base_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
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
        let admin_seed_password = std::env::var("ADMIN_SEED_PASSWORD").ok();

        // Azure AD (all optional — SSO is disabled when absent)
        let azure_client_id      = std::env::var("AZURE_CLIENT_ID").ok();
        let azure_client_secret  = std::env::var("AZURE_CLIENT_SECRET").ok();
        let azure_tenant_id      = std::env::var("AZURE_TENANT_ID").ok();
        let azure_redirect_uri   = std::env::var("AZURE_REDIRECT_URI").ok();
        let azure_group_admin    = std::env::var("AZURE_GROUP_ADMIN").ok();
        let azure_group_manager  = std::env::var("AZURE_GROUP_MANAGER").ok();

        // Teams / Notifications
        let teams_default_webhook = std::env::var("TEAMS_DEFAULT_WEBHOOK").ok();
        let frontend_base_url     = std::env::var("FRONTEND_BASE_URL").ok();

        Config {
            database_url,
            jwt_secret,
            jwt_expiry,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            base_url,
            admin_seed_password,
            azure_client_id,
            azure_client_secret,
            azure_tenant_id,
            azure_redirect_uri,
            azure_group_admin,
            azure_group_manager,
            teams_default_webhook,
            frontend_base_url,
        }
    }
}
