use crate::{
    api::{auth::AppState, ApiError},
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub academic_year: String, // I, II, III, or IV
}

// 1. Updated Struct to match your new requirement list
#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub full_name: String,
    pub registration_number: String,
    pub specialization: Option<String>,
    pub academic_year: Option<String>,
    pub github_username: Option<String>,
    pub leetcode_username: Option<String>,
    pub codechef_username: Option<String>,
    pub codeforces_username: Option<String>,
    pub linkedin_url: Option<String>,
    pub total_solved_last_30_days: i32,
    pub rank: usize,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub academic_year: String,
    pub total_students: usize,
    pub leaderboard: Vec<LeaderboardEntry>,
}

/// Public endpoint to get top 50 students by academic year
pub async fn get_leaderboard(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<Json<LeaderboardResponse>, ApiError> {
    // Validate academic year
    if !["I", "II", "III", "IV"].contains(&params.academic_year.as_str()) {
        return Err(ApiError::ValidationError(
            "Invalid academic_year. Must be one of: I, II, III, IV".to_string(),
        ));
    }

    // Get top 50 students for this academic year
    let students = get_top_students_by_academic_year(&state.db.pool, &params.academic_year).await?;

    let total_students = students.len();

    // Add ranking and map to response struct
    let leaderboard: Vec<LeaderboardEntry> = students
        .into_iter()
        .enumerate()
        .map(|(index, student)| LeaderboardEntry {
            full_name: student.full_name,
            registration_number: student.registration_number,
            specialization: student.specialization,
            academic_year: student.academic_year,
            github_username: student.github_username,
            leetcode_username: student.leetcode_username,
            codechef_username: student.codechef_username,
            codeforces_username: student.codeforces_username,
            linkedin_url: student.linkedin_url,
            total_solved_last_30_days: student.total_solved_last_30_days,
            rank: index + 1,
        })
        .collect();

    Ok(Json(LeaderboardResponse {
        academic_year: params.academic_year,
        total_students,
        leaderboard,
    }))
}

// 2. Updated Internal Data Struct to match DB columns
#[derive(Debug)]
struct StudentLeaderboardData {
    full_name: String,
    registration_number: String,
    specialization: Option<String>,
    academic_year: Option<String>,
    github_username: Option<String>,
    leetcode_username: Option<String>,
    codechef_username: Option<String>,
    codeforces_username: Option<String>,
    linkedin_url: Option<String>,
    total_solved_last_30_days: i32,
}

async fn get_top_students_by_academic_year(
    pool: &sqlx::MySqlPool,
    academic_year: &str,
) -> Result<Vec<StudentLeaderboardData>, sqlx::Error> {
    // 3. Updated Query to select all requested fields and sort by total_solved
    let students = sqlx::query_as!(
        StudentLeaderboardData,
        r#"
        SELECT
            full_name,
            registration_number,
            specialization,
            academic_year,
            github_username,
            leetcode_username,
            codechef_username,
            codeforces_username,
            linkedin_url,
            COALESCE(total_solved_last_30_days, 0) as `total_solved_last_30_days: i32`
        FROM STUDENTS
        WHERE academic_year = ?
        ORDER BY total_solved_last_30_days DESC, registration_number ASC
        LIMIT 50
        "#,
        academic_year
    )
    .fetch_all(pool)
    .await?;

    Ok(students)
}