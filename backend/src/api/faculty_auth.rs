use crate::{
    api::ApiError,
    config::Config,
    db::{
        models::{Claims, FacultyLoginRequest, FacultyAuthResponse, FacultyResponse},
        faculty,
    },
    utils::hash_password,
};
use crate::api::auth::AppState;
use axum::{extract::State, Extension, Json};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::sync::Arc;

// Remove the unused import: use validator::Validate;

pub async fn faculty_login(
    State(state): State<Arc<AppState>>,  // Use AppState instead of FacultyAppState
    Json(req): Json<FacultyLoginRequest>,
) -> Result<Json<FacultyAuthResponse>, ApiError> {
    // Find faculty by credentials
    let faculty_result = faculty::find_faculty_by_credentials(
        &state.db.pool,
        &req.specialization,
        &req.username,
    )
    .await?;

    if let Some((faculty, password_hash)) = faculty_result {
        // Verify password
        if hash_password::verify(&req.password, &password_hash) {
            // Generate JWT
            let token = generate_faculty_jwt(
                &req.username,
                faculty.id,
                &req.specialization,
                faculty.academic_year.as_deref(),
                &state.config,
            )?;

            return Ok(Json(FacultyAuthResponse {
                token,
                faculty: faculty.into(),
            }));
        }
    }

    Err(ApiError::Unauthorized("Invalid credentials".to_string()))
}

pub async fn faculty_me(
    State(state): State<Arc<AppState>>, 
    Extension(claims): Extension<Claims>,
) -> Result<Json<FacultyResponse>, ApiError> {
    // Verify this is a faculty token
    if claims.specialization.is_none() {
        return Err(ApiError::Unauthorized("Not a faculty token".to_string()));
    }

    let faculty = faculty::find_faculty_by_credentials(
        &state.db.pool,
        claims.specialization.as_ref().unwrap(),
        &claims.sub,
    )
    .await?
    .map(|(f, _)| f)
    .ok_or_else(|| ApiError::NotFound("Faculty not found".to_string()))?;

    Ok(Json(faculty.into()))
}

fn generate_faculty_jwt(
    username: &str,
    faculty_id: i32,
    specialization: &str,
    academic_year: Option<&str>,
    config: &Config,
) -> Result<String, ApiError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(config.jwt_expiry))
        .expect("Valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_string(),
        student_id: faculty_id,
        registration_number: None,
        specialization: Some(specialization.to_string()),
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
