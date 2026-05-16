use super::models::Achievement;
use sqlx::MySqlPool;

pub fn compute_score(uom_type: &str, target: f64, actual: f64) -> f64 {
    match uom_type {
        "min_numeric" | "min_percent" => (actual / target).min(1.0) * 100.0,
        "max_numeric" | "max_percent" => (target / actual).min(1.0) * 100.0,
        "timeline" => {
            if actual <= target {
                100.0
            } else {
                0.0
            }
        }
        "zero" => {
            if actual == 0.0 {
                100.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

pub async fn upsert_achievement(
    pool: &MySqlPool,
    goal_id: i32,
    quarter: &str,
    actual_value: Option<f64>,
    actual_date: Option<chrono::NaiveDateTime>,
    status: &str,
) -> Result<Achievement, sqlx::Error> {
    let goal = sqlx::query_as::<_, super::models::Goal>(
        "SELECT id, sheet_id, thrust_area_id, title, description, uom_type, target_value, target_date, weightage, is_shared, shared_from_goal_id, sort_order, created_at, updated_at FROM goals WHERE id = ?",
    )
    .bind(goal_id)
    .fetch_one(pool)
    .await?;

    let computed = actual_value.map(|v| compute_score(&goal.uom_type, goal.target_value, v));

    let existing = sqlx::query_as::<_, Achievement>(
        "SELECT id, goal_id, quarter, actual_value, actual_date, status, computed_score, updated_at FROM achievements WHERE goal_id = ? AND quarter = ?",
    )
    .bind(goal_id)
    .bind(quarter)
    .fetch_optional(pool)
    .await?;

    match existing {
        Some(ach) => {
            sqlx::query(
                r#"
                UPDATE achievements
                SET actual_value = ?, actual_date = ?, status = ?, computed_score = ?
                WHERE id = ?
                "#,
            )
            .bind(actual_value)
            .bind(actual_date)
            .bind(status)
            .bind(computed)
            .bind(ach.id)
            .execute(pool)
            .await?;

            sqlx::query_as::<_, Achievement>(
                "SELECT id, goal_id, quarter, actual_value, actual_date, status, computed_score, updated_at FROM achievements WHERE id = ?",
            )
            .bind(ach.id)
            .fetch_one(pool)
            .await
        }
        None => {
            let result = sqlx::query(
                r#"
                INSERT INTO achievements (goal_id, quarter, actual_value, actual_date, status, computed_score)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(goal_id)
            .bind(quarter)
            .bind(actual_value)
            .bind(actual_date)
            .bind(status)
            .bind(computed)
            .execute(pool)
            .await?;

            let id = result.last_insert_id() as i32;
            sqlx::query_as::<_, Achievement>(
                "SELECT id, goal_id, quarter, actual_value, actual_date, status, computed_score, updated_at FROM achievements WHERE id = ?",
            )
            .bind(id)
            .fetch_one(pool)
            .await
        }
    }
}

pub async fn get_achievements_by_sheet(
    pool: &MySqlPool,
    sheet_id: i32,
) -> Result<Vec<Achievement>, sqlx::Error> {
    sqlx::query_as::<_, Achievement>(
        r#"
        SELECT a.id, a.goal_id, a.quarter, a.actual_value, a.actual_date, a.status, a.computed_score, a.updated_at
        FROM achievements a
        JOIN goals g ON g.id = a.goal_id
        WHERE g.sheet_id = ?
        ORDER BY g.sort_order ASC, g.id ASC, a.quarter
        "#,
    )
    .bind(sheet_id)
    .fetch_all(pool)
    .await
}

pub async fn get_achievements_by_goal(
    pool: &MySqlPool,
    goal_id: i32,
) -> Result<Vec<Achievement>, sqlx::Error> {
    sqlx::query_as::<_, Achievement>(
        r#"
        SELECT id, goal_id, quarter, actual_value, actual_date, status, computed_score, updated_at
        FROM achievements
        WHERE goal_id = ?
        ORDER BY quarter
        "#,
    )
    .bind(goal_id)
    .fetch_all(pool)
    .await
}

pub async fn get_sheet_score_summary(
    pool: &MySqlPool,
    sheet_id: i32,
) -> Result<f64, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT g.weightage, a.computed_score
        FROM goals g
        JOIN achievements a ON a.goal_id = g.id
        WHERE g.sheet_id = ?
        "#,
    )
    .bind(sheet_id)
    .fetch_all(pool)
    .await?;

    let total_weighted_score: f64 = rows.iter().fold(0.0, |acc, row| {
        use sqlx::Row;
        let weightage: f64 = row.get("weightage");
        let score: Option<f64> = row.get("computed_score");
        acc + weightage * score.unwrap_or(0.0) / 100.0
    });

    Ok(total_weighted_score)
}
