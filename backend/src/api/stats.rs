use crate::{
    api::{auth::AppState, ApiError},
    db::models::Claims,
};
use axum::{
    extract::State,
    Extension,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use sqlx::QueryBuilder;



#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub specialization: String,
    pub academic_year: String,
    pub total_students: i64,
    pub with_leetcode_profiles: i64,
    pub without_leetcode_profiles: i64,
    pub defaulters: i64,
}



/// Faculty-only endpoint to get statistics for their specialization and year
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<StatsResponse>>, ApiError> {
    // Verify this is a faculty token
    let specialization = claims.specialization
        .ok_or_else(|| ApiError::Unauthorized("Not a faculty token".to_string()))?;
    
    // Get the faculty's academic_year directly from claims
    let academic_year = claims.academic_year
        .as_ref()
        .ok_or_else(|| ApiError::Unauthorized("Faculty academic year not found in token".to_string()))?;



    // Get statistics
    let stats = get_specialization_stats(&state.db.pool, &specialization, academic_year).await?;



    Ok(Json(stats))
}

#[allow(dead_code)]
#[derive(Debug)]
struct SpecializationStats {
    specialization: String,
    total_students: i64,
    with_leetcode_profiles: i64,
    defaulters: i64,
}



async fn get_specialization_stats(
    pool: &sqlx::MySqlPool,
    specialization: &str,
    academic_year: &str,
) -> Result<Vec<StatsResponse>, sqlx::Error> {
    
    // Handle year VI - admin access to all specializations and years
    if academic_year == "VI" {
        let mut query = QueryBuilder::new(
            "SELECT 
                specialization,
                COUNT(*) as total_students,
                CAST(COALESCE(SUM(CASE WHEN leetcode_username IS NOT NULL THEN 1 ELSE 0 END), 0) AS SIGNED) as with_leetcode_profiles,
                CAST(COALESCE(SUM(CASE WHEN COALESCE(leetcode_solved_last_30_days, 0) < 15 THEN 1 ELSE 0 END), 0) AS SIGNED) as defaulters
            FROM STUDENTS
            WHERE academic_year IN ('I', 'II', 'III', 'IV')
            GROUP BY specialization
            ORDER BY specialization"
        );

        let results = query
            .build_query_as::<(String, i64, i64, i64)>()
            .fetch_all(pool)
            .await?;

        Ok(results
            .into_iter()
            .map(|(spec, total, with_leetcode, defaulters)| StatsResponse {
                specialization: spec,
                academic_year: academic_year.to_string(),
                total_students: total,
                with_leetcode_profiles: with_leetcode,
                without_leetcode_profiles: total - with_leetcode,
                defaulters,
            })
            .collect())
    } else {
        // Regular faculty or year V coordinator
        let mut query = QueryBuilder::new(
            "SELECT 
                COUNT(*) as total_students,
                CAST(COALESCE(SUM(CASE WHEN leetcode_username IS NOT NULL THEN 1 ELSE 0 END), 0) AS SIGNED) as with_leetcode_profiles,
                CAST(COALESCE(SUM(CASE WHEN COALESCE(leetcode_solved_last_30_days, 0) < 15 THEN 1 ELSE 0 END), 0) AS SIGNED) as defaulters
            FROM STUDENTS
            WHERE specialization = "
        );
        query.push_bind(specialization);
        
        // Handle year V - all years except V for specific specialization
        if academic_year == "V" {
            query.push(" AND academic_year != 'V'");
        } else {
            // Specific year for specific specialization
            query.push(" AND academic_year = ");
            query.push_bind(academic_year);
        }

        let result = query
            .build_query_as::<(i64, i64, i64)>()
            .fetch_one(pool)
            .await?;

        Ok(vec![StatsResponse {
            specialization: specialization.to_string(),
            academic_year: academic_year.to_string(),
            total_students: result.0,
            with_leetcode_profiles: result.1,
            without_leetcode_profiles: result.0 - result.1,
            defaulters: result.2,
        }])
    }
}
