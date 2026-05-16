use super::models::{Department, GoalCycle, ThrustArea};
use sqlx::MySqlPool;

pub async fn create_cycle(
    pool: &MySqlPool,
    name: &str,
    goal_setting_opens: Option<chrono::NaiveDateTime>,
    q1_opens: Option<chrono::NaiveDateTime>,
    q2_opens: Option<chrono::NaiveDateTime>,
    q3_opens: Option<chrono::NaiveDateTime>,
    q4_opens: Option<chrono::NaiveDateTime>,
    is_active: Option<bool>,
    created_by: Option<i32>,
) -> Result<GoalCycle, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO goal_cycles (name, goal_setting_opens, q1_opens, q2_opens, q3_opens, q4_opens, is_active, created_by)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(name)
    .bind(goal_setting_opens)
    .bind(q1_opens)
    .bind(q2_opens)
    .bind(q3_opens)
    .bind(q4_opens)
    .bind(is_active)
    .bind(created_by)
    .execute(pool)
    .await?;

    let id = result.last_insert_id() as i32;
    find_cycle_by_id(pool, id).await.map(|c| c.unwrap())
}

pub async fn get_active_cycle(
    pool: &MySqlPool,
) -> Result<Option<GoalCycle>, sqlx::Error> {
    sqlx::query_as::<_, GoalCycle>(
        r#"
        SELECT id, name, goal_setting_opens, q1_opens, q2_opens, q3_opens, q4_opens, is_active, created_by, created_at, updated_at
        FROM goal_cycles
        WHERE is_active = 1
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_all_cycles(
    pool: &MySqlPool,
) -> Result<Vec<GoalCycle>, sqlx::Error> {
    sqlx::query_as::<_, GoalCycle>(
        r#"
        SELECT id, name, goal_setting_opens, q1_opens, q2_opens, q3_opens, q4_opens, is_active, created_by, created_at, updated_at
        FROM goal_cycles
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_cycle_by_id(
    pool: &MySqlPool,
    id: i32,
) -> Result<Option<GoalCycle>, sqlx::Error> {
    sqlx::query_as::<_, GoalCycle>(
        r#"
        SELECT id, name, goal_setting_opens, q1_opens, q2_opens, q3_opens, q4_opens, is_active, created_by, created_at, updated_at
        FROM goal_cycles
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn update_cycle(
    pool: &MySqlPool,
    id: i32,
    name: &str,
    goal_setting_opens: Option<chrono::NaiveDateTime>,
    q1_opens: Option<chrono::NaiveDateTime>,
    q2_opens: Option<chrono::NaiveDateTime>,
    q3_opens: Option<chrono::NaiveDateTime>,
    q4_opens: Option<chrono::NaiveDateTime>,
    is_active: Option<bool>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE goal_cycles
        SET name = ?, goal_setting_opens = ?, q1_opens = ?, q2_opens = ?, q3_opens = ?, q4_opens = ?, is_active = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(goal_setting_opens)
    .bind(q1_opens)
    .bind(q2_opens)
    .bind(q3_opens)
    .bind(q4_opens)
    .bind(is_active)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_active_cycle(
    pool: &MySqlPool,
    id: i32,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE goal_cycles SET is_active = 0")
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE goal_cycles SET is_active = 1 WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await
}

pub async fn list_departments(pool: &MySqlPool) -> Result<Vec<Department>, sqlx::Error> {
    sqlx::query_as::<_, Department>(
        r#"
        SELECT id, name, short_name, created_at
        FROM departments
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn create_department(
    pool: &MySqlPool,
    name: &str,
    short_name: &str,
) -> Result<Department, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO departments (name, short_name)
        VALUES (?, ?)
        "#,
    )
    .bind(name)
    .bind(short_name)
    .execute(pool)
    .await?;

    let id = result.last_insert_id() as i32;
    sqlx::query_as::<_, Department>(
        "SELECT id, name, short_name, created_at FROM departments WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn list_thrust_areas(pool: &MySqlPool) -> Result<Vec<ThrustArea>, sqlx::Error> {
    sqlx::query_as::<_, ThrustArea>(
        r#"
        SELECT id, name, department_id, created_by, created_at
        FROM thrust_areas
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn create_thrust_area(
    pool: &MySqlPool,
    name: &str,
    department_id: Option<i32>,
    created_by: Option<i32>,
) -> Result<ThrustArea, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO thrust_areas (name, department_id, created_by)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(name)
    .bind(department_id)
    .bind(created_by)
    .execute(pool)
    .await?;

    let id = result.last_insert_id() as i32;
    sqlx::query_as::<_, ThrustArea>(
        "SELECT id, name, department_id, created_by, created_at FROM thrust_areas WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
}
