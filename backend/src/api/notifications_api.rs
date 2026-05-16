/// 5.2 — Notification Preferences API
///
/// Lets any authenticated user read and update their notification preferences
/// (email on/off, Teams webhook URL).
/// Admins can also browse the notification log.
use crate::{
    api::ApiError,
    db::models::Claims,
};
use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::auth::AppState;

// ─── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct NotificationPrefsResponse {
    pub user_id:            i32,
    pub email_enabled:      bool,
    pub teams_enabled:      bool,
    pub teams_webhook_url:  Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePrefsRequest {
    pub email_enabled:     Option<bool>,
    pub teams_enabled:     Option<bool>,
    pub teams_webhook_url: Option<String>,
}

#[derive(Deserialize)]
pub struct NotifLogQuery {
    pub limit:  Option<i64>,
    pub offset: Option<i64>,
    pub event_type: Option<String>,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// GET /notifications/preferences
pub async fn get_preferences(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<NotificationPrefsResponse>, ApiError> {
    let row = sqlx::query(
        r#"SELECT user_id, email_enabled, teams_enabled, teams_webhook_url
           FROM notification_preferences
           WHERE user_id = ?"#,
    )
    .bind(claims.user_id)
    .fetch_optional(&state.db.pool)
    .await?;

    if let Some(r) = row {
        use sqlx::Row;
        Ok(Json(NotificationPrefsResponse {
            user_id:           r.get("user_id"),
            email_enabled:     r.get("email_enabled"),
            teams_enabled:     r.get("teams_enabled"),
            teams_webhook_url: r.get("teams_webhook_url"),
        }))
    } else {
        // Return defaults if no row exists yet
        Ok(Json(NotificationPrefsResponse {
            user_id:           claims.user_id,
            email_enabled:     true,
            teams_enabled:     false,
            teams_webhook_url: None,
        }))
    }
}

/// PUT /notifications/preferences
pub async fn update_preferences(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdatePrefsRequest>,
) -> Result<Json<NotificationPrefsResponse>, ApiError> {
    // UPSERT: create row if not exists, then update fields
    sqlx::query(
        r#"INSERT INTO notification_preferences (user_id, email_enabled, teams_enabled, teams_webhook_url)
           VALUES (?, ?, ?, ?)
           ON DUPLICATE KEY UPDATE
               email_enabled     = COALESCE(VALUES(email_enabled),     email_enabled),
               teams_enabled     = COALESCE(VALUES(teams_enabled),     teams_enabled),
               teams_webhook_url = COALESCE(VALUES(teams_webhook_url), teams_webhook_url)"#,
    )
    .bind(claims.user_id)
    .bind(req.email_enabled.unwrap_or(true))
    .bind(req.teams_enabled.unwrap_or(false))
    .bind(req.teams_webhook_url.as_deref())
    .execute(&state.db.pool)
    .await?;

    // Re-fetch the updated row
    get_preferences(
        State(state),
        Extension(claims),
    )
    .await
}

/// GET /notifications/log   (admin only)
pub async fn get_notification_log(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<NotifLogQuery>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    if claims.role != "admin" {
        return Err(ApiError::Unauthorized("Admin role required".to_string()));
    }

    let limit  = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);

    let rows = sqlx::query(
        r#"SELECT id, user_id, event_type, channel, recipient, subject,
                  body_snippet, status, error_message, sent_at
           FROM notification_log
           WHERE (? IS NULL OR event_type = ?)
           ORDER BY sent_at DESC
           LIMIT ? OFFSET ?"#,
    )
    .bind(q.event_type.as_deref())
    .bind(q.event_type.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    use sqlx::Row;
    let result = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id":           r.get::<i32, _>("id"),
                "user_id":      r.get::<Option<i32>, _>("user_id"),
                "event_type":   r.get::<String, _>("event_type"),
                "channel":      r.get::<String, _>("channel"),
                "recipient":    r.get::<String, _>("recipient"),
                "subject":      r.get::<Option<String>, _>("subject"),
                "body_snippet": r.get::<Option<String>, _>("body_snippet"),
                "status":       r.get::<String, _>("status"),
                "error_message":r.get::<Option<String>, _>("error_message"),
                "sent_at":      r.get::<Option<chrono::NaiveDateTime>, _>("sent_at")
                                 .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(result))
}
