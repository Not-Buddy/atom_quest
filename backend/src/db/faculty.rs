use crate::db::models::Faculty;
use sqlx::MySqlPool;

pub async fn find_faculty_by_credentials(
    pool: &MySqlPool,
    specialization: &str,
    username: &str,
) -> Result<Option<(Faculty, String)>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, specialization, academic_year, username, created_at, password_hash
        FROM FACULTY
        WHERE specialization = ? AND username = ?
        "#,
        specialization,
        username
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let faculty = Faculty {
            id: row.id,
            specialization: row.specialization,
            academic_year: row.academic_year,  // Add academic_year
            username: row.username,
            created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now().naive_utc()).and_utc(),
        };
        Ok(Some((faculty, row.password_hash)))
    } else {
        Ok(None)
    }
}

