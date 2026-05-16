-- AtomQuest: Goal Setting & Tracking Portal
-- Migration 002: Bonus Features Schema
-- Azure AD SSO, Notifications, Escalations, Analytics

-- ============================================================
-- 5.1 Azure AD / Entra ID SSO
-- ============================================================

-- Store Azure AD identity linked to a user
ALTER TABLE `users` ADD COLUMN `azure_oid`        VARCHAR(36)  DEFAULT NULL COMMENT 'Azure AD Object ID';
ALTER TABLE `users` ADD COLUMN `azure_upn`        VARCHAR(255) DEFAULT NULL COMMENT 'Azure AD UPN (user principal name)';
ALTER TABLE `users` ADD COLUMN `auth_provider`    ENUM('local','azure_ad') NOT NULL DEFAULT 'local';
ALTER TABLE `users` ADD UNIQUE KEY `uk_azure_oid` (`azure_oid`);

-- ============================================================
-- 5.2 Notification Preferences & Log
-- ============================================================

CREATE TABLE IF NOT EXISTS `notification_preferences` (
    `id`                INT NOT NULL AUTO_INCREMENT,
    `user_id`           INT NOT NULL,
    `email_enabled`     TINYINT(1) NOT NULL DEFAULT 1,
    `teams_enabled`     TINYINT(1) NOT NULL DEFAULT 0,
    `teams_webhook_url` VARCHAR(500) DEFAULT NULL,
    `updated_at`        DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_user_prefs` (`user_id`),
    CONSTRAINT `fk_notif_prefs_user` FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `notification_log` (
    `id`            INT NOT NULL AUTO_INCREMENT,
    `user_id`       INT DEFAULT NULL,
    `event_type`    VARCHAR(100) NOT NULL COMMENT 'e.g. goal_submitted, goal_approved, checkin_reminder',
    `channel`       ENUM('email','teams') NOT NULL DEFAULT 'email',
    `recipient`     VARCHAR(255) NOT NULL COMMENT 'email address or Teams webhook URL',
    `subject`       VARCHAR(500) DEFAULT NULL,
    `body_snippet`  TEXT DEFAULT NULL,
    `status`        ENUM('sent','failed','skipped') NOT NULL DEFAULT 'sent',
    `error_message` TEXT DEFAULT NULL,
    `sent_at`       DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_user_event` (`user_id`, `event_type`),
    KEY `idx_sent_at` (`sent_at`),
    CONSTRAINT `fk_notif_log_user` FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- ============================================================
-- 5.3 Escalation Module
-- ============================================================

CREATE TABLE IF NOT EXISTS `escalation_rules` (
    `id`                INT NOT NULL AUTO_INCREMENT,
    `name`              VARCHAR(200) NOT NULL,
    `description`       TEXT DEFAULT NULL,
    `trigger_event`     ENUM(
        'goal_not_submitted',
        'goal_not_approved',
        'checkin_not_completed'
    ) NOT NULL,
    `days_after_trigger` INT NOT NULL DEFAULT 3 COMMENT 'Days after the triggering event before escalation fires',
    `notify_employee`   TINYINT(1) NOT NULL DEFAULT 1,
    `notify_manager`    TINYINT(1) NOT NULL DEFAULT 1,
    `notify_hr`         TINYINT(1) NOT NULL DEFAULT 0 COMMENT 'Skip-level / HR notification on 2nd escalation',
    `is_active`         TINYINT(1) NOT NULL DEFAULT 1,
    `created_by`        INT DEFAULT NULL,
    `created_at`        DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at`        DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    CONSTRAINT `fk_esc_rules_creator` FOREIGN KEY (`created_by`) REFERENCES `users`(`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `escalation_log` (
    `id`            INT NOT NULL AUTO_INCREMENT,
    `rule_id`       INT NOT NULL,
    `user_id`       INT NOT NULL COMMENT 'The employee being escalated',
    `sheet_id`      INT DEFAULT NULL,
    `stage`         TINYINT NOT NULL DEFAULT 1 COMMENT '1=employee, 2=manager, 3=HR',
    `status`        ENUM('pending','sent','resolved','suppressed') NOT NULL DEFAULT 'pending',
    `resolved_at`   DATETIME DEFAULT NULL,
    `resolved_by`   INT DEFAULT NULL,
    `note`          TEXT DEFAULT NULL,
    `created_at`    DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at`    DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_rule_user`   (`rule_id`, `user_id`),
    KEY `idx_status`      (`status`),
    KEY `idx_created_at`  (`created_at`),
    CONSTRAINT `fk_esc_log_rule`    FOREIGN KEY (`rule_id`)     REFERENCES `escalation_rules`(`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_esc_log_user`    FOREIGN KEY (`user_id`)     REFERENCES `users`(`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_esc_log_sheet`   FOREIGN KEY (`sheet_id`)    REFERENCES `goal_sheets`(`id`) ON DELETE SET NULL,
    CONSTRAINT `fk_esc_log_resolver` FOREIGN KEY (`resolved_by`) REFERENCES `users`(`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
