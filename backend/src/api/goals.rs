use crate::{
    api::ApiError,
    db::{
        goals, cycles, users,
        models::{
            Claims, CreateGoalRequest, GoalResponse, GoalSheetResponse, GoalSheetSummary,
            UpdateGoalRequest,
        },
    },
};
use axum::{extract::Path, Extension, Json, extract::State};
use chrono::NaiveDate;
use std::sync::Arc;
use validator::Validate;

use super::auth::AppState;

pub async fn create_goal_sheet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<GoalSheetResponse>, ApiError> {
    let cycle = cycles::get_active_cycle(&state.db.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound("No active goal cycle found".to_string()))?;

    // Check if sheet already exists for this user+cycle
    if let Some(existing) = goals::find_sheet_by_user_cycle(&state.db.pool, claims.user_id, cycle.id).await? {
        let detail = goals::get_sheet_detail(&state.db.pool, existing.id).await?;
        return Ok(Json(detail));
    }

    let sheet = goals::create_sheet(&state.db.pool, claims.user_id, cycle.id).await?;
    let detail = goals::get_sheet_detail(&state.db.pool, sheet.id).await?;

    Ok(Json(detail))
}

pub async fn list_goal_sheets(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<GoalSheetSummary>>, ApiError> {
    let sheets = goals::list_sheets_by_user(&state.db.pool, claims.user_id).await?;

    let mut summaries = Vec::new();
    for sheet in sheets {
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
            user_name: None,
            cycle_id: sheet.cycle_id,
            cycle_name,
            status: sheet.status,
            goal_count,
            total_weightage,
        });
    }

    Ok(Json(summaries))
}

pub async fn get_sheet_detail(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(sheet_id): Path<i32>,
) -> Result<Json<GoalSheetResponse>, ApiError> {
    let detail = goals::get_sheet_detail(&state.db.pool, sheet_id).await?;

    // Owner can view. Manager of the owner can also view.
    if detail.user_id != claims.user_id {
        let owner = users::find_by_id(&state.db.pool, detail.user_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;
        if owner.manager_id != Some(claims.user_id) {
            return Err(ApiError::NotFound("Sheet not found".to_string()));
        }
    }

    Ok(Json(detail))
}

pub async fn submit_sheet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(sheet_id): Path<i32>,
) -> Result<Json<&'static str>, ApiError> {
    let owner_id = goals::get_sheet_owner(&state.db.pool, sheet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

    if owner_id != claims.user_id {
        return Err(ApiError::NotFound("Sheet not found".to_string()));
    }

    let status = goals::get_sheet_status(&state.db.pool, sheet_id)
        .await?
        .unwrap_or_default();

    if status != "draft" {
        return Err(ApiError::ValidationError("Only draft sheets can be submitted".to_string()));
    }

    goals::submit_sheet(&state.db.pool, sheet_id, claims.user_id).await?;

    Ok(Json("Sheet submitted for approval"))
}

pub async fn add_goal_to_sheet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(sheet_id): Path<i32>,
    Json(req): Json<CreateGoalRequest>,
) -> Result<Json<GoalResponse>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let owner_id = goals::get_sheet_owner(&state.db.pool, sheet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

    if owner_id != claims.user_id {
        return Err(ApiError::NotFound("Sheet not found".to_string()));
    }

    let status = goals::get_sheet_status(&state.db.pool, sheet_id)
        .await?
        .unwrap_or_default();

    if status != "draft" {
        return Err(ApiError::ValidationError("Can only add goals to draft sheets".to_string()));
    }

    let current_weightage = goals::get_total_weightage(&state.db.pool, sheet_id).await?;

    if current_weightage + req.weightage > 100.0 {
        return Err(ApiError::ValidationError(format!(
            "Total weightage would exceed 100% (current: {:.1}%, adding: {:.1}%)",
            current_weightage, req.weightage
        )));
    }

    let goal_count = goals::count_goals_by_sheet(&state.db.pool, sheet_id).await?;

    if goal_count >= 8 {
        return Err(ApiError::ValidationError(
            "Maximum 8 goals allowed per sheet".to_string(),
        ));
    }

    let target_date = req
        .target_date
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    if let Some(ref date) = target_date {
        let today = chrono::Local::now().date_naive();
        if *date < today {
            return Err(ApiError::ValidationError(
                "Target date cannot be in the past".to_string()
            ));
        }
    }

    let goal = goals::add_goal(
        &state.db.pool,
        sheet_id,
        req.thrust_area_id,
        &req.title,
        req.description.as_deref(),
        &req.uom_type,
        req.target_value,
        target_date,
        req.weightage,
        req.is_shared,
        None,
        req.sort_order,
    )
    .await?;

    Ok(Json(GoalResponse {
        id: goal.id,
        sheet_id: goal.sheet_id,
        thrust_area_id: goal.thrust_area_id,
        thrust_area_name: None,
        title: goal.title,
        description: goal.description,
        uom_type: goal.uom_type,
        target_value: goal.target_value,
        target_date: goal.target_date.map(|d| d.format("%Y-%m-%d").to_string()),
        weightage: goal.weightage,
        is_shared: goal.is_shared.unwrap_or(false),
        shared_from_goal_id: goal.shared_from_goal_id,
        sort_order: goal.sort_order.unwrap_or(0),
        achievements: vec![],
    }))
}

pub async fn update_goal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(goal_id): Path<i32>,
    Json(req): Json<UpdateGoalRequest>,
) -> Result<Json<GoalResponse>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let goal = goals::find_goal_by_id(&state.db.pool, goal_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Goal not found".to_string()))?;

    let owner_id = goals::get_sheet_owner(&state.db.pool, goal.sheet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

    if owner_id != claims.user_id {
        return Err(ApiError::NotFound("Goal not found".to_string()));
    }

    let status = goals::get_sheet_status(&state.db.pool, goal.sheet_id)
        .await?
        .unwrap_or_default();

    if status != "draft" {
        return Err(ApiError::ValidationError("Can only edit goals on draft sheets".to_string()));
    }

    if let Some(w) = req.weightage {
        let current_weightage = goals::get_total_weightage(&state.db.pool, goal.sheet_id).await?;
        let new_total = current_weightage - goal.weightage + w;
        if new_total > 100.0 {
            return Err(ApiError::ValidationError(format!(
                "Total weightage would exceed 100% (would be: {:.1}%)",
                new_total
            )));
        }
    }

    let target_date = req
        .target_date
        .as_deref()
        .map(parse_date_str)
        .transpose()
        .map_err(|e| ApiError::ValidationError(e))?;

    if let Some(ref date) = target_date {
        let today = chrono::Local::now().date_naive();
        if *date < today {
            return Err(ApiError::ValidationError(
                "Target date cannot be in the past".to_string()
            ));
        }
    }

    let updated = goals::update_goal(
        &state.db.pool,
        goal_id,
        req.thrust_area_id.map(Some),
        req.title.as_deref(),
        req.description.as_deref().map(|d| Some(d)),
        req.uom_type.as_deref(),
        req.target_value,
        target_date.map(Some),
        req.weightage,
        req.is_shared.map(Some),
        req.sort_order.map(Some),
    )
    .await?;

    Ok(Json(GoalResponse {
        id: updated.id,
        sheet_id: updated.sheet_id,
        thrust_area_id: updated.thrust_area_id,
        thrust_area_name: None,
        title: updated.title,
        description: updated.description,
        uom_type: updated.uom_type,
        target_value: updated.target_value,
        target_date: updated.target_date.map(|d| d.format("%Y-%m-%d").to_string()),
        weightage: updated.weightage,
        is_shared: updated.is_shared.unwrap_or(false),
        shared_from_goal_id: updated.shared_from_goal_id,
        sort_order: updated.sort_order.unwrap_or(0),
        achievements: vec![],
    }))
}

pub async fn delete_goal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(goal_id): Path<i32>,
) -> Result<Json<&'static str>, ApiError> {
    let goal = goals::find_goal_by_id(&state.db.pool, goal_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Goal not found".to_string()))?;

    let owner_id = goals::get_sheet_owner(&state.db.pool, goal.sheet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

    if owner_id != claims.user_id {
        return Err(ApiError::NotFound("Goal not found".to_string()));
    }

    let status = goals::get_sheet_status(&state.db.pool, goal.sheet_id)
        .await?
        .unwrap_or_default();

    if status != "draft" {
        return Err(ApiError::ValidationError(
            "Can only delete goals from draft sheets".to_string(),
        ));
    }

    goals::delete_goal(&state.db.pool, goal_id).await?;

    Ok(Json("Goal deleted"))
}

fn parse_date_str(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))
}
