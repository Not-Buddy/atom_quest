use super::models::AuditLogEntry;
use sqlx::MySqlPool;

pub async fn log_change(
    pool: &MySqlPool,
    table_name: &str,
    record_id: i32,
    field_name: Option<&str>,
    old_value: Option<&str>,
    new_value: Option<&str>,
    changed_by: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (table_name, record_id, field_name, old_value, new_value, changed_by)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(table_name)
    .bind(record_id)
    .bind(field_name)
    .bind(old_value)
    .bind(new_value)
    .bind(changed_by)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_audit_logs(
    pool: &MySqlPool,
    table_name: Option<&str>,
    record_id: Option<i32>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
    let mut base = String::from(
        "SELECT id, table_name, record_id, field_name, old_value, new_value, changed_by, changed_at FROM audit_log WHERE 1=1",
    );

    if table_name.is_some() {
        base.push_str(" AND table_name = ?");
    }
    if record_id.is_some() {
        base.push_str(" AND record_id = ?");
    }
    base.push_str(" ORDER BY changed_at DESC LIMIT ? OFFSET ?");

    let mut query = sqlx::query_as::<_, AuditLogEntry>(&base);

    if let Some(t) = table_name {
        query = query.bind(t);
    }
    if let Some(r) = record_id {
        query = query.bind(r);
    }
    query = query.bind(limit).bind(offset);

    query.fetch_all(pool).await
}
