use super::models::{PasswordResetToken, User};
use chrono::NaiveDateTime;
use sqlx::MySqlPool;

// ─── Azure AD helpers ────────────────────────────────────────────────────────

/// Extended User row that includes the Azure AD columns added in migration 002.
#[derive(Debug, sqlx::FromRow, Clone)]
pub struct UserWithAzure {
    pub id:            i32,
    pub email:         String,
    pub password_hash: String,
    pub full_name:     String,
    pub department_id: Option<i32>,
    pub role:          String,
    pub manager_id:    Option<i32>,
    pub created_at:    Option<NaiveDateTime>,
    pub updated_at:    Option<NaiveDateTime>,
    pub azure_oid:     Option<String>,
    pub azure_upn:     Option<String>,
    pub auth_provider: Option<String>,
}

impl From<UserWithAzure> for User {
    fn from(u: UserWithAzure) -> Self {
        User {
            id: u.id,
            email: u.email,
            password_hash: u.password_hash,
            full_name: u.full_name,
            department_id: u.department_id,
            role: u.role,
            manager_id: u.manager_id,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

/// Upsert a user that logged in via Azure AD.
/// Priority: (1) match by azure_oid, (2) match by email, (3) create new.
pub async fn upsert_azure_user(
    pool: &MySqlPool,
    email: &str,
    full_name: &str,
    azure_oid: &str,
    azure_upn: &str,
    role: &str,
) -> Result<User, sqlx::Error> {
    // Try by OID first
    if let Some(existing) = find_by_azure_oid(pool, azure_oid).await? {
        // Update name/upn in case they changed in Azure AD
        sqlx::query(
            "UPDATE users SET full_name = ?, azure_upn = ?, role = ? WHERE id = ?",
        )
        .bind(full_name)
        .bind(azure_upn)
        .bind(role)
        .bind(existing.id)
        .execute(pool)
        .await?;
        return find_by_id(pool, existing.id).await.map(|u| u.unwrap());
    }

    // Try by email (existing local user linking their Azure account)
    if let Some(existing) = find_by_email(pool, email).await? {
        sqlx::query(
            "UPDATE users SET azure_oid = ?, azure_upn = ?, auth_provider = 'azure_ad', full_name = ? WHERE id = ?",
        )
        .bind(azure_oid)
        .bind(azure_upn)
        .bind(full_name)
        .bind(existing.id)
        .execute(pool)
        .await?;
        return find_by_id(pool, existing.id).await.map(|u| u.unwrap());
    }

    // Create new user with a locked dummy password hash
    let result = sqlx::query(
        r#"INSERT INTO users (email, password_hash, full_name, role, azure_oid, azure_upn, auth_provider)
           VALUES (?, '__azure_sso__', ?, ?, ?, ?, 'azure_ad')"#,
    )
    .bind(email)
    .bind(full_name)
    .bind(role)
    .bind(azure_oid)
    .bind(azure_upn)
    .execute(pool)
    .await?;

    let id = result.last_insert_id() as i32;
    find_by_id(pool, id).await.map(|u| u.unwrap())
}

pub async fn find_by_azure_oid(
    pool: &MySqlPool,
    oid: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, full_name, department_id, role, manager_id, created_at, updated_at
         FROM users WHERE azure_oid = ?",
    )
    .bind(oid)
    .fetch_optional(pool)
    .await
}

/// Returns all users that were provisioned through Azure AD.
pub async fn list_azure_users(pool: &MySqlPool) -> Result<Vec<UserWithAzure>, sqlx::Error> {
    sqlx::query_as::<_, UserWithAzure>(
        r#"SELECT id, email, password_hash, full_name, department_id, role, manager_id,
                  created_at, updated_at, azure_oid, azure_upn, auth_provider
           FROM users
           WHERE auth_provider = 'azure_ad' AND azure_oid IS NOT NULL
           ORDER BY full_name"#,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_by_email(
    pool: &MySqlPool,
    email: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, full_name, department_id, role, manager_id, created_at, updated_at
        FROM users
        WHERE email = ?
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(
    pool: &MySqlPool,
    id: i32,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, full_name, department_id, role, manager_id, created_at, updated_at
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn list_by_manager(
    pool: &MySqlPool,
    manager_id: i32,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, full_name, department_id, role, manager_id, created_at, updated_at
        FROM users
        WHERE manager_id = ?
        ORDER BY full_name
        "#,
    )
    .bind(manager_id)
    .fetch_all(pool)
    .await
}

pub async fn list_by_department(
    pool: &MySqlPool,
    department_id: i32,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, full_name, department_id, role, manager_id, created_at, updated_at
        FROM users
        WHERE department_id = ?
        ORDER BY full_name
        "#,
    )
    .bind(department_id)
    .fetch_all(pool)
    .await
}

pub async fn list_all(pool: &MySqlPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, full_name, department_id, role, manager_id, created_at, updated_at
        FROM users
        ORDER BY full_name
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn create_user(
    pool: &MySqlPool,
    email: &str,
    password_hash: &str,
    full_name: &str,
    department_id: Option<i32>,
    role: &str,
    manager_id: Option<i32>,
) -> Result<User, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO users (email, password_hash, full_name, department_id, role, manager_id)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(full_name)
    .bind(department_id)
    .bind(role)
    .bind(manager_id)
    .execute(pool)
    .await?;

    let id = result.last_insert_id() as i32;
    find_by_id(pool, id).await.map(|u| u.unwrap())
}

pub async fn update_user(
    pool: &MySqlPool,
    id: i32,
    full_name: &str,
    department_id: Option<i32>,
    role: &str,
    manager_id: Option<i32>,
) -> Result<User, sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users
        SET full_name = ?, department_id = ?, role = ?, manager_id = ?
        WHERE id = ?
        "#,
    )
    .bind(full_name)
    .bind(department_id)
    .bind(role)
    .bind(manager_id)
    .bind(id)
    .execute(pool)
    .await?;

    find_by_id(pool, id).await.map(|u| u.unwrap())
}

pub async fn update_user_password(
    pool: &MySqlPool,
    id: i32,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users SET password_hash = ? WHERE id = ?
        "#,
    )
    .bind(password_hash)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_user(pool: &MySqlPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn set_password_reset_token(
    pool: &MySqlPool,
    user_id: i32,
    token: &str,
    expires_at: NaiveDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO password_reset_tokens (user_id, token, expires_at)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(token)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_by_reset_token(
    pool: &MySqlPool,
    token: &str,
) -> Result<Option<(User, PasswordResetToken)>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
            u.id, u.email, u.password_hash, u.full_name, u.department_id, u.role, u.manager_id, u.created_at, u.updated_at,
            t.id, t.user_id, t.token, t.expires_at, t.used, t.created_at
        FROM users u
        JOIN password_reset_tokens t ON t.user_id = u.id
        WHERE t.token = ? AND t.used = false AND t.expires_at > NOW()
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            use sqlx::Row;
            let user = User {
                id: row.get("id"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                full_name: row.get("full_name"),
                department_id: row.get("department_id"),
                role: row.get("role"),
                manager_id: row.get("manager_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            let reset_token = PasswordResetToken {
                id: row.get(9),
                user_id: row.get(10),
                token: row.get(11),
                expires_at: row.get(12),
                used: row.get(13),
                created_at: row.get(14),
            };
            Ok(Some((user, reset_token)))
        }
        None => Ok(None),
    }
}

pub async fn mark_token_used_and_update_password(
    pool: &MySqlPool,
    user_id: i32,
    token_id: i32,
    new_password_hash: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE password_reset_tokens SET used = true WHERE id = ?")
        .bind(token_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(new_password_hash)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await
}
