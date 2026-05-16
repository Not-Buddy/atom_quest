/// 5.1 — Microsoft Entra ID (Azure AD) SSO
///
/// Flow:
///   1. Frontend hits GET /auth/azure/login  → backend returns the Azure AD auth URL
///   2. Azure redirects back to GET /auth/azure/callback?code=…&state=…
///   3. Backend exchanges the code for tokens, fetches user profile, upserts the
///      user row, and returns a signed JWT exactly like the local login endpoint.
///
/// Config env vars required:
///   AZURE_CLIENT_ID      – App Registration client ID
///   AZURE_CLIENT_SECRET  – Client secret value
///   AZURE_TENANT_ID      – Tenant ID (or "common" for multi-tenant)
///   AZURE_REDIRECT_URI   – Must match the redirect URI in the App Registration
use crate::{
    api::ApiError,
    config::Config,
    db::{models::Claims, users},
};
use axum::{
    extract::{Query, State},
    response::Json,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::auth::AppState;

// ─── Azure Token Response ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct AzureTokenResponse {
    access_token: String,
    id_token:     Option<String>,
    #[allow(dead_code)]
    token_type:   String,
    #[allow(dead_code)]
    expires_in:   Option<u64>,
}

// ─── Microsoft Graph /me response ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphUser {
    id:                   String,        // Azure Object ID
    display_name:         String,
    mail:                 Option<String>,
    user_principal_name:  String,
    job_title:            Option<String>,
    manager_id:           Option<String>, // populated by a separate Graph call
}

// ─── ID-token claims we care about ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    oid:  Option<String>, // Azure Object ID
    upn:  Option<String>,
    name: Option<String>,
    #[serde(rename = "preferred_username")]
    preferred_username: Option<String>,
    /// Azure AD groups the user is a member of (if configured in the token)
    groups: Option<Vec<String>>,
}

// ─── Request / Response DTOs ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AzureCallbackQuery {
    pub code:  String,
    pub state: Option<String>,
}

#[derive(Serialize)]
pub struct AzureLoginUrlResponse {
    pub login_url: String,
}

#[derive(Serialize)]
pub struct AuthTokenResponse {
    pub token: String,
    pub user: crate::db::models::UserResponse,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Build the Azure AD authorization URL that the client should redirect to.
fn build_auth_url(config: &Config) -> String {
    let tenant = config.azure_tenant_id.as_deref().unwrap_or("common");
    let client_id = config.azure_client_id.as_deref().unwrap_or_default();
    let redirect_uri = config.azure_redirect_uri.as_deref().unwrap_or_default();

    format!(
        "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize\
        ?client_id={client_id}\
        &response_type=code\
        &redirect_uri={redirect_uri}\
        &response_mode=query\
        &scope=openid%20profile%20email%20User.Read\
        &state=atomquest_sso"
    )
}

/// Exchange the code for tokens at the Azure AD token endpoint.
async fn exchange_code_for_tokens(
    config: &Config,
    code: &str,
) -> Result<AzureTokenResponse, String> {
    let tenant = config.azure_tenant_id.as_deref().unwrap_or("common");
    let token_url = format!(
        "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token"
    );

    let params = [
        ("client_id",     config.azure_client_id.as_deref().unwrap_or_default()),
        ("client_secret", config.azure_client_secret.as_deref().unwrap_or_default()),
        ("redirect_uri",  config.azure_redirect_uri.as_deref().unwrap_or_default()),
        ("grant_type",    "authorization_code"),
        ("code",          code),
        ("scope",         "openid profile email User.Read"),
    ];

    let client = Client::new();
    let resp = client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange network error: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed: {body}"));
    }

    resp.json::<AzureTokenResponse>()
        .await
        .map_err(|e| format!("Token parse error: {e}"))
}

/// Fetch the signed-in user's profile from Microsoft Graph.
async fn fetch_graph_user(access_token: &str) -> Result<GraphUser, String> {
    let client = Client::new();
    let resp = client
        .get("https://graph.microsoft.com/v1.0/me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Graph /me network error: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Graph /me failed: {body}"));
    }

    resp.json::<GraphUser>()
        .await
        .map_err(|e| format!("Graph /me parse error: {e}"))
}

/// Decode the ID token's payload (no signature verification needed here —
/// Azure already validated our code exchange). For production you should
/// validate the signature against Azure's JWKS endpoint.
fn decode_id_token_claims(id_token: &str) -> Option<IdTokenClaims> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let payload = parts[1];
    // URL-safe base64 without padding
    let padded = match payload.len() % 4 {
        2 => format!("{payload}=="),
        3 => format!("{payload}="),
        _ => payload.to_string(),
    };
    let bytes = base64_decode_url_safe(&padded)?;
    serde_json::from_slice::<IdTokenClaims>(&bytes).ok()
}

fn base64_decode_url_safe(s: &str) -> Option<Vec<u8>> {
    // Manual URL-safe base64 decode without an extra crate dependency.
    // Production code should use the `base64` crate.
    let standard = s.replace('-', "+").replace('_', "/");
    let alphabet =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [255u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        lookup[c as usize] = i as u8;
    }
    let chars: Vec<u8> = standard.bytes().collect();
    let mut out = Vec::with_capacity(chars.len() * 3 / 4);
    let mut i = 0;
    while i + 3 < chars.len() {
        let a = lookup[chars[i] as usize];
        let b = lookup[chars[i + 1] as usize];
        let c = lookup[chars[i + 2] as usize];
        let d = lookup[chars[i + 3] as usize];
        if a == 255 || b == 255 { break; }
        out.push((a << 2) | (b >> 4));
        if c != 255 { out.push((b << 4) | (c >> 2)); }
        if d != 255 { out.push((c << 6) | d); }
        i += 4;
    }
    Some(out)
}

/// Derive an AtomQuest role from Azure AD group membership.
/// Group Object IDs should be configured in env vars:
///   AZURE_GROUP_ADMIN   – group OID → admin
///   AZURE_GROUP_MANAGER – group OID → manager
/// Everyone else defaults to "employee".
fn role_from_groups(groups: &[String], config: &Config) -> &'static str {
    let admin_group   = config.azure_group_admin.as_deref().unwrap_or("__none__");
    let manager_group = config.azure_group_manager.as_deref().unwrap_or("__none__");

    if groups.iter().any(|g| g == admin_group) {
        "admin"
    } else if groups.iter().any(|g| g == manager_group) {
        "manager"
    } else {
        "employee"
    }
}

/// Generate the same JWT format used by local login.
fn generate_jwt_for_user(
    user: &crate::db::models::User,
    config: &Config,
) -> Result<String, ApiError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(config.jwt_expiry))
        .expect("Valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub:           user.email.clone(),
        user_id:       user.id,
        email:         user.email.clone(),
        role:          user.role.clone(),
        department_id: user.department_id,
        exp:           expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::InternalServerError(format!("JWT error: {e}")))
}

// ─── Route Handlers ───────────────────────────────────────────────────────────

/// GET /auth/azure/login
/// Returns the Azure AD authorization URL for the frontend to redirect the
/// browser to.
pub async fn azure_login_url(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AzureLoginUrlResponse>, ApiError> {
    if state.config.azure_client_id.is_none() {
        return Err(ApiError::ValidationError(
            "Azure AD SSO is not configured on this server.".to_string(),
        ));
    }
    let url = build_auth_url(&state.config);
    Ok(Json(AzureLoginUrlResponse { login_url: url }))
}

/// GET /auth/azure/callback?code=…&state=…
/// Azure AD redirects here after the user authenticates.
/// We exchange the code, upsert the local user, and return a JWT.
pub async fn azure_callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AzureCallbackQuery>,
) -> Result<Json<AuthTokenResponse>, ApiError> {
    // 1. Exchange code → tokens
    let tokens = exchange_code_for_tokens(&state.config, &params.code)
        .await
        .map_err(|e| ApiError::Unauthorized(e))?;

    // 2. Fetch user profile from Microsoft Graph
    let graph_user = fetch_graph_user(&tokens.access_token)
        .await
        .map_err(|e| ApiError::InternalServerError(e))?;

    // 3. Decode ID token claims (groups, name, etc.)
    let id_claims = tokens
        .id_token
        .as_deref()
        .and_then(decode_id_token_claims);

    let email = graph_user
        .mail
        .clone()
        .unwrap_or_else(|| graph_user.user_principal_name.clone());

    let full_name = graph_user.display_name.clone();
    let azure_oid = graph_user.id.clone();
    let azure_upn = graph_user.user_principal_name.clone();

    let groups = id_claims
        .as_ref()
        .and_then(|c| c.groups.clone())
        .unwrap_or_default();
    let role = role_from_groups(&groups, &state.config);

    // 4. Upsert user: look up by Azure OID, then by email, then create
    let user = users::upsert_azure_user(
        &state.db.pool,
        &email,
        &full_name,
        &azure_oid,
        &azure_upn,
        role,
    )
    .await
    .map_err(|e| ApiError::DatabaseError(e))?;

    // 5. Issue JWT
    let token = generate_jwt_for_user(&user, &state.config)?;

    Ok(Json(AuthTokenResponse {
        token,
        user: user.into(),
    }))
}

/// PUT /auth/azure/sync-org
/// Admin-only: Trigger a full org-hierarchy sync from Azure AD (manager
/// reporting lines). Reads all users with azure_oid set, fetches their
/// Graph /me/manager and updates manager_id accordingly.
pub async fn sync_org_hierarchy(
    State(state): State<Arc<AppState>>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if claims.role != "admin" {
        return Err(ApiError::Unauthorized("Admin only".to_string()));
    }

    // Fetch all users who were created via Azure AD
    let azure_users = users::list_azure_users(&state.db.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e))?;

    if azure_users.is_empty() {
        return Ok(Json(serde_json::json!({
            "synced": 0,
            "message": "No Azure AD users found"
        })));
    }

    // We need a Graph access token obtained via client_credentials for the
    // org-sync flow (no user context).  Build it from client_id + client_secret.
    let tenant = state
        .config
        .azure_tenant_id
        .as_deref()
        .unwrap_or("common");
    let cc_token = fetch_client_credentials_token(&state.config, tenant)
        .await
        .map_err(|e| ApiError::InternalServerError(e))?;

    let client = Client::new();
    let mut synced = 0u32;

    for user in &azure_users {
        let Some(ref oid) = user.azure_oid else { continue };

        // Fetch manager from Graph
        let mgr_url = format!(
            "https://graph.microsoft.com/v1.0/users/{oid}/manager"
        );
        let resp = client
            .get(&mgr_url)
            .bearer_auth(&cc_token)
            .send()
            .await;

        let Ok(resp) = resp else { continue };
        if !resp.status().is_success() { continue; }

        #[derive(Deserialize)]
        struct GraphMgrResp { id: String }

        let Ok(mgr) = resp.json::<GraphMgrResp>().await else { continue };

        // Find the local user corresponding to this Azure OID
        let mgr_local = users::find_by_azure_oid(&state.db.pool, &mgr.id).await;
        if let Ok(Some(mgr_user)) = mgr_local {
            let _ = sqlx::query(
                "UPDATE users SET manager_id = ? WHERE id = ?",
            )
            .bind(mgr_user.id)
            .bind(user.id)
            .execute(&state.db.pool)
            .await;
            synced += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "synced": synced,
        "total_azure_users": azure_users.len()
    })))
}

/// Obtain a client-credentials access token for Graph API (app-only context).
async fn fetch_client_credentials_token(
    config: &Config,
    tenant: &str,
) -> Result<String, String> {
    let token_url = format!(
        "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token"
    );
    let params = [
        ("client_id",     config.azure_client_id.as_deref().unwrap_or_default()),
        ("client_secret", config.azure_client_secret.as_deref().unwrap_or_default()),
        ("grant_type",    "client_credentials"),
        ("scope",         "https://graph.microsoft.com/.default"),
    ];
    let client = Client::new();
    let resp = client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("CC token network error: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("CC token failed: {body}"));
    }

    #[derive(Deserialize)]
    struct CcTokenResp { access_token: String }

    let t = resp
        .json::<CcTokenResp>()
        .await
        .map_err(|e| format!("CC token parse: {e}"))?;

    Ok(t.access_token)
}
