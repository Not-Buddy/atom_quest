use crate::{
    api::ApiError,
    config::Config,
    db::{
        models::{
            AuthResponse, Claims, ForgotPasswordRequest, LoginRequest, ResetPasswordFormBody,
            ResetPasswordRequest, UserResponse,
        },
        users, Database,
    },
    utils::{email, hash_password},
};
use axum::{extract::Query, Extension, Json, extract::State};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{distributions::Alphanumeric, Rng};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use validator::Validate;

pub type ForgotPasswordRateLimiter = Arc<Mutex<HashMap<String, Instant>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
    pub forgot_password_rate_limiter: ForgotPasswordRateLimiter,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let user = users::find_by_email(&state.db.pool, &req.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    if !hash_password::verify(&req.password, &user.password_hash) {
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    let token = generate_jwt(&user, &state.config)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = users::find_by_id(&state.db.pool, claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<&'static str>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    {
        let mut limiter = state.forgot_password_rate_limiter.lock().unwrap();
        if let Some(last) = limiter.get(&req.email) {
            if last.elapsed() < Duration::from_secs(15 * 60) {
                return Err(ApiError::TooManyRequests(
                    "You can only make one password reset request every 15 minutes.".to_string(),
                ));
            }
        }
        limiter.insert(req.email.clone(), Instant::now());
    }

    let user = users::find_by_email(&state.db.pool, &req.email)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::hours(12);

    users::set_password_reset_token(&state.db.pool, user.id, &token, expires_at).await?;

    email::send_password_reset_email(&state.config, &req.email, &token)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to send email: {}", e)))?;

    Ok(Json("Password reset email sent"))
}

pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<&'static str>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let (user, token_record) =
        users::find_by_reset_token(&state.db.pool, &req.token)
            .await?
            .ok_or_else(|| ApiError::Unauthorized("Invalid or expired token".to_string()))?;

    let new_hash = hash_password::hash(&req.new_password);

    users::mark_token_used_and_update_password(&state.db.pool, user.id, token_record.id, &new_hash).await?;

    Ok(Json("Password has been reset successfully"))
}

pub async fn reset_password_form_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    Json(req_body): Json<ResetPasswordFormBody>,
) -> Result<Json<&'static str>, ApiError> {
    let token = params
        .get("token")
        .ok_or_else(|| ApiError::ValidationError("Token is missing".to_string()))?;

    req_body
        .validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let (user, token_record) =
        users::find_by_reset_token(&state.db.pool, token)
            .await?
            .ok_or_else(|| ApiError::Unauthorized("Invalid or expired token".to_string()))?;

    let new_hash = hash_password::hash(&req_body.new_password);

    users::mark_token_used_and_update_password(&state.db.pool, user.id, token_record.id, &new_hash).await?;

    Ok(Json("Password has been reset successfully"))
}

fn generate_jwt(user: &crate::db::models::User, config: &Config) -> Result<String, ApiError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(config.jwt_expiry))
        .expect("Valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.email.clone(),
        user_id: user.id,
        email: user.email.clone(),
        role: user.role.clone(),
        department_id: user.department_id,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| e.into())
}
