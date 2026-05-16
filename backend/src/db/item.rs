use super::models::{CreateItemRequest, Item, UpdateItemRequest};
use sqlx::MySqlPool;

pub async fn create_item(
    pool: &MySqlPool,
    user_id: i32,
    req: &CreateItemRequest,
) -> Result<Item, sqlx::Error> {
    let item = sqlx::query_as::<_, Item>(
        r#"
        INSERT INTO items (user_id, name, description)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(&req.name)
    .bind(&req.description)
    .fetch_one(pool)
    .await?;

    Ok(item)
}

pub async fn get_item_by_id(pool: &MySqlPool, id: i32) -> Result<Option<Item>, sqlx::Error> {
    let item = sqlx::query_as::<_, Item>(
        r#"
        SELECT id, user_id, name, description, created_at, updated_at
        FROM items
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(item)
}

pub async fn list_items(
    pool: &MySqlPool,
    user_id: Option<i32>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Item>, sqlx::Error> {
    let items = match user_id {
        Some(uid) => {
            sqlx::query_as::<_, Item>(
                r#"
                SELECT id, user_id, name, description, created_at, updated_at
                FROM items
                WHERE user_id = ?
                ORDER BY created_at DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(uid)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, Item>(
                r#"
                SELECT id, user_id, name, description, created_at, updated_at
                FROM items
                ORDER BY created_at DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
    };

    Ok(items)
}

pub async fn update_item(
    pool: &MySqlPool,
    id: i32,
    user_id: i32,
    req: &UpdateItemRequest,
) -> Result<Option<Item>, sqlx::Error> {
    // First check if item exists and belongs to user
    let existing = get_item_by_id(pool, id).await?;
    if let Some(item) = existing {
        if item.user_id != user_id {
            return Ok(None);
        }
    } else {
        return Ok(None);
    }

    let name = req.name.as_ref();
    let description = req.description.as_ref();

    let mut query = String::from("UPDATE items SET updated_at = NOW()");
    let mut has_update = false;

    if name.is_some() {
        query.push_str(", name = ?");
        has_update = true;
    }
    if description.is_some() {
        query.push_str(", description = ?");
        has_update = true;
    }

    if !has_update {
        return get_item_by_id(pool, id).await;
    }

    query.push_str(" WHERE id = ? AND user_id = ?");

    let mut q = sqlx::query(&query);
    
    if let Some(n) = name {
        q = q.bind(n);
    }
    if let Some(d) = description {
        q = q.bind(d);
    }
    
    q = q.bind(id).bind(user_id);
    q.execute(pool).await?;

    get_item_by_id(pool, id).await
}

pub async fn delete_item(pool: &MySqlPool, id: i32, user_id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM items WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
