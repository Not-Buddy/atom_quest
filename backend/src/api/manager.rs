use crate::{
    api::ApiError,
    db::{
        goals, cycles, users,
        models::{
            Claims, CreateCheckinRequest, GoalSheetSummary, ManagerEditGoalRequest,
            ReturnSheetRequest, SharedGoalPushRequest,
        },
    },
};
use axum::{extract::Path, Extension, Json, extract::State};
use chrono::NaiveDate;
use std::sync::Arc;
use validator::Validate;

use super::auth::AppState;

pub async fn list_team_sheets(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<GoalSheetSummary>>, ApiError> {
    let sheets = goals::list_sheets_by_manager(&state.db.pool, claims.user_id).await?;

    let mut summaries = Vec::new();
    for (sheet, user_name) in sheets {
        let goal_count = goals::count_goals_by_sheet(&state.db.pool, sheet.id).await?;
        let total_weightage = goals::get_total_weightage(&state.db.pool, sheet.id).await?;

        let cycle_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM goal_cycles WHERE id = ?",
        )
        .bind(sheet.cycle_id)
        .fetch_optional(&state.db.pool)
        .await?;

        summaries.push(GoalSheetSummary {
            id: sheet.id,
            user_id: sheet.user_id,
            user_name: Some(user_name),
            cycle_id: sheet.cycle_id,
            cycle_name,
            status: sheet.status,
            goal_count,
            total_weightage,
        });
    }

    Ok(Json(summaries))
}

pub async fn approve_sheet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(sheet_id): Path<i32>,
) -> Result<Json<&'static str>, ApiError> {
    let owner_id = goals::get_sheet_owner(&state.db.pool, sheet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

    let employee = users::find_by_id(&state.db.pool, owner_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Employee not found".to_string()))?;

    if employee.manager_id != Some(claims.user_id) {
        return Err(ApiError::NotFound("Sheet not found".to_string()));
    }

    let status = goals::get_sheet_status(&state.db.pool, sheet_id)
        .await?
        .unwrap_or_default();

    if status != "submitted" && status != "returned" {
        return Err(ApiError::ValidationError(
            "Can only approve submitted or returned sheets".to_string(),
        ));
    }

    goals::approve_sheet(&state.db.pool, sheet_id, claims.user_id).await?;

    Ok(Json("Sheet approved and locked"))
}

pub async fn return_sheet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(sheet_id): Path<i32>,
    Json(req): Json<ReturnSheetRequest>,
) -> Result<Json<&'static str>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let owner_id = goals::get_sheet_owner(&state.db.pool, sheet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

    let employee = users::find_by_id(&state.db.pool, owner_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Employee not found".to_string()))?;

    if employee.manager_id != Some(claims.user_id) {
        return Err(ApiError::NotFound("Sheet not found".to_string()));
    }

    let status = goals::get_sheet_status(&state.db.pool, sheet_id)
        .await?
        .unwrap_or_default();

    if status != "submitted" {
        return Err(ApiError::ValidationError(
            "Can only return submitted sheets".to_string(),
        ));
    }

    goals::return_sheet(&state.db.pool, sheet_id, &req.reason).await?;

    Ok(Json("Sheet returned to employee"))
}

pub async fn edit_goal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((_sheet_id, goal_id)): Path<(i32, i32)>,
    Json(req): Json<ManagerEditGoalRequest>,
) -> Result<Json<&'static str>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let goal = goals::find_goal_by_id(&state.db.pool, goal_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Goal not found".to_string()))?;

    let owner_id = goals::get_sheet_owner(&state.db.pool, goal.sheet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

    let employee = users::find_by_id(&state.db.pool, owner_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Employee not found".to_string()))?;

    if employee.manager_id != Some(claims.user_id) {
        return Err(ApiError::NotFound("Sheet not found".to_string()));
    }

    goals::manager_edit_goal(&state.db.pool, goal_id, req.target_value, req.weightage).await?;

    Ok(Json("Goal updated"))
}

pub async fn push_shared_goal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SharedGoalPushRequest>,
) -> Result<Json<&'static str>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    for &sheet_id in &req.sheet_ids {
        let owner_id = goals::get_sheet_owner(&state.db.pool, sheet_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Sheet {} not found", sheet_id)))?;

        let employee = users::find_by_id(&state.db.pool, owner_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Employee not found".to_string()))?;

        if employee.manager_id != Some(claims.user_id) {
            return Err(ApiError::Unauthorized(format!(
                "Not the manager of sheet {}'s owner",
                sheet_id
            )));
        }

        let status = goals::get_sheet_status(&state.db.pool, sheet_id)
            .await?
            .unwrap_or_default();

        if status != "draft" {
            return Err(ApiError::ValidationError(format!(
                "Sheet {} is not in draft status",
                sheet_id
            )));
        }
    }

    let cycle = cycles::get_active_cycle(&state.db.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound("No active goal cycle found".to_string()))?;

    let manager_sheet = goals::find_sheet_by_user_cycle(&state.db.pool, claims.user_id, cycle.id)
        .await?
        .ok_or_else(|| ApiError::NotFound("You must create your own goal sheet first".to_string()))?;

    let target_date = req
        .target_date
        .as_deref()
        .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| ApiError::ValidationError(format!("Invalid date format: {}", e)))?;

    let source_goal = goals::add_goal(
        &state.db.pool,
        manager_sheet.id,
        req.thrust_area_id,
        &req.title,
        req.description.as_deref(),
        &req.uom_type,
        req.target_value,
        target_date,
        req.weightage,
        Some(true),
        None,
        None,
    )
    .await?;

    goals::push_shared_goal(&state.db.pool, source_goal.id, &req.sheet_ids).await?;

    Ok(Json("Shared goal pushed to team"))
}

pub async fn view_team_checkins(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let data = goals::get_team_checkins(&state.db.pool, claims.user_id).await?;
    Ok(Json(data))
}

pub async fn add_checkin_comment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(sheet_id): Path<i32>,
    Json(req): Json<CreateCheckinRequest>,
) -> Result<Json<&'static str>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let valid_quarters = ["Q1", "Q2", "Q3", "Q4"];
    if !valid_quarters.contains(&req.quarter.as_str()) {
        return Err(ApiError::ValidationError(
            "Quarter must be one of: Q1, Q2, Q3, Q4".to_string(),
        ));
    }

    let owner_id = goals::get_sheet_owner(&state.db.pool, sheet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

    let employee = users::find_by_id(&state.db.pool, owner_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Employee not found".to_string()))?;

    if employee.manager_id != Some(claims.user_id) {
        return Err(ApiError::NotFound("Sheet not found".to_string()));
    }

    goals::create_checkin(
        &state.db.pool,
        sheet_id,
        &req.quarter,
        claims.user_id,
        &req.comment,
    )
    .await?;

    Ok(Json("Check-in comment added"))
}
