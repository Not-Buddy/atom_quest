use crate::{
    api::ApiError,
    db::{
        audit, cycles, goals, users,
        models::{
            Claims, CreateCycleRequest, CreateDepartmentRequest, CreateThrustAreaRequest,
            CreateUserRequest, UpdateCycleRequest, UpdateUserRequest, UserResponse,
        },
    },
};
use axum::{extract::{Path, Query}, Extension, Json, extract::State};
use chrono::{NaiveDate, NaiveDateTime};
use std::sync::Arc;
use validator::Validate;
use crate::utils::hash_password;

use super::auth::AppState;

#[derive(serde::Deserialize)]
pub struct AuditLogQuery {
    pub table_name: Option<String>,
    pub record_id: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================
// Cycle endpoints
// ============================================================

pub async fn list_cycles(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<crate::db::models::GoalCycle>>, ApiError> {
    let result = cycles::list_all_cycles(&state.db.pool).await?;
    Ok(Json(result))
}

pub async fn create_cycle(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateCycleRequest>,
) -> Result<Json<crate::db::models::GoalCycle>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let goal_setting_opens = req
        .goal_setting_opens
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    let q1_opens = req
        .q1_opens
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    let q2_opens = req
        .q2_opens
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    let q3_opens = req
        .q3_opens
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    let q4_opens = req
        .q4_opens
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    let result = cycles::create_cycle(
        &state.db.pool,
        &req.name,
        goal_setting_opens,
        q1_opens,
        q2_opens,
        q3_opens,
        q4_opens,
        req.is_active,
        Some(claims.user_id),
    )
    .await?;

    Ok(Json(result))
}

pub async fn update_cycle(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(cycle_id): Path<i32>,
    Json(req): Json<UpdateCycleRequest>,
) -> Result<Json<crate::db::models::GoalCycle>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let existing = cycles::find_cycle_by_id(&state.db.pool, cycle_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Cycle not found".to_string()))?;

    let name = req.name.as_deref().unwrap_or(&existing.name);

    let goal_setting_opens = req
        .goal_setting_opens
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    let q1_opens = req
        .q1_opens
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    let q2_opens = req
        .q2_opens
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    let q3_opens = req
        .q3_opens
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    let q4_opens = req
        .q4_opens
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    cycles::update_cycle(
        &state.db.pool,
        cycle_id,
        name,
        goal_setting_opens,
        q1_opens,
        q2_opens,
        q3_opens,
        q4_opens,
        req.is_active,
    )
    .await?;

    let updated = cycles::find_cycle_by_id(&state.db.pool, cycle_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Cycle not found".to_string()))?;

    Ok(Json(updated))
}

// ============================================================
// Thrust area endpoints
// ============================================================

pub async fn list_thrust_areas(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<crate::db::models::ThrustArea>>, ApiError> {
    let result = cycles::list_thrust_areas(&state.db.pool).await?;
    Ok(Json(result))
}

pub async fn create_thrust_area(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateThrustAreaRequest>,
) -> Result<Json<crate::db::models::ThrustArea>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let result = cycles::create_thrust_area(
        &state.db.pool,
        &req.name,
        req.department_id,
        Some(claims.user_id),
    )
    .await?;

    Ok(Json(result))
}

// ============================================================
// Department endpoints
// ============================================================

pub async fn list_departments(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<crate::db::models::Department>>, ApiError> {
    let result = cycles::list_departments(&state.db.pool).await?;
    Ok(Json(result))
}

pub async fn create_department(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<Json<crate::db::models::Department>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let result = cycles::create_department(&state.db.pool, &req.name, &req.short_name).await?;
    Ok(Json(result))
}

// ============================================================
// User management endpoints
// ============================================================

#[derive(serde::Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
    let users_list = users::list_all(&state.db.pool).await?;

    let limit = query.limit.unwrap_or(50) as usize;
    let offset = query.offset.unwrap_or(0) as usize;

    let result: Vec<UserResponse> = users_list
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|u| u.into())
        .collect();

    Ok(Json(result))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let valid_roles = ["admin", "manager", "employee"];
    if !valid_roles.contains(&req.role.as_str()) {
        return Err(ApiError::ValidationError(
            "Role must be one of: admin, manager, employee".to_string(),
        ));
    }

    let password_hash = hash_password::hash(&req.password);

    let user = users::create_user(
        &state.db.pool,
        &req.email,
        &password_hash,
        &req.full_name,
        req.department_id,
        &req.role,
        req.manager_id,
    )
    .await?;

    Ok(Json(user.into()))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(user_id): Path<i32>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    if let Some(ref role) = req.role {
        let valid_roles = ["admin", "manager", "employee"];
        if !valid_roles.contains(&role.as_str()) {
            return Err(ApiError::ValidationError(
                "Role must be one of: admin, manager, employee".to_string(),
            ));
        }
    }

    let existing = users::find_by_id(&state.db.pool, user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let full_name = req.full_name.as_deref().unwrap_or(&existing.full_name);
    let department_id = req.department_id.or(existing.department_id);
    let role = req.role.as_deref().unwrap_or(&existing.role);
    let manager_id = req.manager_id.or(existing.manager_id);

    let user = users::update_user(
        &state.db.pool,
        user_id,
        full_name,
        department_id,
        role,
        manager_id,
    )
    .await?;

    if let Some(password) = req.password {
        let password_hash = hash_password::hash(&password);
        users::update_user_password(&state.db.pool, user_id, &password_hash).await?;
    }

    Ok(Json(user.into()))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<i32>,
) -> Result<Json<&'static str>, ApiError> {
    if user_id == claims.user_id {
        return Err(ApiError::ValidationError("Cannot delete yourself".to_string()));
    }

    let _existing = users::find_by_id(&state.db.pool, user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    users::delete_user(&state.db.pool, user_id).await?;

    Ok(Json("User deleted"))
}

// ============================================================
// Sheet unlock
// ============================================================

pub async fn unlock_sheet(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(sheet_id): Path<i32>,
) -> Result<Json<&'static str>, ApiError> {
    let status = goals::get_sheet_status(&state.db.pool, sheet_id)
        .await?
        .unwrap_or_default();

    if status != "locked" {
        return Err(ApiError::ValidationError(
            "Only locked sheets can be unlocked".to_string(),
        ));
    }

    goals::unlock_sheet(&state.db.pool, sheet_id).await?;

    Ok(Json("Sheet unlocked"))
}

// ============================================================
// Audit log
// ============================================================

pub async fn view_audit_log(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<Vec<crate::db::models::AuditLogEntry>>, ApiError> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let entries = audit::get_audit_logs(
        &state.db.pool,
        query.table_name.as_deref(),
        query.record_id,
        limit,
        offset,
    )
    .await?;

    Ok(Json(entries))
}

fn parse_date_str(s: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        })
        .map_err(|e| format!("Invalid date format: {}", e))
}
