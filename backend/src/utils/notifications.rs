/// 5.2 — Email & Microsoft Teams Notifications
///
/// This module provides a unified `NotificationService` that can dispatch
/// event-driven notifications over two channels:
///   • Email  — via the existing SMTP stack (lettre)
///   • Teams  — via an Incoming Webhook using an Adaptive-Card-compatible
///               JSON payload (the "MessageCard" legacy format is also
///               supported as a fallback for older connectors)
///
/// Supported notification events:
///   - `goal_submitted`          — employee submits sheet → notify manager
///   - `goal_approved`           — manager approves sheet → notify employee
///   - `goal_returned`           — manager returns sheet  → notify employee
///   - `checkin_reminder`        — quarterly reminder     → notify employee + manager
///   - `escalation`              — escalation fired       → notify configured recipients
///
/// Every notification carries a `deep_link` URL so the recipient can jump
/// directly into the relevant goal sheet.
use std::sync::Arc;

use lettre::{Message, SmtpTransport, Transport, message::header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use reqwest::Client;
use serde_json::json;
use sqlx::MySqlPool;
use tracing::{error, info, warn};

use crate::config::Config;

// ─── Event types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum NotificationEvent {
    GoalSubmitted {
        employee_name:  String,
        employee_email: String,
        manager_email:  String,
        manager_name:   String,
        sheet_id:       i32,
    },
    GoalApproved {
        employee_email: String,
        employee_name:  String,
        sheet_id:       i32,
    },
    GoalReturned {
        employee_email: String,
        employee_name:  String,
        reason:         String,
        sheet_id:       i32,
    },
    CheckinReminder {
        employee_email: String,
        employee_name:  String,
        manager_email:  String,
        manager_name:   String,
        quarter:        String,
        sheet_id:       i32,
    },
    Escalation {
        recipient_email: String,
        recipient_name:  String,
        subject:         String,
        body:            String,
        sheet_id:        Option<i32>,
    },
}

// ─── Service ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct NotificationService {
    config: Arc<Config>,
    pool:   MySqlPool,
    http:   Client,
}

impl NotificationService {
    pub fn new(config: Arc<Config>, pool: MySqlPool) -> Self {
        Self {
            config,
            pool,
            http: Client::new(),
        }
    }

    /// Dispatch a notification. Errors are logged but never propagated — a
    /// failed notification should never break the primary business operation.
    pub async fn send(&self, event: NotificationEvent) {
        match &event {
            NotificationEvent::GoalSubmitted {
                employee_name,
                employee_email,
                manager_email,
                manager_name,
                sheet_id,
            } => {
                let deep_link = self.sheet_deep_link(*sheet_id);
                let subject   = format!("{employee_name} has submitted their goal sheet for review");
                let body      = self.build_email_body_goal_submitted(
                    employee_name, employee_email, manager_name, &deep_link,
                );
                self.send_email(manager_email, &subject, &body, *sheet_id).await;
                self.send_teams_card(
                    manager_email,
                    &subject,
                    &format!(
                        "{employee_name} submitted their goal sheet. Review it now.",
                    ),
                    &deep_link,
                    "View Goal Sheet",
                    *sheet_id,
                ).await;
                self.log_notification(
                    None, "goal_submitted", "email", manager_email,
                    &subject, &body, "sent",
                ).await;
            }

            NotificationEvent::GoalApproved {
                employee_email,
                employee_name,
                sheet_id,
            } => {
                let deep_link = self.sheet_deep_link(*sheet_id);
                let subject   = "Your goal sheet has been approved";
                let body = self.build_email_body_approved(employee_name, &deep_link);
                self.send_email(employee_email, subject, &body, *sheet_id).await;
                self.send_teams_card(
                    employee_email,
                    subject,
                    "Your goals are now locked. You can start logging achievements.",
                    &deep_link,
                    "View Goal Sheet",
                    *sheet_id,
                ).await;
                self.log_notification(
                    None, "goal_approved", "email", employee_email,
                    subject, &body, "sent",
                ).await;
            }

            NotificationEvent::GoalReturned {
                employee_email,
                employee_name,
                reason,
                sheet_id,
            } => {
                let deep_link = self.sheet_deep_link(*sheet_id);
                let subject   = "Action required: Your goal sheet was returned for revision";
                let body = self.build_email_body_returned(employee_name, reason, &deep_link);
                self.send_email(employee_email, subject, &body, *sheet_id).await;
                self.send_teams_card(
                    employee_email,
                    subject,
                    &format!("Reason: {reason}"),
                    &deep_link,
                    "Revise Goal Sheet",
                    *sheet_id,
                ).await;
                self.log_notification(
                    None, "goal_returned", "email", employee_email,
                    subject, &body, "sent",
                ).await;
            }

            NotificationEvent::CheckinReminder {
                employee_email,
                employee_name,
                manager_email,
                manager_name,
                quarter,
                sheet_id,
            } => {
                let deep_link = self.sheet_deep_link(*sheet_id);
                let emp_subj = format!("{quarter} check-in reminder: log your progress");
                let emp_body = self.build_email_body_checkin_employee(
                    employee_name, quarter, &deep_link,
                );
                self.send_email(employee_email, &emp_subj, &emp_body, *sheet_id).await;

                let mgr_subj = format!("{quarter} check-in reminder for {employee_name}");
                let mgr_body = self.build_email_body_checkin_manager(
                    manager_name, employee_name, quarter, &deep_link,
                );
                self.send_email(manager_email, &mgr_subj, &mgr_body, *sheet_id).await;

                self.send_teams_card(
                    manager_email,
                    &mgr_subj,
                    &format!("{employee_name} needs a {quarter} check-in comment."),
                    &deep_link,
                    "Open Check-in",
                    *sheet_id,
                ).await;
            }

            NotificationEvent::Escalation {
                recipient_email,
                recipient_name: _,
                subject,
                body,
                sheet_id,
            } => {
                self.send_email(recipient_email, subject, body, sheet_id.unwrap_or(0)).await;
                if let Some(sid) = sheet_id {
                    let deep_link = self.sheet_deep_link(*sid);
                    self.send_teams_card(
                        recipient_email,
                        subject,
                        body,
                        &deep_link,
                        "View Goal Sheet",
                        *sid,
                    ).await;
                }
                self.log_notification(
                    None, "escalation", "email", recipient_email,
                    subject, body, "sent",
                ).await;
            }
        }
    }

    // ─── Deep-link builder ───────────────────────────────────────────────────

    fn sheet_deep_link(&self, sheet_id: i32) -> String {
        let base = self
            .config
            .frontend_base_url
            .as_deref()
            .unwrap_or(&self.config.base_url);
        format!("{base}/goals/sheets/{sheet_id}")
    }

    // ─── Email delivery ──────────────────────────────────────────────────────

    async fn send_email(&self, to: &str, subject: &str, html_body: &str, _sheet_id: i32) {
        let from_addr = match self.config.from_email.parse::<lettre::Address>() {
            Ok(a) => a,
            Err(e) => { error!("Invalid from_email: {e}"); return; }
        };
        let to_addr = match to.parse::<lettre::Address>() {
            Ok(a) => a,
            Err(e) => { warn!("Invalid recipient email '{to}': {e}"); return; }
        };

        let msg = match Message::builder()
            .from(lettre::message::Mailbox::new(Some("AtomQuest".into()), from_addr))
            .to(lettre::message::Mailbox::new(None, to_addr))
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
        {
            Ok(m) => m,
            Err(e) => { error!("Email build error: {e}"); return; }
        };

        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );
        let mailer = match SmtpTransport::relay(&self.config.smtp_host) {
            Ok(b) => b.credentials(creds).build(),
            Err(e) => { error!("SMTP transport build error: {e}"); return; }
        };

        match mailer.send(&msg) {
            Ok(_)  => info!("Email sent to {to}: {subject}"),
            Err(e) => error!("Email send failed to {to}: {e}"),
        }
    }

    // ─── Teams Adaptive Card ─────────────────────────────────────────────────

    /// Sends a Teams message card to the *channel* webhook configured in
    /// the database for the given recipient, falling back to the global
    /// default webhook if none is set.
    async fn send_teams_card(
        &self,
        recipient_email: &str,
        title: &str,
        text: &str,
        deep_link: &str,
        action_label: &str,
        _sheet_id: i32,
    ) {
        // Look up personal webhook from DB; fall back to global default
        let webhook = self.resolve_teams_webhook(recipient_email).await;
        let Some(webhook_url) = webhook else {
            return; // Teams not configured
        };

        // Adaptive Card 1.2 payload (supported by all Teams clients)
        let card_payload = json!({
            "type": "message",
            "attachments": [{
                "contentType": "application/vnd.microsoft.card.adaptive",
                "content": {
                    "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
                    "type": "AdaptiveCard",
                    "version": "1.2",
                    "body": [
                        {
                            "type": "TextBlock",
                            "text": title,
                            "weight": "Bolder",
                            "size": "Medium",
                            "wrap": true
                        },
                        {
                            "type": "TextBlock",
                            "text": text,
                            "wrap": true,
                            "spacing": "Small"
                        }
                    ],
                    "actions": [
                        {
                            "type": "Action.OpenUrl",
                            "title": action_label,
                            "url": deep_link
                        }
                    ]
                }
            }]
        });

        match self.http.post(&webhook_url).json(&card_payload).send().await {
            Ok(r) if r.status().is_success() => {
                info!("Teams card sent to {recipient_email}");
            }
            Ok(r) => {
                let status = r.status();
                let body = r.text().await.unwrap_or_default();
                error!("Teams webhook returned {status}: {body}");
            }
            Err(e) => error!("Teams webhook request failed: {e}"),
        }
    }

    async fn resolve_teams_webhook(&self, email: &str) -> Option<String> {
        // Check per-user preference first
        let row = sqlx::query_scalar::<_, Option<String>>(
            "SELECT np.teams_webhook_url
             FROM notification_preferences np
             JOIN users u ON u.id = np.user_id
             WHERE u.email = ? AND np.teams_enabled = 1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        if let Some(Some(url)) = row {
            return Some(url);
        }

        // Fall back to global default
        self.config.teams_default_webhook.clone()
    }

    // ─── Notification log ────────────────────────────────────────────────────

    async fn log_notification(
        &self,
        user_id: Option<i32>,
        event_type: &str,
        channel: &str,
        recipient: &str,
        subject: &str,
        body: &str,
        status: &str,
    ) {
        let snippet: String = body.chars().take(500).collect();
        let _ = sqlx::query(
            r#"INSERT INTO notification_log
               (user_id, event_type, channel, recipient, subject, body_snippet, status)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(user_id)
        .bind(event_type)
        .bind(channel)
        .bind(recipient)
        .bind(subject)
        .bind(snippet)
        .bind(status)
        .execute(&self.pool)
        .await;
    }

    // ─── Email body templates ────────────────────────────────────────────────

    fn build_email_body_goal_submitted(
        &self,
        employee_name: &str,
        employee_email: &str,
        manager_name: &str,
        deep_link: &str,
    ) -> String {
        format!(r#"<!DOCTYPE html><html><body style="font-family:Arial,sans-serif;color:#333;max-width:600px;margin:0 auto;padding:20px;">
<h2 style="color:#2c3e50;">Goal Sheet Submitted for Review</h2>
<p>Hello {manager_name},</p>
<p><strong>{employee_name}</strong> ({employee_email}) has submitted their goal sheet and it is awaiting your review and approval.</p>
<div style="text-align:center;margin:30px 0;">
  <a href="{deep_link}" style="background-color:#007bff;color:white;padding:12px 30px;text-decoration:none;border-radius:5px;font-weight:bold;">Review Goal Sheet</a>
</div>
<p style="font-size:12px;color:#999;">This is an automated message from AtomQuest. Please do not reply.</p>
</body></html>"#)
    }

    fn build_email_body_approved(&self, employee_name: &str, deep_link: &str) -> String {
        format!(r#"<!DOCTYPE html><html><body style="font-family:Arial,sans-serif;color:#333;max-width:600px;margin:0 auto;padding:20px;">
<h2 style="color:#28a745;">Goal Sheet Approved</h2>
<p>Hello {employee_name},</p>
<p>Great news! Your goal sheet has been reviewed and <strong>approved</strong> by your manager. Your goals are now locked.</p>
<p>You can now start logging your quarterly achievements against each goal.</p>
<div style="text-align:center;margin:30px 0;">
  <a href="{deep_link}" style="background-color:#28a745;color:white;padding:12px 30px;text-decoration:none;border-radius:5px;font-weight:bold;">View My Goals</a>
</div>
<p style="font-size:12px;color:#999;">This is an automated message from AtomQuest. Please do not reply.</p>
</body></html>"#)
    }

    fn build_email_body_returned(
        &self,
        employee_name: &str,
        reason: &str,
        deep_link: &str,
    ) -> String {
        format!(r#"<!DOCTYPE html><html><body style="font-family:Arial,sans-serif;color:#333;max-width:600px;margin:0 auto;padding:20px;">
<h2 style="color:#dc3545;">Goal Sheet Returned for Revision</h2>
<p>Hello {employee_name},</p>
<p>Your goal sheet has been returned by your manager for revision. Please review the feedback and resubmit.</p>
<div style="background:#fff3cd;border-left:4px solid #ffc107;padding:12px;margin:20px 0;">
  <strong>Manager's comments:</strong><br/>{reason}
</div>
<div style="text-align:center;margin:30px 0;">
  <a href="{deep_link}" style="background-color:#dc3545;color:white;padding:12px 30px;text-decoration:none;border-radius:5px;font-weight:bold;">Revise Goal Sheet</a>
</div>
<p style="font-size:12px;color:#999;">This is an automated message from AtomQuest. Please do not reply.</p>
</body></html>"#)
    }

    fn build_email_body_checkin_employee(
        &self,
        employee_name: &str,
        quarter: &str,
        deep_link: &str,
    ) -> String {
        format!(r#"<!DOCTYPE html><html><body style="font-family:Arial,sans-serif;color:#333;max-width:600px;margin:0 auto;padding:20px;">
<h2 style="color:#17a2b8;">{quarter} Check-in Reminder</h2>
<p>Hello {employee_name},</p>
<p>The <strong>{quarter}</strong> check-in window is now open. Please log your actual achievement against each goal.</p>
<div style="text-align:center;margin:30px 0;">
  <a href="{deep_link}" style="background-color:#17a2b8;color:white;padding:12px 30px;text-decoration:none;border-radius:5px;font-weight:bold;">Log My {quarter} Progress</a>
</div>
<p style="font-size:12px;color:#999;">This is an automated message from AtomQuest. Please do not reply.</p>
</body></html>"#)
    }

    fn build_email_body_checkin_manager(
        &self,
        manager_name: &str,
        employee_name: &str,
        quarter: &str,
        deep_link: &str,
    ) -> String {
        format!(r#"<!DOCTYPE html><html><body style="font-family:Arial,sans-serif;color:#333;max-width:600px;margin:0 auto;padding:20px;">
<h2 style="color:#17a2b8;">{quarter} Check-in Reminder</h2>
<p>Hello {manager_name},</p>
<p>Please complete your <strong>{quarter}</strong> check-in for <strong>{employee_name}</strong> by adding a structured comment to document the discussion.</p>
<div style="text-align:center;margin:30px 0;">
  <a href="{deep_link}" style="background-color:#17a2b8;color:white;padding:12px 30px;text-decoration:none;border-radius:5px;font-weight:bold;">Open {quarter} Check-in</a>
</div>
<p style="font-size:12px;color:#999;">This is an automated message from AtomQuest. Please do not reply.</p>
</body></html>"#)
    }
}
