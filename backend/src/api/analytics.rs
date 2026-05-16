/// 5.4 — Analytics Module
///
/// Endpoints:
///   GET /analytics/qoq-trends          — Quarter-on-Quarter achievement trends
///   GET /analytics/heatmap             — Org-wide progress heatmap data
///   GET /analytics/goal-distribution   — Breakdown by Thrust Area, UoM, status
///   GET /analytics/manager-effectiveness — Check-in completion per manager
///
/// All endpoints require a valid JWT (any role). Some add extra filtering for
/// non-admin users (employees see only their own data).
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

// ─── Query params ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AnalyticsQuery {
    pub cycle_id:     Option<i32>,
    pub department_id: Option<i32>,
    pub manager_id:   Option<i32>,
    pub user_id:      Option<i32>,
}

// ─── 5.4.1  Quarter-on-Quarter Trends ────────────────────────────────────────

#[derive(Serialize)]
pub struct QoQTrendEntry {
    pub user_name:    String,
    pub department:   Option<String>,
    pub goal_title:   String,
    pub uom_type:     String,
    pub weightage:    f64,
    pub q1_score:     Option<f64>,
    pub q2_score:     Option<f64>,
    pub q3_score:     Option<f64>,
    pub q4_score:     Option<f64>,
    /// Weighted overall score (sum of score * weightage / 100)
    pub weighted_avg: Option<f64>,
}

pub async fn qoq_trends(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<AnalyticsQuery>,
) -> Result<Json<Vec<QoQTrendEntry>>, ApiError> {
    // Employees can only see their own data
    let user_filter = if claims.role == "employee" {
        Some(claims.user_id)
    } else {
        q.user_id
    };

    let rows = sqlx::query(
        r#"
        SELECT
            u.full_name           AS user_name,
            d.short_name          AS department,
            g.title               AS goal_title,
            g.uom_type,
            g.weightage,
            MAX(CASE WHEN a.quarter = 'q1' THEN a.computed_score END) AS q1_score,
            MAX(CASE WHEN a.quarter = 'q2' THEN a.computed_score END) AS q2_score,
            MAX(CASE WHEN a.quarter = 'q3' THEN a.computed_score END) AS q3_score,
            MAX(CASE WHEN a.quarter = 'q4' THEN a.computed_score END) AS q4_score,
            SUM(
                COALESCE(a.computed_score, 0) * g.weightage / 100
            ) / NULLIF(COUNT(DISTINCT a.quarter), 0) AS weighted_avg
        FROM goals g
        JOIN goal_sheets gs ON gs.id = g.sheet_id
        JOIN users u        ON u.id  = gs.user_id
        JOIN goal_cycles gc ON gc.id = gs.cycle_id
        LEFT JOIN departments d ON d.id = u.department_id
        LEFT JOIN achievements a ON a.goal_id = g.id
        WHERE (? IS NULL OR gc.id = ?)
          AND (? IS NULL OR u.department_id = ?)
          AND (? IS NULL OR u.manager_id = ?)
          AND (? IS NULL OR u.id = ?)
        GROUP BY u.id, u.full_name, d.short_name, g.id, g.title, g.uom_type, g.weightage
        ORDER BY u.full_name, g.title
        "#,
    )
    .bind(q.cycle_id).bind(q.cycle_id)
    .bind(q.department_id).bind(q.department_id)
    .bind(q.manager_id).bind(q.manager_id)
    .bind(user_filter).bind(user_filter)
    .fetch_all(&state.db.pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        use sqlx::Row;
        result.push(QoQTrendEntry {
            user_name:    row.get("user_name"),
            department:   row.get("department"),
            goal_title:   row.get("goal_title"),
            uom_type:     row.get("uom_type"),
            weightage:    row.get("weightage"),
            q1_score:     row.get("q1_score"),
            q2_score:     row.get("q2_score"),
            q3_score:     row.get("q3_score"),
            q4_score:     row.get("q4_score"),
            weighted_avg: row.get("weighted_avg"),
        });
    }
    Ok(Json(result))
}

// ─── 5.4.2  Org-Wide Progress Heatmap ────────────────────────────────────────

/// One cell in the heatmap: employee × quarter → average progress score (0-100).
#[derive(Serialize)]
pub struct HeatmapCell {
    pub user_id:        i32,
    pub user_name:      String,
    pub department:     Option<String>,
    pub manager_name:   Option<String>,
    pub q1_avg:         Option<f64>,
    pub q2_avg:         Option<f64>,
    pub q3_avg:         Option<f64>,
    pub q4_avg:         Option<f64>,
    /// Overall average across all quarters with data
    pub overall_avg:    Option<f64>,
    /// Sheet status for context
    pub sheet_status:   Option<String>,
}

pub async fn org_heatmap(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<AnalyticsQuery>,
) -> Result<Json<Vec<HeatmapCell>>, ApiError> {
    // Employees only see their own row
    let user_filter = if claims.role == "employee" {
        Some(claims.user_id)
    } else {
        q.user_id
    };

    let rows = sqlx::query(
        r#"
        SELECT
            u.id                  AS user_id,
            u.full_name           AS user_name,
            d.short_name          AS department,
            m.full_name           AS manager_name,
            gs.status             AS sheet_status,
            AVG(CASE WHEN a.quarter = 'q1' THEN a.computed_score END) AS q1_avg,
            AVG(CASE WHEN a.quarter = 'q2' THEN a.computed_score END) AS q2_avg,
            AVG(CASE WHEN a.quarter = 'q3' THEN a.computed_score END) AS q3_avg,
            AVG(CASE WHEN a.quarter = 'q4' THEN a.computed_score END) AS q4_avg,
            AVG(a.computed_score)                                       AS overall_avg
        FROM users u
        JOIN goal_sheets gs    ON gs.user_id = u.id
        JOIN goal_cycles gc    ON gc.id      = gs.cycle_id
        LEFT JOIN departments d ON d.id      = u.department_id
        LEFT JOIN users m       ON m.id      = u.manager_id
        LEFT JOIN goals g       ON g.sheet_id = gs.id
        LEFT JOIN achievements a ON a.goal_id = g.id
        WHERE (? IS NULL OR gc.id = ?)
          AND (? IS NULL OR u.department_id = ?)
          AND (? IS NULL OR u.manager_id = ?)
          AND (? IS NULL OR u.id = ?)
          AND gc.is_active = 1
        GROUP BY u.id, u.full_name, d.short_name, m.full_name, gs.status
        ORDER BY d.short_name, u.full_name
        "#,
    )
    .bind(q.cycle_id).bind(q.cycle_id)
    .bind(q.department_id).bind(q.department_id)
    .bind(q.manager_id).bind(q.manager_id)
    .bind(user_filter).bind(user_filter)
    .fetch_all(&state.db.pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        use sqlx::Row;
        result.push(HeatmapCell {
            user_id:      row.get("user_id"),
            user_name:    row.get("user_name"),
            department:   row.get("department"),
            manager_name: row.get("manager_name"),
            sheet_status: row.get("sheet_status"),
            q1_avg:       row.get("q1_avg"),
            q2_avg:       row.get("q2_avg"),
            q3_avg:       row.get("q3_avg"),
            q4_avg:       row.get("q4_avg"),
            overall_avg:  row.get("overall_avg"),
        });
    }
    Ok(Json(result))
}

// ─── 5.4.3  Goal Distribution Analysis ───────────────────────────────────────

#[derive(Serialize)]
pub struct GoalDistributionEntry {
    pub dimension:  String,   // e.g. thrust area name, UoM type, or status
    pub dimension_type: String, // "thrust_area" | "uom_type" | "status"
    pub count:      i64,
    pub total_weightage: f64,
    pub avg_score:  Option<f64>,
}

pub async fn goal_distribution(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<AnalyticsQuery>,
) -> Result<Json<Vec<GoalDistributionEntry>>, ApiError> {
    let user_filter = if claims.role == "employee" {
        Some(claims.user_id)
    } else {
        q.user_id
    };

    // Thrust-area breakdown
    let ta_rows = sqlx::query(
        r#"
        SELECT
            COALESCE(ta.name, 'Unassigned') AS dimension,
            COUNT(g.id)                      AS cnt,
            SUM(g.weightage)                 AS total_w,
            AVG(a.computed_score)            AS avg_score
        FROM goals g
        JOIN goal_sheets gs ON gs.id = g.sheet_id
        JOIN goal_cycles gc ON gc.id = gs.cycle_id
        JOIN users u        ON u.id  = gs.user_id
        LEFT JOIN thrust_areas ta ON ta.id = g.thrust_area_id
        LEFT JOIN achievements a  ON a.goal_id = g.id
        WHERE (? IS NULL OR gc.id = ?)
          AND (? IS NULL OR u.department_id = ?)
          AND (? IS NULL OR u.id = ?)
        GROUP BY ta.id, ta.name
        ORDER BY cnt DESC
        "#,
    )
    .bind(q.cycle_id).bind(q.cycle_id)
    .bind(q.department_id).bind(q.department_id)
    .bind(user_filter).bind(user_filter)
    .fetch_all(&state.db.pool)
    .await?;

    // UoM-type breakdown
    let uom_rows = sqlx::query(
        r#"
        SELECT
            g.uom_type                 AS dimension,
            COUNT(g.id)                AS cnt,
            SUM(g.weightage)           AS total_w,
            AVG(a.computed_score)      AS avg_score
        FROM goals g
        JOIN goal_sheets gs ON gs.id = g.sheet_id
        JOIN goal_cycles gc ON gc.id = gs.cycle_id
        JOIN users u        ON u.id  = gs.user_id
        LEFT JOIN achievements a ON a.goal_id = g.id
        WHERE (? IS NULL OR gc.id = ?)
          AND (? IS NULL OR u.department_id = ?)
          AND (? IS NULL OR u.id = ?)
        GROUP BY g.uom_type
        ORDER BY cnt DESC
        "#,
    )
    .bind(q.cycle_id).bind(q.cycle_id)
    .bind(q.department_id).bind(q.department_id)
    .bind(user_filter).bind(user_filter)
    .fetch_all(&state.db.pool)
    .await?;

    // Achievement status breakdown
    let status_rows = sqlx::query(
        r#"
        SELECT
            COALESCE(a.status, 'not_started') AS dimension,
            COUNT(a.id)                        AS cnt,
            0.0                                AS total_w,
            AVG(a.computed_score)              AS avg_score
        FROM goals g
        JOIN goal_sheets gs ON gs.id = g.sheet_id
        JOIN goal_cycles gc ON gc.id = gs.cycle_id
        JOIN users u        ON u.id  = gs.user_id
        LEFT JOIN achievements a ON a.goal_id = g.id
        WHERE (? IS NULL OR gc.id = ?)
          AND (? IS NULL OR u.department_id = ?)
          AND (? IS NULL OR u.id = ?)
        GROUP BY a.status
        ORDER BY cnt DESC
        "#,
    )
    .bind(q.cycle_id).bind(q.cycle_id)
    .bind(q.department_id).bind(q.department_id)
    .bind(user_filter).bind(user_filter)
    .fetch_all(&state.db.pool)
    .await?;

    use sqlx::Row;
    let mut result: Vec<GoalDistributionEntry> = Vec::new();

    for row in ta_rows {
        result.push(GoalDistributionEntry {
            dimension:       row.get("dimension"),
            dimension_type:  "thrust_area".to_string(),
            count:           row.get("cnt"),
            total_weightage: row.get::<Option<f64>, _>("total_w").unwrap_or(0.0),
            avg_score:       row.get("avg_score"),
        });
    }
    for row in uom_rows {
        result.push(GoalDistributionEntry {
            dimension:       row.get("dimension"),
            dimension_type:  "uom_type".to_string(),
            count:           row.get("cnt"),
            total_weightage: row.get::<Option<f64>, _>("total_w").unwrap_or(0.0),
            avg_score:       row.get("avg_score"),
        });
    }
    for row in status_rows {
        result.push(GoalDistributionEntry {
            dimension:       row.get("dimension"),
            dimension_type:  "achievement_status".to_string(),
            count:           row.get("cnt"),
            total_weightage: 0.0,
            avg_score:       row.get("avg_score"),
        });
    }

    Ok(Json(result))
}

// ─── 5.4.4  Manager Effectiveness Dashboard ───────────────────────────────────

#[derive(Serialize)]
pub struct ManagerEffectivenessEntry {
    pub manager_id:             i32,
    pub manager_name:           String,
    pub department:             Option<String>,
    pub team_size:              i64,
    /// Total sheets awaiting approval
    pub pending_approvals:      i64,
    /// Sheets approved within the cycle
    pub sheets_approved:        i64,
    /// Sheets still in draft/submitted state (not actioned)
    pub sheets_not_actioned:    i64,
    /// Number of Q1 check-in comments left by this manager
    pub q1_checkins_done:       i64,
    /// Number of Q2 check-in comments left by this manager
    pub q2_checkins_done:       i64,
    /// Number of Q3 check-in comments left by this manager
    pub q3_checkins_done:       i64,
    /// Number of Q4 check-in comments left by this manager
    pub q4_checkins_done:       i64,
    /// Average score of this manager's team (weighted)
    pub team_avg_score:         Option<f64>,
}

pub async fn manager_effectiveness(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<AnalyticsQuery>,
) -> Result<Json<Vec<ManagerEffectivenessEntry>>, ApiError> {
    // Only managers and admins can see this
    if claims.role == "employee" {
        return Err(ApiError::Unauthorized(
            "Manager or admin role required".to_string(),
        ));
    }

    // If the caller is a manager, restrict to themselves
    let manager_filter = if claims.role == "manager" {
        Some(claims.user_id)
    } else {
        q.manager_id
    };

    let rows = sqlx::query(
        r#"
        SELECT
            m.id                    AS manager_id,
            m.full_name             AS manager_name,
            d.short_name            AS department,
            COUNT(DISTINCT u.id)    AS team_size,
            SUM(CASE WHEN gs.status = 'submitted'                          THEN 1 ELSE 0 END) AS pending_approvals,
            SUM(CASE WHEN gs.status IN ('locked','approved')               THEN 1 ELSE 0 END) AS sheets_approved,
            SUM(CASE WHEN gs.status IN ('draft','submitted','returned')     THEN 1 ELSE 0 END) AS sheets_not_actioned,
            SUM(CASE WHEN cc.quarter = 'q1'                                THEN 1 ELSE 0 END) AS q1_checkins_done,
            SUM(CASE WHEN cc.quarter = 'q2'                                THEN 1 ELSE 0 END) AS q2_checkins_done,
            SUM(CASE WHEN cc.quarter = 'q3'                                THEN 1 ELSE 0 END) AS q3_checkins_done,
            SUM(CASE WHEN cc.quarter = 'q4'                                THEN 1 ELSE 0 END) AS q4_checkins_done,
            AVG(a.computed_score)   AS team_avg_score
        FROM users m
        JOIN users u          ON u.manager_id  = m.id
        JOIN goal_sheets gs   ON gs.user_id    = u.id
        JOIN goal_cycles gc   ON gc.id         = gs.cycle_id
        LEFT JOIN departments d  ON d.id       = m.department_id
        LEFT JOIN checkin_comments cc ON cc.goal_sheet_id = gs.id AND cc.manager_id = m.id
        LEFT JOIN goals g     ON g.sheet_id    = gs.id
        LEFT JOIN achievements a ON a.goal_id  = g.id
        WHERE m.role IN ('manager','admin')
          AND (? IS NULL OR gc.id = ?)
          AND (? IS NULL OR m.id = ?)
        GROUP BY m.id, m.full_name, d.short_name
        ORDER BY m.full_name
        "#,
    )
    .bind(q.cycle_id).bind(q.cycle_id)
    .bind(manager_filter).bind(manager_filter)
    .fetch_all(&state.db.pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        use sqlx::Row;
        result.push(ManagerEffectivenessEntry {
            manager_id:          row.get("manager_id"),
            manager_name:        row.get("manager_name"),
            department:          row.get("department"),
            team_size:           row.get("team_size"),
            pending_approvals:   row.get("pending_approvals"),
            sheets_approved:     row.get("sheets_approved"),
            sheets_not_actioned: row.get("sheets_not_actioned"),
            q1_checkins_done:    row.get("q1_checkins_done"),
            q2_checkins_done:    row.get("q2_checkins_done"),
            q3_checkins_done:    row.get("q3_checkins_done"),
            q4_checkins_done:    row.get("q4_checkins_done"),
            team_avg_score:      row.get("team_avg_score"),
        });
    }
    Ok(Json(result))
}
