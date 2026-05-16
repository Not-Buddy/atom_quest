use super::models::{
    AchievementReportEntry, CheckinCommentResponse, CompletionDashboardEntry, Goal, GoalResponse,
    GoalSheet, GoalSheetResponse,
};
use chrono::NaiveDateTime;
use sqlx::MySqlPool;

pub async fn create_sheet(
    pool: &MySqlPool,
    user_id: i32,
    cycle_id: i32,
) -> Result<GoalSheet, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO goal_sheets (user_id, cycle_id, status)
        VALUES (?, ?, 'draft')
        "#,
    )
    .bind(user_id)
    .bind(cycle_id)
    .execute(pool)
    .await?;

    let id = result.last_insert_id() as i32;
    find_sheet_by_id(pool, id).await.map(|s| s.unwrap())
}

pub async fn find_sheet_by_id(
    pool: &MySqlPool,
    id: i32,
) -> Result<Option<GoalSheet>, sqlx::Error> {
    sqlx::query_as::<_, GoalSheet>(
        r#"
        SELECT id, user_id, cycle_id, status, submitted_at, approved_at, approved_by, returned_reason, created_at, updated_at
        FROM goal_sheets
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_sheet_by_user_cycle(
    pool: &MySqlPool,
    user_id: i32,
    cycle_id: i32,
) -> Result<Option<GoalSheet>, sqlx::Error> {
    sqlx::query_as::<_, GoalSheet>(
        r#"
        SELECT id, user_id, cycle_id, status, submitted_at, approved_at, approved_by, returned_reason, created_at, updated_at
        FROM goal_sheets
        WHERE user_id = ? AND cycle_id = ?
        "#,
    )
    .bind(user_id)
    .bind(cycle_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_sheets_by_user(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<Vec<GoalSheet>, sqlx::Error> {
    sqlx::query_as::<_, GoalSheet>(
        r#"
        SELECT id, user_id, cycle_id, status, submitted_at, approved_at, approved_by, returned_reason, created_at, updated_at
        FROM goal_sheets
        WHERE user_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn list_sheets_by_manager(
    pool: &MySqlPool,
    manager_id: i32,
) -> Result<Vec<(GoalSheet, String)>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            gs.id, gs.user_id, gs.cycle_id, gs.status, gs.submitted_at, gs.approved_at, gs.approved_by, gs.returned_reason, gs.created_at, gs.updated_at,
            u.full_name
        FROM goal_sheets gs
        JOIN users u ON u.id = gs.user_id
        WHERE u.manager_id = ?
        ORDER BY gs.created_at DESC
        "#,
    )
    .bind(manager_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        use sqlx::Row;
        let sheet = GoalSheet {
            id: row.get("id"),
            user_id: row.get("user_id"),
            cycle_id: row.get("cycle_id"),
            status: row.get("status"),
            submitted_at: row.get("submitted_at"),
            approved_at: row.get("approved_at"),
            approved_by: row.get("approved_by"),
            returned_reason: row.get("returned_reason"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        let user_name: String = row.get("full_name");
        result.push((sheet, user_name));
    }
    Ok(result)
}

pub async fn list_all_sheets(
    pool: &MySqlPool,
) -> Result<Vec<(GoalSheet, String)>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            gs.id, gs.user_id, gs.cycle_id, gs.status, gs.submitted_at, gs.approved_at, gs.approved_by, gs.returned_reason, gs.created_at, gs.updated_at,
            u.full_name
        FROM goal_sheets gs
        JOIN users u ON u.id = gs.user_id
        ORDER BY gs.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        use sqlx::Row;
        let sheet = GoalSheet {
            id: row.get("id"),
            user_id: row.get("user_id"),
            cycle_id: row.get("cycle_id"),
            status: row.get("status"),
            submitted_at: row.get("submitted_at"),
            approved_at: row.get("approved_at"),
            approved_by: row.get("approved_by"),
            returned_reason: row.get("returned_reason"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        let user_name: String = row.get("full_name");
        result.push((sheet, user_name));
    }
    Ok(result)
}

pub async fn update_sheet_status(
    pool: &MySqlPool,
    sheet_id: i32,
    status: &str,
    submitted_at: Option<chrono::NaiveDateTime>,
    approved_at: Option<chrono::NaiveDateTime>,
    approved_by: Option<i32>,
    returned_reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE goal_sheets
        SET status = ?, submitted_at = ?, approved_at = ?, approved_by = ?, returned_reason = ?
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(submitted_at)
    .bind(approved_at)
    .bind(approved_by)
    .bind(returned_reason)
    .bind(sheet_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn add_goal(
    pool: &MySqlPool,
    sheet_id: i32,
    thrust_area_id: Option<i32>,
    title: &str,
    description: Option<&str>,
    uom_type: &str,
    target_value: f64,
    target_date: Option<chrono::NaiveDateTime>,
    weightage: f64,
    is_shared: Option<bool>,
    shared_from_goal_id: Option<i32>,
    sort_order: Option<i32>,
) -> Result<Goal, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO goals (sheet_id, thrust_area_id, title, description, uom_type, target_value, target_date, weightage, is_shared, shared_from_goal_id, sort_order)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(sheet_id)
    .bind(thrust_area_id)
    .bind(title)
    .bind(description)
    .bind(uom_type)
    .bind(target_value)
    .bind(target_date)
    .bind(weightage)
    .bind(is_shared)
    .bind(shared_from_goal_id)
    .bind(sort_order)
    .execute(pool)
    .await?;

    let id = result.last_insert_id() as i32;
    find_goal_by_id(pool, id).await.map(|g| g.unwrap())
}

pub async fn get_goals_by_sheet(
    pool: &MySqlPool,
    sheet_id: i32,
) -> Result<Vec<(Goal, Option<String>)>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            g.id, g.sheet_id, g.thrust_area_id, g.title, g.description, g.uom_type, g.target_value, g.target_date, g.weightage, g.is_shared, g.shared_from_goal_id, g.sort_order, g.created_at, g.updated_at,
            ta.name
        FROM goals g
        LEFT JOIN thrust_areas ta ON ta.id = g.thrust_area_id
        WHERE g.sheet_id = ?
        ORDER BY g.sort_order ASC, g.id ASC
        "#,
    )
    .bind(sheet_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        use sqlx::Row;
        let goal = Goal {
            id: row.get("id"),
            sheet_id: row.get("sheet_id"),
            thrust_area_id: row.get("thrust_area_id"),
            title: row.get("title"),
            description: row.get("description"),
            uom_type: row.get("uom_type"),
            target_value: row.get("target_value"),
            target_date: row.get("target_date"),
            weightage: row.get("weightage"),
            is_shared: row.get("is_shared"),
            shared_from_goal_id: row.get("shared_from_goal_id"),
            sort_order: row.get("sort_order"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        let thrust_name: Option<String> = row.get("name");
        result.push((goal, thrust_name));
    }
    Ok(result)
}

pub async fn find_goal_by_id(
    pool: &MySqlPool,
    goal_id: i32,
) -> Result<Option<Goal>, sqlx::Error> {
    sqlx::query_as::<_, Goal>(
        r#"
        SELECT id, sheet_id, thrust_area_id, title, description, uom_type, target_value, target_date, weightage, is_shared, shared_from_goal_id, sort_order, created_at, updated_at
        FROM goals
        WHERE id = ?
        "#,
    )
    .bind(goal_id)
    .fetch_optional(pool)
    .await
}

pub async fn update_goal(
    pool: &MySqlPool,
    goal_id: i32,
    thrust_area_id: Option<Option<i32>>,
    title: Option<&str>,
    description: Option<Option<&str>>,
    uom_type: Option<&str>,
    target_value: Option<f64>,
    target_date: Option<Option<chrono::NaiveDateTime>>,
    weightage: Option<f64>,
    is_shared: Option<Option<bool>>,
    sort_order: Option<Option<i32>>,
) -> Result<Goal, sqlx::Error> {
    let mut tx = pool.begin().await?;

    if let Some(v) = thrust_area_id {
        sqlx::query("UPDATE goals SET thrust_area_id = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(v) = title {
        sqlx::query("UPDATE goals SET title = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(v) = description {
        sqlx::query("UPDATE goals SET description = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(v) = uom_type {
        sqlx::query("UPDATE goals SET uom_type = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(v) = target_value {
        sqlx::query("UPDATE goals SET target_value = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(v) = target_date {
        sqlx::query("UPDATE goals SET target_date = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(v) = weightage {
        sqlx::query("UPDATE goals SET weightage = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(v) = is_shared {
        sqlx::query("UPDATE goals SET is_shared = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(v) = sort_order {
        sqlx::query("UPDATE goals SET sort_order = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    find_goal_by_id(pool, goal_id).await.map(|g| g.unwrap())
}

pub async fn delete_goal(pool: &MySqlPool, goal_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM goals WHERE id = ?")
        .bind(goal_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_total_weightage(
    pool: &MySqlPool,
    sheet_id: i32,
) -> Result<f64, sqlx::Error> {
    let row = sqlx::query(
        "SELECT COALESCE(SUM(weightage), 0.0) AS total FROM goals WHERE sheet_id = ?",
    )
    .bind(sheet_id)
    .fetch_one(pool)
    .await?;

    use sqlx::Row;
    let total: f64 = row.get("total");
    Ok(total)
}

pub async fn count_goals_by_sheet(
    pool: &MySqlPool,
    sheet_id: i32,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) AS cnt FROM goals WHERE sheet_id = ?")
        .bind(sheet_id)
        .fetch_one(pool)
        .await?;

    use sqlx::Row;
    let count: i64 = row.get("cnt");
    Ok(count)
}

pub async fn get_sheet_owner(pool: &MySqlPool, sheet_id: i32) -> Result<Option<i32>, sqlx::Error> {
    let row = sqlx::query("SELECT user_id FROM goal_sheets WHERE id = ?")
        .bind(sheet_id)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| {
        use sqlx::Row;
        r.get("user_id")
    }))
}

pub async fn get_sheet_status(
    pool: &MySqlPool,
    sheet_id: i32,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query("SELECT status FROM goal_sheets WHERE id = ?")
        .bind(sheet_id)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| {
        use sqlx::Row;
        r.get("status")
    }))
}

pub async fn get_sheet_detail(
    pool: &MySqlPool,
    sheet_id: i32,
) -> Result<GoalSheetResponse, sqlx::Error> {
    let sheet = find_sheet_by_id(pool, sheet_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

    let user_name: Option<String> = sqlx::query_scalar(
        "SELECT full_name FROM users WHERE id = ?",
    )
    .bind(sheet.user_id)
    .fetch_optional(pool)
    .await?;

    let cycle_name: Option<String> = sqlx::query_scalar(
        "SELECT name FROM goal_cycles WHERE id = ?",
    )
    .bind(sheet.cycle_id)
    .fetch_optional(pool)
    .await?;

    let goals_rows = get_goals_by_sheet(pool, sheet_id).await?;

    let goal_ids: Vec<i32> = goals_rows.iter().map(|(g, _)| g.id).collect();

    let achievements = if goal_ids.is_empty() {
        Vec::new()
    } else {
        super::achievements::get_achievements_by_sheet(pool, sheet_id).await?
    };

    let mut goal_responses: Vec<GoalResponse> = Vec::new();
    for (goal, thrust_name) in goals_rows {
        let goal_achievements: Vec<_> = achievements
            .iter()
            .filter(|a| a.goal_id == goal.id)
            .cloned()
            .map(crate::db::models::AchievementResponse::from)
            .collect();

        goal_responses.push(GoalResponse {
            id: goal.id,
            sheet_id: goal.sheet_id,
            thrust_area_id: goal.thrust_area_id,
            thrust_area_name: thrust_name,
            title: goal.title,
            description: goal.description,
            uom_type: goal.uom_type,
            target_value: goal.target_value,
            target_date: goal.target_date.map(|d| d.format("%Y-%m-%d").to_string()),
            weightage: goal.weightage,
            is_shared: goal.is_shared.unwrap_or(false),
            shared_from_goal_id: goal.shared_from_goal_id,
            sort_order: goal.sort_order.unwrap_or(0),
            achievements: goal_achievements,
        });
    }

    let total_weightage = get_total_weightage(pool, sheet_id).await?;

    let checkin_rows = sqlx::query_as::<_, (i32, i32, String, i32, String, Option<NaiveDateTime>)>(
        r#"
        SELECT cc.id, cc.goal_sheet_id, cc.quarter, cc.manager_id, cc.comment, cc.created_at
        FROM checkin_comments cc
        WHERE cc.goal_sheet_id = ?
        ORDER BY cc.created_at DESC
        "#,
    )
    .bind(sheet_id)
    .fetch_all(pool)
    .await?;

    let mut checkins = Vec::new();
    for (id, goal_sheet_id, quarter, manager_id, comment, created_at) in checkin_rows {
        let manager_name: Option<String> = sqlx::query_scalar(
            "SELECT full_name FROM users WHERE id = ?",
        )
        .bind(manager_id)
        .fetch_optional(pool)
        .await?;

        checkins.push(CheckinCommentResponse {
            id,
            goal_sheet_id,
            quarter,
            manager_id,
            manager_name,
            comment,
            created_at: created_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
        });
    }

    Ok(GoalSheetResponse {
        id: sheet.id,
        user_id: sheet.user_id,
        user_name,
        cycle_id: sheet.cycle_id,
        cycle_name,
        status: sheet.status,
        submitted_at: sheet.submitted_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
        approved_at: sheet.approved_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
        approved_by: sheet.approved_by,
        returned_reason: sheet.returned_reason,
        goals: goal_responses,
        total_weightage,
        checkins,
    })
}

pub async fn submit_sheet(
    pool: &MySqlPool,
    sheet_id: i32,
    user_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE goal_sheets SET status = 'submitted', submitted_at = NOW() WHERE id = ?",
    )
    .bind(sheet_id)
    .execute(pool)
    .await?;

    let _ = user_id;
    Ok(())
}

pub async fn approve_sheet(
    pool: &MySqlPool,
    sheet_id: i32,
    approved_by: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE goal_sheets SET status = 'locked', approved_at = NOW(), approved_by = ? WHERE id = ?",
    )
    .bind(approved_by)
    .bind(sheet_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn return_sheet(
    pool: &MySqlPool,
    sheet_id: i32,
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE goal_sheets SET status = 'returned', returned_reason = ? WHERE id = ?",
    )
    .bind(reason)
    .bind(sheet_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlock_sheet(pool: &MySqlPool, sheet_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE goal_sheets SET status = 'approved' WHERE id = ? AND status = 'locked'")
        .bind(sheet_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn manager_edit_goal(
    pool: &MySqlPool,
    goal_id: i32,
    target_value: Option<f64>,
    weightage: Option<f64>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    if let Some(v) = target_value {
        sqlx::query("UPDATE goals SET target_value = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(v) = weightage {
        sqlx::query("UPDATE goals SET weightage = ? WHERE id = ?")
            .bind(v)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await
}

pub async fn push_shared_goal(
    pool: &MySqlPool,
    shared_from_goal_id: i32,
    sheet_ids: &[i32],
) -> Result<(), sqlx::Error> {
    let source = find_goal_by_id(pool, shared_from_goal_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

    let mut tx = pool.begin().await?;

    for &sheet_id in sheet_ids {
        sqlx::query(
            r#"
            INSERT INTO goals (sheet_id, thrust_area_id, title, description, uom_type, target_value, target_date, weightage, is_shared, shared_from_goal_id, sort_order)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, true, ?, ?)
            "#,
        )
        .bind(sheet_id)
        .bind(source.thrust_area_id)
        .bind(&source.title)
        .bind(&source.description)
        .bind(&source.uom_type)
        .bind(source.target_value)
        .bind(source.target_date)
        .bind(source.weightage)
        .bind(source.id)
        .bind(source.sort_order)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await
}

pub async fn create_checkin(
    pool: &MySqlPool,
    sheet_id: i32,
    quarter: &str,
    manager_id: i32,
    comment: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO checkin_comments (goal_sheet_id, quarter, manager_id, comment)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(sheet_id)
    .bind(quarter)
    .bind(manager_id)
    .bind(comment)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_team_checkins(
    pool: &MySqlPool,
    manager_id: i32,
) -> Result<serde_json::Value, sqlx::Error> {
    use serde_json::json;

    let rows = sqlx::query(
        r#"
        SELECT cc.id, cc.goal_sheet_id, cc.quarter, cc.manager_id, cc.comment, cc.created_at,
               u.full_name AS employee_name
        FROM checkin_comments cc
        JOIN goal_sheets gs ON gs.id = cc.goal_sheet_id
        JOIN users u ON u.id = gs.user_id
        WHERE u.manager_id = ?
        ORDER BY cc.created_at DESC
        "#,
    )
    .bind(manager_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        use sqlx::Row;
        let id: i32 = row.get("id");
        let goal_sheet_id: i32 = row.get("goal_sheet_id");
        let quarter: String = row.get("quarter");
        let mgr_id: i32 = row.get("manager_id");
        let comment: String = row.get("comment");
        let created_at_raw: Option<NaiveDateTime> = row.get("created_at");
        let employee_name: String = row.get("employee_name");

        result.push(json!({
            "id": id,
            "goal_sheet_id": goal_sheet_id,
            "quarter": quarter,
            "manager_id": mgr_id,
            "comment": comment,
            "created_at": created_at_raw.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            "employee_name": employee_name,
        }));
    }

    Ok(serde_json::Value::Array(result))
}

pub async fn get_achievement_report(
    pool: &MySqlPool,
) -> Result<Vec<AchievementReportEntry>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            u.full_name AS user_name,
            d.short_name AS department,
            gc.name AS cycle_name,
            gs.status AS sheet_status,
            g.title AS goal_title,
            g.uom_type,
            g.target_value,
            g.weightage,
            MAX(CASE WHEN a.quarter = 'Q1' THEN a.actual_value END) AS q1_actual,
            MAX(CASE WHEN a.quarter = 'Q1' THEN a.computed_score END) AS q1_score,
            MAX(CASE WHEN a.quarter = 'Q2' THEN a.actual_value END) AS q2_actual,
            MAX(CASE WHEN a.quarter = 'Q2' THEN a.computed_score END) AS q2_score,
            MAX(CASE WHEN a.quarter = 'Q3' THEN a.actual_value END) AS q3_actual,
            MAX(CASE WHEN a.quarter = 'Q3' THEN a.computed_score END) AS q3_score,
            MAX(CASE WHEN a.quarter = 'Q4' THEN a.actual_value END) AS q4_actual,
            MAX(CASE WHEN a.quarter = 'Q4' THEN a.computed_score END) AS q4_score
        FROM goals g
        JOIN goal_sheets gs ON gs.id = g.sheet_id
        JOIN users u ON u.id = gs.user_id
        JOIN goal_cycles gc ON gc.id = gs.cycle_id
        LEFT JOIN departments d ON d.id = u.department_id
        LEFT JOIN achievements a ON a.goal_id = g.id
        GROUP BY u.id, u.full_name, d.short_name, gc.name, gs.status, g.id, g.title, g.uom_type, g.target_value, g.weightage
        ORDER BY u.full_name, gc.name, gs.status
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut entries = Vec::new();
    for row in rows {
        use sqlx::Row;
        entries.push(AchievementReportEntry {
            user_name: row.get("user_name"),
            department: row.get("department"),
            cycle_name: row.get("cycle_name"),
            sheet_status: row.get("sheet_status"),
            goal_title: row.get("goal_title"),
            uom_type: row.get("uom_type"),
            target_value: row.get("target_value"),
            weightage: row.get("weightage"),
            q1_actual: row.get("q1_actual"),
            q1_score: row.get("q1_score"),
            q2_actual: row.get("q2_actual"),
            q2_score: row.get("q2_score"),
            q3_actual: row.get("q3_actual"),
            q3_score: row.get("q3_score"),
            q4_actual: row.get("q4_actual"),
            q4_score: row.get("q4_score"),
        });
    }

    Ok(entries)
}

pub async fn get_completion_dashboard(
    pool: &MySqlPool,
) -> Result<Vec<CompletionDashboardEntry>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            d.short_name AS department,
            COUNT(gs.id) AS total_sheets,
            SUM(CASE WHEN gs.status = 'draft' THEN 1 ELSE 0 END) AS draft_count,
            SUM(CASE WHEN gs.status = 'submitted' THEN 1 ELSE 0 END) AS submitted_count,
            SUM(CASE WHEN gs.status = 'approved' THEN 1 ELSE 0 END) AS approved_count,
            SUM(CASE WHEN gs.status = 'returned' THEN 1 ELSE 0 END) AS returned_count,
            SUM(CASE WHEN gs.status = 'locked' THEN 1 ELSE 0 END) AS locked_count
        FROM goal_sheets gs
        JOIN users u ON u.id = gs.user_id
        LEFT JOIN departments d ON d.id = u.department_id
        GROUP BY d.id, d.short_name
        ORDER BY d.short_name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut entries = Vec::new();
    for row in rows {
        use sqlx::Row;
        entries.push(CompletionDashboardEntry {
            department: row.get("department"),
            total_sheets: row.get("total_sheets"),
            draft_count: row.get("draft_count"),
            submitted_count: row.get("submitted_count"),
            approved_count: row.get("approved_count"),
            returned_count: row.get("returned_count"),
            locked_count: row.get("locked_count"),
        });
    }

    Ok(entries)
}
