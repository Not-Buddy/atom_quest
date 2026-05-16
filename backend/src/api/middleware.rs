use crate::{api::ApiError, config::Config, db::models::Claims};
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::{future::Future, pin::Pin, sync::Arc};

pub async fn auth_middleware(
    State(config): State<Arc<Config>>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Invalid authorization format".to_string()))?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| ApiError::Unauthorized("Invalid token".to_string()))?;

    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}

/// Factory that produces a role-guard middleware function.
/// Usage: `middleware::from_fn_with_state(config_arc, require_role("manager"))`
pub fn require_role(
    role: &'static str,
) -> impl Fn(State<Arc<Config>>, Request, Next) -> Pin<Box<dyn Future<Output = Result<Response, ApiError>> + Send>>
       + Clone
       + Send
       + 'static
{
    move |State(config): State<Arc<Config>>, mut req: Request, next: Next| -> Pin<Box<dyn Future<Output = Result<Response, ApiError>> + Send>> {
        let role = role;
        Box::pin(async move {
            let auth_header = req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or_else(|| ApiError::Unauthorized("Invalid authorization format".to_string()))?;

            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
                &Validation::default(),
            )
            .map_err(|_| ApiError::Unauthorized("Invalid token".to_string()))?;

            let claims = token_data.claims;

            if claims.role != role && claims.role != "admin" {
                return Err(ApiError::Unauthorized(format!(
                    "This endpoint requires '{}' role",
                    role
                )));
            }

            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        })
    }
}
