use super::models::student::{PasswordResetToken, Student};
use sqlx::MySqlPool;
use chrono::Utc;

pub async fn find_student_by_email(
    pool: &MySqlPool,
    email: &str,
) -> Result<Option<Student>, sqlx::Error> {
    let student = sqlx::query_as::<_, Student>(
        r#"
        SELECT * FROM STUDENTS WHERE email = ?
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(student)
}

pub async fn find_student_by_id(
    pool: &MySqlPool,
    id: i32,
) -> Result<Option<Student>, sqlx::Error> {
    let student = sqlx::query_as::<_, Student>(
        r#"
        SELECT * FROM STUDENTS WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(student)
}

pub async fn get_student_by_ra(
    pool: &MySqlPool,
    registration_number: &str,
) -> Result<Student, sqlx::Error> {
    let student = sqlx::query_as::<_, Student>(
        r#"
        SELECT * FROM STUDENTS WHERE registration_number = ?
        "#,
    )
    .bind(registration_number)
    .fetch_one(pool)
    .await?;

    Ok(student)
}

pub async fn get_all_students(
    pool: &MySqlPool,
) -> Result<Vec<Student>, sqlx::Error> {
    let students = sqlx::query_as::<_, Student>(
        r#"
        SELECT * FROM STUDENTS
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(students)
}

pub async fn find_student_by_leetcode_username(
    pool: &MySqlPool,
    leetcode_username: &str,
) -> Result<Option<Student>, sqlx::Error> {
    let student = sqlx::query_as::<_, Student>(
        r#"
        SELECT * FROM STUDENTS WHERE leetcode_username = ?
        "#,
    )
    .bind(leetcode_username)
    .fetch_optional(pool)
    .await?;

    Ok(student)
}
pub async fn find_student_by_codechef_username(
    pool: &MySqlPool,
    codechef_username: &str,
) -> Result<Option<Student>, sqlx::Error> {
    if codechef_username.is_empty() {
        return Ok(None);
    }

    let student = sqlx::query_as::<_, Student>(
        "SELECT * FROM STUDENTS WHERE codechef_username = ?"
    )
    .bind(codechef_username)
    .fetch_optional(pool)
    .await?;

    Ok(student)
}

pub async fn find_student_by_codeforces_username(
    pool: &MySqlPool,
    codeforces_username: &str,
) -> Result<Option<Student>, sqlx::Error> {
    if codeforces_username.is_empty() {
        return Ok(None);
    }

    let student = sqlx::query_as::<_, Student>(
        "SELECT * FROM STUDENTS WHERE codeforces_username = ?"
    )
    .bind(codeforces_username)
    .fetch_optional(pool)
    .await?;

    Ok(student)
}


pub async fn update_student_links(
    pool: &MySqlPool,
    student_id: i32,
    github_username: Option<&str>,
    linkedin_url: Option<&str>,
    leetcode_username: Option<&str>,
    codechef_username: Option<&str>,
    codeforces_username: Option<&str>,
) -> Result<Student, sqlx::Error> {
    sqlx::query(
        "UPDATE STUDENTS SET
        github_username = ?,
        linkedin_url = ?,
        leetcode_username = ?,
        codechef_username = ?,
        codeforces_username = ?
        WHERE id = ?"
    )
    .bind(github_username)
    .bind(linkedin_url)
    .bind(leetcode_username)
    .bind(codechef_username)
    .bind(codeforces_username)
    .bind(student_id)
    .execute(pool)
    .await?;

    // Fetch and return updated student
    find_student_by_id(pool, student_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn set_password_reset_token(
    pool: &MySqlPool,
    student_id: i32,
    token: &str,
    expires_at: chrono::NaiveDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO password_reset_tokens (student_id, token, expires_at) VALUES (?, ?, ?)"
    )
    .bind(student_id)
    .bind(token)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_student_by_reset_token(
    pool: &MySqlPool,
    token: &str,
) -> Result<Option<(Student, PasswordResetToken)>, sqlx::Error> {
    let reset_token_record = sqlx::query_as::<_, PasswordResetToken>(
        r#"
        SELECT *
        FROM password_reset_tokens
        WHERE token = ? AND expires_at > ? AND used = FALSE
        "#
    )
    .bind(token)
    .bind(Utc::now().naive_utc())
    .fetch_optional(pool)
    .await?;

    if let Some(prt) = reset_token_record {
        let student = find_student_by_id(pool, prt.student_id).await?;
        if let Some(s) = student {
            Ok(Some((s, prt)))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub async fn update_password(
    pool: &MySqlPool,
    student_id: i32,
    token_id: i32,
    new_password_hash: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE STUDENTS SET password = ? WHERE id = ?"
    )
    .bind(new_password_hash)
    .bind(student_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE password_reset_tokens SET used = TRUE WHERE id = ?"
    )
    .bind(token_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

