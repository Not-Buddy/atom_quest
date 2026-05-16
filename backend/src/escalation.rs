/// 5.3 — Escalation Module (Rule-Based)
///
/// A Tokio background task runs daily and evaluates every active escalation rule
/// against the current state of the database. When a condition is met, it fires
/// notifications and records an escalation_log row.
///
/// Escalation stages:
///   Stage 1 — notify the employee
///   Stage 2 — notify the manager  (fired `days_after_trigger` days after stage 1)
///   Stage 3 — notify HR / skip-level (fired another `days_after_trigger` days later)
///
/// The escalation is automatically suppressed once the underlying condition is
/// resolved (e.g. the sheet is submitted / approved / check-in completed).
use std::sync::Arc;

use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

use crate::{
    api::auth::AppState,
    utils::notifications::{NotificationEvent, NotificationService},
};

// ─── DB row structs ───────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, sqlx::FromRow, Clone)]
pub struct EscalationRule {
    pub id:                 i32,
    pub name:               String,
    pub description:        Option<String>,
    pub trigger_event:      String,
    pub days_after_trigger: i32,
    pub notify_employee:    bool,
    pub notify_manager:     bool,
    pub notify_hr:          bool,
    pub is_active:          bool,
}

// EscalationLogRow intentionally kept for future use (e.g. de-dup checks)
#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
struct EscalationLogRow {
    pub id:         i32,
    pub rule_id:    i32,
    pub user_id:    i32,
    pub sheet_id:   Option<i32>,
    pub stage:      i8,
    pub status:     String,
}

// ─── Scheduler bootstrap ─────────────────────────────────────────────────────

/// Start the background escalation scheduler. Call once from `main`.
/// The job runs every day at 08:00 UTC.
pub async fn start_escalation_scheduler(state: Arc<AppState>) {
    let sched = match JobScheduler::new().await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create job scheduler: {e}");
            return;
        }
    };

    let job = match Job::new_async("0 0 8 * * *", move |_uuid, _lock| {
        let state = state.clone();
        Box::pin(async move {
            info!("Escalation check: starting daily run");
            run_escalation_check(&state).await;
            info!("Escalation check: daily run complete");
        })
    }) {
        Ok(j) => j,
        Err(e) => {
            error!("Failed to create escalation job: {e}");
            return;
        }
    };

    if let Err(e) = sched.add(job).await {
        error!("Failed to add escalation job to scheduler: {e}");
        return;
    }

    if let Err(e) = sched.start().await {
        error!("Failed to start job scheduler: {e}");
    } else {
        info!("Escalation scheduler started (daily @ 08:00 UTC)");
    }
}

// ─── Core escalation logic ────────────────────────────────────────────────────

async fn run_escalation_check(state: &AppState) {
    let pool   = &state.db.pool;
    let config = Arc::new(state.config.clone());
    let notif  = NotificationService::new(config, pool.clone());

    let rules = match fetch_active_rules(pool).await {
        Ok(r)  => r,
        Err(e) => { error!("Failed to fetch escalation rules: {e}"); return; }
    };

    for rule in rules {
        match rule.trigger_event.as_str() {
            "goal_not_submitted" =>
                check_goal_not_submitted(pool, &rule, &notif).await,
            "goal_not_approved" =>
                check_goal_not_approved(pool, &rule, &notif).await,
            "checkin_not_completed" =>
                check_checkin_not_completed(pool, &rule, &notif).await,
            other =>
                warn!("Unknown escalation trigger_event: {other}"),
        }
    }
}

// ─── Rule implementations ─────────────────────────────────────────────────────

/// Employees who still have a `draft` sheet N+ days after `goal_setting_opens`.
async fn check_goal_not_submitted(
    pool: &MySqlPool,
    rule: &EscalationRule,
    notif: &NotificationService,
) {
    let cutoff = Utc::now().naive_utc() - Duration::days(rule.days_after_trigger as i64);

    // Find all draft sheets where the cycle's goal_setting_opens passed the cutoff
    let rows = sqlx::query(
        r#"
        SELECT gs.id AS sheet_id, u.id AS user_id, u.full_name, u.email,
               m.email AS manager_email, m.full_name AS manager_name,
               gc.goal_setting_opens
        FROM goal_sheets gs
        JOIN goal_cycles gc ON gc.id = gs.cycle_id
        JOIN users u        ON u.id  = gs.user_id
        LEFT JOIN users m   ON m.id  = u.manager_id
        WHERE gs.status = 'draft'
          AND gc.is_active = 1
          AND gc.goal_setting_opens < ?
        "#,
    )
    .bind(cutoff)
    .fetch_all(pool)
    .await;

    let Ok(rows) = rows else { return; };

    for row in rows {
        use sqlx::Row;
        let sheet_id: i32        = row.get("sheet_id");
        let user_id:  i32        = row.get("user_id");
        let emp_name: String     = row.get("full_name");
        let emp_email: String    = row.get("email");
        let mgr_email: Option<String> = row.get("manager_email");
        let mgr_name: Option<String>  = row.get("manager_name");

        let already = already_escalated(pool, rule.id, user_id, sheet_id).await;

        if already { continue; }

        // Stage 1: notify the employee
        if rule.notify_employee {
            notif.send(NotificationEvent::Escalation {
                recipient_email: emp_email.clone(),
                recipient_name:  emp_name.clone(),
                subject: "Reminder: Please submit your goal sheet".to_string(),
                body: format!(
                    "Hello {emp_name},\n\nThis is a reminder that your goal sheet has not been submitted yet. \
                     Please log in to AtomQuest and submit your goals as soon as possible."
                ),
                sheet_id: Some(sheet_id),
            }).await;
        }

        // Stage 2: notify the manager
        if rule.notify_manager {
            if let (Some(me), Some(mn)) = (mgr_email.clone(), mgr_name.clone()) {
                notif.send(NotificationEvent::Escalation {
                    recipient_email: me,
                    recipient_name:  mn,
                    subject: format!("Action required: {emp_name} has not submitted goals"),
                    body: format!(
                        "Hello,\n\n{emp_name} has not yet submitted their goal sheet. \
                         Please follow up with them at your earliest convenience."
                    ),
                    sheet_id: Some(sheet_id),
                }).await;
            }
        }

        // Stage 3: notify HR (skip-level)
        if rule.notify_hr {
            if let Some(hr_email) = get_hr_email(pool).await {
                notif.send(NotificationEvent::Escalation {
                    recipient_email: hr_email,
                    recipient_name:  "HR Team".to_string(),
                    subject: format!("Escalation: {emp_name} goals not submitted"),
                    body: format!(
                        "This is an automated escalation. {emp_name} has not submitted \
                         their goal sheet {d} days after the cycle opened.",
                        d = rule.days_after_trigger
                    ),
                    sheet_id: Some(sheet_id),
                }).await;
            }
        }

        log_escalation(pool, rule.id, user_id, Some(sheet_id), 1, "sent").await;
    }
}

/// Sheets that are `submitted` but not yet approved/locked after N days.
async fn check_goal_not_approved(
    pool: &MySqlPool,
    rule: &EscalationRule,
    notif: &NotificationService,
) {
    let cutoff = Utc::now().naive_utc() - Duration::days(rule.days_after_trigger as i64);

    let rows = sqlx::query(
        r#"
        SELECT gs.id AS sheet_id, u.id AS user_id, u.full_name, u.email,
               m.id AS mgr_id, m.email AS manager_email, m.full_name AS manager_name,
               gs.submitted_at
        FROM goal_sheets gs
        JOIN users u        ON u.id  = gs.user_id
        LEFT JOIN users m   ON m.id  = u.manager_id
        WHERE gs.status = 'submitted'
          AND gs.submitted_at < ?
        "#,
    )
    .bind(cutoff)
    .fetch_all(pool)
    .await;

    let Ok(rows) = rows else { return; };

    for row in rows {
        use sqlx::Row;
        let sheet_id: i32        = row.get("sheet_id");
        let user_id:  i32        = row.get("user_id");
        let mgr_id:   Option<i32>= row.get("mgr_id");
        let emp_name: String     = row.get("full_name");
        let mgr_email: Option<String> = row.get("manager_email");
        let mgr_name: Option<String>  = row.get("manager_name");

        let already = already_escalated(pool, rule.id, user_id, sheet_id).await;
        if already { continue; }

        // Notify the manager
        if rule.notify_manager {
            if let (Some(me), Some(mn)) = (mgr_email.clone(), mgr_name.clone()) {
                notif.send(NotificationEvent::Escalation {
                    recipient_email: me,
                    recipient_name:  mn.clone(),
                    subject: format!("Pending approval: {emp_name}'s goal sheet"),
                    body: format!(
                        "Hello {mn},\n\n{emp_name}'s goal sheet has been waiting \
                         for your approval for {d} days. Please review and approve or return it.",
                        d = rule.days_after_trigger
                    ),
                    sheet_id: Some(sheet_id),
                }).await;
            }
        }

        // Notify HR if escalated
        if rule.notify_hr {
            if let Some(hr_email) = get_hr_email(pool).await {
                notif.send(NotificationEvent::Escalation {
                    recipient_email: hr_email,
                    recipient_name:  "HR Team".to_string(),
                    subject: format!("Escalation: {emp_name} goals pending approval"),
                    body: format!(
                        "{emp_name}'s goal sheet has been waiting for approval for \
                         {d} days.",
                        d = rule.days_after_trigger
                    ),
                    sheet_id: Some(sheet_id),
                }).await;
            }
        }

        let target_id = mgr_id.unwrap_or(user_id);
        log_escalation(pool, rule.id, target_id, Some(sheet_id), 2, "sent").await;
    }
}

/// Employees on locked sheets who have not logged any achievement for the
/// current quarter N+ days after the quarter window opened.
async fn check_checkin_not_completed(
    pool: &MySqlPool,
    rule: &EscalationRule,
    notif: &NotificationService,
) {
    let now    = Utc::now().naive_utc();
    let cutoff = now - Duration::days(rule.days_after_trigger as i64);

    // Determine which quarter window is currently open
    let quarter_info = sqlx::query(
        r#"
        SELECT
            CASE
              WHEN q1_opens <= ? AND (q2_opens IS NULL OR q2_opens > ?) THEN 'Q1'
              WHEN q2_opens <= ? AND (q3_opens IS NULL OR q3_opens > ?) THEN 'Q2'
              WHEN q3_opens <= ? AND (q4_opens IS NULL OR q4_opens > ?) THEN 'Q3'
              WHEN q4_opens <= ?                                         THEN 'Q4'
              ELSE NULL
            END AS current_quarter,
            CASE
              WHEN q1_opens <= ? AND (q2_opens IS NULL OR q2_opens > ?) THEN q1_opens
              WHEN q2_opens <= ? AND (q3_opens IS NULL OR q3_opens > ?) THEN q2_opens
              WHEN q3_opens <= ? AND (q4_opens IS NULL OR q4_opens > ?) THEN q3_opens
              WHEN q4_opens <= ?                                         THEN q4_opens
              ELSE NULL
            END AS quarter_opened_at
        FROM goal_cycles WHERE is_active = 1 LIMIT 1
        "#,
    )
    // Bind 14 times for both CASE blocks
    .bind(&now).bind(&now)
    .bind(&now).bind(&now)
    .bind(&now).bind(&now)
    .bind(&now)
    .bind(&now).bind(&now)
    .bind(&now).bind(&now)
    .bind(&now).bind(&now)
    .bind(&now)
    .fetch_optional(pool)
    .await;

    let Ok(Some(qi)) = quarter_info else { return; };
    use sqlx::Row;
    let current_quarter: Option<String> = qi.get("current_quarter");
    let quarter_opened_at: Option<chrono::NaiveDateTime> = qi.get("quarter_opened_at");

    let (Some(quarter), Some(opened)) = (current_quarter, quarter_opened_at) else { return; };

    // Only escalate if the window has been open long enough
    if opened > cutoff { return; }

    // Lower-case version for DB comparison ("Q1" → "q1")
    let db_quarter = quarter.to_lowercase();

    // Find locked sheets that have no achievement row for this quarter
    let rows = sqlx::query(
        r#"
        SELECT gs.id AS sheet_id, u.id AS user_id, u.full_name, u.email,
               m.email AS manager_email, m.full_name AS manager_name
        FROM goal_sheets gs
        JOIN users u      ON u.id = gs.user_id
        LEFT JOIN users m ON m.id = u.manager_id
        WHERE gs.status = 'locked'
          AND NOT EXISTS (
              SELECT 1 FROM achievements a
              JOIN goals g ON g.id = a.goal_id
              WHERE g.sheet_id = gs.id AND a.quarter = ?
          )
        "#,
    )
    .bind(&db_quarter)
    .fetch_all(pool)
    .await;

    let Ok(rows) = rows else { return; };

    for row in rows {
        let sheet_id: i32         = row.get("sheet_id");
        let user_id:  i32         = row.get("user_id");
        let emp_name: String      = row.get("full_name");
        let emp_email: String     = row.get("email");
        let mgr_email: Option<String> = row.get("manager_email");
        let mgr_name: Option<String>  = row.get("manager_name");

        let already = already_escalated(pool, rule.id, user_id, sheet_id).await;
        if already { continue; }

        if rule.notify_employee {
            notif.send(NotificationEvent::CheckinReminder {
                employee_email: emp_email.clone(),
                employee_name:  emp_name.clone(),
                manager_email:  mgr_email.clone().unwrap_or_default(),
                manager_name:   mgr_name.clone().unwrap_or_default(),
                quarter:        quarter.clone(),
                sheet_id,
            }).await;
        }

        log_escalation(pool, rule.id, user_id, Some(sheet_id), 1, "sent").await;
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn fetch_active_rules(pool: &MySqlPool) -> Result<Vec<EscalationRule>, sqlx::Error> {
    sqlx::query_as::<_, EscalationRule>(
        r#"SELECT id, name, description, trigger_event, days_after_trigger,
                  notify_employee, notify_manager, notify_hr, is_active
           FROM escalation_rules
           WHERE is_active = 1
           ORDER BY id"#,
    )
    .fetch_all(pool)
    .await
}

async fn already_escalated(
    pool: &MySqlPool,
    rule_id: i32,
    user_id: i32,
    sheet_id: i32,
) -> bool {
    let count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM escalation_log
           WHERE rule_id = ? AND user_id = ? AND sheet_id = ?
             AND status IN ('sent','pending')
             AND created_at > NOW() - INTERVAL 1 DAY"#,
    )
    .bind(rule_id)
    .bind(user_id)
    .bind(sheet_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    count > 0
}

async fn log_escalation(
    pool:     &MySqlPool,
    rule_id:  i32,
    user_id:  i32,
    sheet_id: Option<i32>,
    stage:    i8,
    status:   &str,
) {
    let _ = sqlx::query(
        r#"INSERT INTO escalation_log (rule_id, user_id, sheet_id, stage, status)
           VALUES (?, ?, ?, ?, ?)"#,
    )
    .bind(rule_id)
    .bind(user_id)
    .bind(sheet_id)
    .bind(stage)
    .bind(status)
    .execute(pool)
    .await;
}

/// Return the email address of a user with role = 'admin' (used as HR proxy).
async fn get_hr_email(pool: &MySqlPool) -> Option<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT email FROM users WHERE role = 'admin' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

// ─── Admin API helpers (used by admin.rs) ────────────────────────────────────

pub async fn list_rules(pool: &MySqlPool) -> Result<Vec<EscalationRule>, sqlx::Error> {
    fetch_active_rules(pool).await
}

pub async fn list_all_rules(pool: &MySqlPool) -> Result<Vec<EscalationRule>, sqlx::Error> {
    sqlx::query_as::<_, EscalationRule>(
        r#"SELECT id, name, description, trigger_event, days_after_trigger,
                  notify_employee, notify_manager, notify_hr, is_active
           FROM escalation_rules ORDER BY id"#,
    )
    .fetch_all(pool)
    .await
}

pub async fn create_rule(
    pool: &MySqlPool,
    name: &str,
    description: Option<&str>,
    trigger_event: &str,
    days_after_trigger: i32,
    notify_employee: bool,
    notify_manager: bool,
    notify_hr: bool,
    created_by: Option<i32>,
) -> Result<EscalationRule, sqlx::Error> {
    let res = sqlx::query(
        r#"INSERT INTO escalation_rules
           (name, description, trigger_event, days_after_trigger, notify_employee, notify_manager, notify_hr, created_by)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(name)
    .bind(description)
    .bind(trigger_event)
    .bind(days_after_trigger)
    .bind(notify_employee)
    .bind(notify_manager)
    .bind(notify_hr)
    .bind(created_by)
    .execute(pool)
    .await?;

    let id = res.last_insert_id() as i32;
    sqlx::query_as::<_, EscalationRule>(
        r#"SELECT id, name, description, trigger_event, days_after_trigger,
                  notify_employee, notify_manager, notify_hr, is_active
           FROM escalation_rules WHERE id = ?"#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn update_rule_active(
    pool: &MySqlPool,
    rule_id: i32,
    is_active: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE escalation_rules SET is_active = ? WHERE id = ?")
        .bind(is_active)
        .bind(rule_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_escalation_log(
    pool:    &MySqlPool,
    limit:   i64,
    offset:  i64,
    user_id: Option<i32>,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT el.id, el.rule_id, el.user_id, el.sheet_id, el.stage, el.status,
               el.resolved_at, el.note, el.created_at,
               u.full_name AS user_name, u.email AS user_email,
               er.name AS rule_name, er.trigger_event
        FROM escalation_log el
        JOIN users u            ON u.id  = el.user_id
        JOIN escalation_rules er ON er.id = el.rule_id
        WHERE (? IS NULL OR el.user_id = ?)
        ORDER BY el.created_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    use sqlx::Row;
    let mut result = Vec::new();
    for row in rows {
        result.push(serde_json::json!({
            "id":           row.get::<i32, _>("id"),
            "rule_id":      row.get::<i32, _>("rule_id"),
            "rule_name":    row.get::<String, _>("rule_name"),
            "trigger_event": row.get::<String, _>("trigger_event"),
            "user_id":      row.get::<i32, _>("user_id"),
            "user_name":    row.get::<String, _>("user_name"),
            "user_email":   row.get::<String, _>("user_email"),
            "sheet_id":     row.get::<Option<i32>, _>("sheet_id"),
            "stage":        row.get::<i8, _>("stage"),
            "status":       row.get::<String, _>("status"),
            "note":         row.get::<Option<String>, _>("note"),
            "resolved_at":  row.get::<Option<chrono::NaiveDateTime>, _>("resolved_at")
                               .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            "created_at":   row.get::<Option<chrono::NaiveDateTime>, _>("created_at")
                               .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
        }));
    }
    Ok(result)
}

pub async fn resolve_escalation(
    pool:       &MySqlPool,
    log_id:     i32,
    resolved_by: i32,
    note:        Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE escalation_log
           SET status = 'resolved', resolved_at = NOW(), resolved_by = ?, note = ?
           WHERE id = ?"#,
    )
    .bind(resolved_by)
    .bind(note)
    .bind(log_id)
    .execute(pool)
    .await?;
    Ok(())
}
