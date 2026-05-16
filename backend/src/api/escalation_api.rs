/// 5.3 — Escalation API endpoints (Admin & HR only)
use crate::{
    api::ApiError,
    db::models::Claims,
    escalation,
};
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

use super::auth::AppState;

// ─── Request DTOs ─────────────────────────────────────────────────────────────

#[derive(Deserialize, Validate)]
pub struct CreateRuleRequest {
    #[validate(length(min = 1, max = 200))]
    pub name:               String,
    pub description:        Option<String>,
    /// One of: goal_not_submitted | goal_not_approved | checkin_not_completed
    pub trigger_event:      String,
    pub days_after_trigger: i32,
    pub notify_employee:    Option<bool>,
    pub notify_manager:     Option<bool>,
    pub notify_hr:          Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateRuleRequest {
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct EscalationLogQuery {
    pub user_id: Option<i32>,
    pub limit:   Option<i64>,
    pub offset:  Option<i64>,
}

#[derive(Deserialize)]
pub struct ResolveEscalationRequest {
    pub note: Option<String>,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// GET /admin/escalation-rules
pub async fn list_rules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<escalation::EscalationRule>>, ApiError> {
    require_admin_or_manager(&claims)?;
    let rules = escalation::list_all_rules(&state.db.pool).await?;
    Ok(Json(rules))
}

/// POST /admin/escalation-rules
pub async fn create_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRuleRequest>,
) -> Result<Json<escalation::EscalationRule>, ApiError> {
    require_admin(&claims)?;
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let valid_events = [
        "goal_not_submitted",
        "goal_not_approved",
        "checkin_not_completed",
    ];
    if !valid_events.contains(&req.trigger_event.as_str()) {
        return Err(ApiError::ValidationError(
            "trigger_event must be one of: goal_not_submitted, goal_not_approved, checkin_not_completed".to_string(),
        ));
    }

    let rule = escalation::create_rule(
        &state.db.pool,
        &req.name,
        req.description.as_deref(),
        &req.trigger_event,
        req.days_after_trigger,
        req.notify_employee.unwrap_or(true),
        req.notify_manager.unwrap_or(true),
        req.notify_hr.unwrap_or(false),
        Some(claims.user_id),
    )
    .await?;

    Ok(Json(rule))
}

/// PUT /admin/escalation-rules/:id
pub async fn update_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(rule_id): Path<i32>,
    Json(req): Json<UpdateRuleRequest>,
) -> Result<Json<&'static str>, ApiError> {
    require_admin(&claims)?;
    escalation::update_rule_active(&state.db.pool, rule_id, req.is_active).await?;
    Ok(Json("Rule updated"))
}

/// GET /admin/escalation-log
pub async fn list_log(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<EscalationLogQuery>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    require_admin_or_manager(&claims)?;
    let limit  = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    let entries = escalation::list_escalation_log(
        &state.db.pool, limit, offset, q.user_id,
    ).await?;
    Ok(Json(entries))
}

/// PUT /admin/escalation-log/:id/resolve
pub async fn resolve_log_entry(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(log_id): Path<i32>,
    Json(req): Json<ResolveEscalationRequest>,
) -> Result<Json<&'static str>, ApiError> {
    require_admin_or_manager(&claims)?;
    escalation::resolve_escalation(
        &state.db.pool,
        log_id,
        claims.user_id,
        req.note.as_deref(),
    )
    .await?;
    Ok(Json("Escalation resolved"))
}

// ─── Guards ───────────────────────────────────────────────────────────────────

fn require_admin(claims: &Claims) -> Result<(), ApiError> {
    if claims.role != "admin" {
        Err(ApiError::Unauthorized("Admin role required".to_string()))
    } else {
        Ok(())
    }
}

fn require_admin_or_manager(claims: &Claims) -> Result<(), ApiError> {
    if claims.role != "admin" && claims.role != "manager" {
        Err(ApiError::Unauthorized("Manager or admin role required".to_string()))
    } else {
        Ok(())
    }
}
