use crate::{
    api::ApiError,
    config::Config,
    db::{
        models::{
            AuthResponse, Claims, ForgotPasswordRequest, LoginRequest, ProfileLinksResponse,
            ResetPasswordFormBody, ResetPasswordRequest, StudentResponse, UpdateLinksRequest,
        },
        student, Database,
    },
    codechef::worker::SyncQueue as CodeChefSyncQueue,
    codeforces::worker::SyncQueue as CodeforcesSyncQueue,
    leetcode::worker::SyncQueue as LeetCodeSyncQueue,
    utils::{email, hash_password, url_parser, hash_password::verify},
};
use axum::{extract::{Query, State}, Extension, Json, debug_handler};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{distributions::Alphanumeric, Rng};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use validator::Validate;
use std::time::{Duration, Instant};

pub type ForgotPasswordRateLimiter = Arc<Mutex<HashMap<String, Instant>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
    pub leetcode_sync_queue: LeetCodeSyncQueue,
    pub codechef_sync_queue: CodeChefSyncQueue,
    pub codeforces_sync_queue: CodeforcesSyncQueue,
    pub forgot_password_rate_limiter: ForgotPasswordRateLimiter,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // Find student by email
    let student = student::find_student_by_email(&state.db.pool, &req.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    // Verify password
    if !verify(&req.password, &student.password) {
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate JWT
    let token = generate_jwt(
        &req.email,
        student.id,
        &student.registration_number,
        student.academic_year.as_deref(),
        &state.config,
    )?;

    Ok(Json(AuthResponse {
        token,
        student: student.into(),
    }))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<StudentResponse>, ApiError> {
    let student = student::find_student_by_id(&state.db.pool, claims.student_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Student not found".to_string()))?;
    Ok(Json(student.into()))
}

pub async fn get_profile_links(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ProfileLinksResponse>, ApiError> {
    let student = student::find_student_by_id(&state.db.pool, claims.student_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Student not found".to_string()))?;

    // Reconstruct full URLs from usernames, or return null if not set
    let github_link = student.github_username
        .map(|username| format!("https://github.com/{}", username));
    
    let leetcode_link = student.leetcode_username
        .map(|username| format!("https://leetcode.com/u/{}", username));
    
    let codechef_link = student.codechef_username
        .map(|username| format!("https://www.codechef.com/users/{}", username));
    
    let codeforces_link = student.codeforces_username
        .map(|username| format!("https://codeforces.com/profile/{}", username));
    
    let linkedin_link = student.linkedin_url;

    Ok(Json(ProfileLinksResponse {
        github_link,
        leetcode_link,
        codechef_link,
        codeforces_link,
        linkedin_link,
    }))
}

pub async fn update_profile_links(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateLinksRequest>,
) -> Result<Json<StudentResponse>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // Parse usernames from URLs
    let github_username = req.github_link
        .as_ref()
        .and_then(|url| url_parser::parse_github_url(url))
        .or_else(|| {
            if let Some(ref url) = req.github_link {
                tracing::warn!("Failed to parse GitHub URL: {}", url);
            }
            None
        });

    let leetcode_username = req.leetcode_link
        .as_ref()
        .and_then(|url| url_parser::parse_leetcode_url(url))
        .or_else(|| {
            if let Some(ref url) = req.leetcode_link {
                tracing::warn!("Failed to parse LeetCode URL: {}", url);
            }
            None
        });

    let codechef_username = req.codechef_link
        .as_ref()
        .and_then(|url| url_parser::parse_codechef_url(url))
        .or_else(|| {
            if let Some(ref url) = req.codechef_link {
                tracing::warn!("Failed to parse CodeChef URL: {}", url);
            }
            None
        });

    let codeforces_username = req.codeforces_link
        .as_ref()
        .and_then(|url| url_parser::parse_codeforces_url(url))
        .or_else(|| {
            if let Some(ref url) = req.codeforces_link {
                tracing::warn!("Failed to parse Codeforces URL: {}", url);
            }
            None
        });

    let linkedin_link = req.linkedin_link
        .as_ref()
        .and_then(|url| url_parser::validate_linkedin_url(url))
        .or_else(|| {
            if let Some(ref url) = req.linkedin_link {
                tracing::warn!("Invalid LinkedIn URL: {}", url);
            }
            None
        });

    // Validate URL formats
    if req.github_link.is_some() && github_username.is_none() {
        return Err(ApiError::ValidationError("Invalid GitHub URL format".to_string()));
    }

    if req.leetcode_link.is_some() && leetcode_username.is_none() {
        return Err(ApiError::ValidationError("Invalid LeetCode URL format".to_string()));
    }

    if req.codechef_link.is_some() && codechef_username.is_none() {
        return Err(ApiError::ValidationError("Invalid CodeChef URL format".to_string()));
    }

    if req.codeforces_link.is_some() && codeforces_username.is_none() {
        return Err(ApiError::ValidationError("Invalid Codeforces URL format".to_string()));
    }

    if req.linkedin_link.is_some() && linkedin_link.is_none() {
        return Err(ApiError::ValidationError("Invalid LinkedIn URL format".to_string()));
    }

    // Check for duplicate LeetCode username
    if let Some(existing_student) = student::find_student_by_leetcode_username(
        &state.db.pool,
        leetcode_username.as_deref().unwrap_or_default(),
    )
    .await
    .map_err(|_| {
        ApiError::DatabaseError(sqlx::Error::Io(std::io::Error::other(
            "Database error",
        )))
    })?
        && leetcode_username.is_some() 
        && existing_student.id != claims.student_id 
    {
        return Err(ApiError::ValidationError(
            "LeetCode username is already taken by another user".to_string(),
        ));
    }

    // Check for duplicate CodeChef username
    if let Some(existing_student) = student::find_student_by_codechef_username(
        &state.db.pool,
        codechef_username.as_deref().unwrap_or_default(),
    )
    .await
    .map_err(|_| {
        ApiError::DatabaseError(sqlx::Error::Io(std::io::Error::other(
            "Database error",
        )))
    })?
        && codechef_username.is_some() 
        && existing_student.id != claims.student_id 
    {
        return Err(ApiError::ValidationError(
            "CodeChef username is already taken by another user".to_string(),
        ));
    }

    // Check for duplicate Codeforces username
    if let Some(existing_student) = student::find_student_by_codeforces_username(
        &state.db.pool,
        codeforces_username.as_deref().unwrap_or_default(),
    )
    .await
    .map_err(|_| {
        ApiError::DatabaseError(sqlx::Error::Io(std::io::Error::other(
            "Database error",
        )))
    })?
        && codeforces_username.is_some() 
        && existing_student.id != claims.student_id 
    {
        return Err(ApiError::ValidationError(
            "Codeforces username is already taken by another user".to_string(),
        ));
    }

    let updated_student = student::update_student_links(
        &state.db.pool,
        claims.student_id,
        github_username.as_deref(),
        linkedin_link.as_deref(),
        leetcode_username.as_deref(),
        codechef_username.as_deref(),
        codeforces_username.as_deref(),
    )
    .await?;

    // Trigger sync for each platform if username was updated
    if leetcode_username.is_some() {
        state.leetcode_sync_queue.add_priority_job(updated_student.registration_number.clone()).await;
    }

    if codechef_username.is_some() {
        state.codechef_sync_queue.add_priority_job(updated_student.registration_number.clone()).await;
    }

    if codeforces_username.is_some() {
        state.codeforces_sync_queue.add_priority_job(updated_student.registration_number.clone()).await;
    }

    Ok(Json(updated_student.into()))
}

fn generate_jwt(
    email: &str,
    student_id: i32,
    registration_number: &str,
    academic_year: Option<&str>,
    config: &Config,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(config.jwt_expiry))
        .expect("Valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: email.to_string(),
        student_id,
        registration_number: Some(registration_number.to_string()),
        specialization: None, // For students, specialization is None
        academic_year: academic_year.map(|s| s.to_string()),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;

    Ok(token)
}

#[debug_handler]
pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<&'static str>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;
    
    // Rate limiting logic
    {
        let mut limiter = state.forgot_password_rate_limiter.lock().unwrap();
        if let Some(last_request_time) = limiter.get(&req.email)
            && last_request_time.elapsed() < Duration::from_secs(15 * 60) {
                return Err(ApiError::TooManyRequests(
                    "You can only make one password reset request every 15 minutes.".to_string(),
                ));
            }
        limiter.insert(req.email.clone(), Instant::now());
    }

    let student = student::find_student_by_email(&state.db.pool, &req.email)
        .await?
        .ok_or_else(|| ApiError::NotFound("Student not found".to_string()))?;

    // Generate a random token
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

        // Set token expiry time (e.g., 12 hours from now)
    let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::hours(12);

    // Save the token and expiry to the database
    student::set_password_reset_token(&state.db.pool, student.id, &token, expires_at).await?;

    // Send the password reset email
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

    let (student, token_record) = student::find_student_by_reset_token(&state.db.pool, &req.token)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid or expired token".to_string()))?;

    // Hash the new password
    let new_password_hash = hash_password::hash(&req.new_password);

    // Update the password in the database
    student::update_password(&state.db.pool, student.id, token_record.id, &new_password_hash).await?;

    Ok(Json("Password has been reset successfully"))
}

pub async fn reset_password_form_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    Json(req_body): Json<ResetPasswordFormBody>,
) -> Result<Json<&'static str>, ApiError> {
    // Extract token from query parameters
    let token = params
        .get("token")
        .ok_or_else(|| ApiError::ValidationError("Token is missing".to_string()))?;

    // Validate the new password from the request body
    req_body
        .validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let (student, token_record) = student::find_student_by_reset_token(&state.db.pool, token)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid or expired token".to_string()))?;

    // Hash the new password
    let new_password_hash = hash_password::hash(&req_body.new_password);

    // Update the password in the database
    student::update_password(&state.db.pool, student.id, token_record.id, &new_password_hash).await?;

    Ok(Json("Password has been reset successfully"))
}
