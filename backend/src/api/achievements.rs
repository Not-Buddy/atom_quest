use crate::{
    api::ApiError,
    db::{
        achievements, goals, users,
        models::{AchievementResponse, AchievementUpdateRequest, Claims},
    },
};
use axum::{extract::Path, Extension, Json, extract::State};
use chrono::NaiveDate;
use std::sync::Arc;
use validator::Validate;

use super::auth::AppState;

pub async fn get_achievements_for_sheet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(sheet_id): Path<i32>,
) -> Result<Json<Vec<AchievementResponse>>, ApiError> {
    let owner_id = goals::get_sheet_owner(&state.db.pool, sheet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

    if owner_id != claims.user_id {
        let employee = users::find_by_id(&state.db.pool, owner_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

        if employee.manager_id != Some(claims.user_id) {
            return Err(ApiError::NotFound("Sheet not found".to_string()));
        }
    }

    let achievements = achievements::get_achievements_by_sheet(&state.db.pool, sheet_id).await?;

    let result: Vec<AchievementResponse> = achievements.into_iter().map(|a| a.into()).collect();

    Ok(Json(result))
}

pub async fn update_achievement(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((goal_id, quarter)): Path<(i32, String)>,
    Json(req): Json<AchievementUpdateRequest>,
) -> Result<Json<AchievementResponse>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let valid_quarters = ["Q1", "Q2", "Q3", "Q4"];
    if !valid_quarters.contains(&quarter.as_str()) {
        return Err(ApiError::ValidationError(
            "Quarter must be one of: Q1, Q2, Q3, Q4".to_string(),
        ));
    }

    let goal = goals::find_goal_by_id(&state.db.pool, goal_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Goal not found".to_string()))?;

    let owner_id = goals::get_sheet_owner(&state.db.pool, goal.sheet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Sheet not found".to_string()))?;

    if owner_id != claims.user_id {
        return Err(ApiError::NotFound("Goal not found".to_string()));
    }

    let sheet_status = goals::get_sheet_status(&state.db.pool, goal.sheet_id)
        .await?
        .unwrap_or_default();

    if sheet_status != "approved" && sheet_status != "locked" {
        return Err(ApiError::ValidationError(
            "Can only log achievements for approved or locked sheets".to_string(),
        ));
    }

    let actual_date = req
        .actual_date
        .as_deref()
        .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| ApiError::ValidationError(format!("Invalid date format: {}", e)))?;

    let status = req.status.as_deref().unwrap_or("on_track");

    let achievement = achievements::upsert_achievement(
        &state.db.pool,
        goal_id,
        &quarter,
        req.actual_value,
        actual_date,
        status,
    )
    .await?;

    Ok(Json(achievement.into()))
}
