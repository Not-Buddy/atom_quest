-- AtomQuest: Goal Setting & Tracking Portal
-- Migration 001: Core Schema
-- Run against MySQL

-- Departments
CREATE TABLE IF NOT EXISTS `departments` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `name` VARCHAR(100) NOT NULL,
    `short_name` VARCHAR(20) NOT NULL,
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_name` (`name`),
    UNIQUE KEY `uk_short_name` (`short_name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Users (unified table: employees, managers, admins)
CREATE TABLE IF NOT EXISTS `users` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `email` VARCHAR(255) NOT NULL,
    `password_hash` VARCHAR(255) NOT NULL,
    `full_name` VARCHAR(150) NOT NULL,
    `department_id` INT DEFAULT NULL,
    `role` ENUM('employee', 'manager', 'admin') NOT NULL DEFAULT 'employee',
    `manager_id` INT DEFAULT NULL,
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_email` (`email`),
    KEY `idx_role` (`role`),
    KEY `idx_department_id` (`department_id`),
    KEY `idx_manager_id` (`manager_id`),
    CONSTRAINT `fk_users_department` FOREIGN KEY (`department_id`) REFERENCES `departments`(`id`) ON DELETE SET NULL,
    CONSTRAINT `fk_users_manager` FOREIGN KEY (`manager_id`) REFERENCES `users`(`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Thrust Areas (linked to departments)
CREATE TABLE IF NOT EXISTS `thrust_areas` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `name` VARCHAR(255) NOT NULL,
    `department_id` INT DEFAULT NULL,
    `created_by` INT DEFAULT NULL,
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_department_id` (`department_id`),
    KEY `idx_created_by` (`created_by`),
    CONSTRAINT `fk_thrust_areas_department` FOREIGN KEY (`department_id`) REFERENCES `departments`(`id`) ON DELETE SET NULL,
    CONSTRAINT `fk_thrust_areas_creator` FOREIGN KEY (`created_by`) REFERENCES `users`(`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Goal Cycles (e.g., "FY 2026-27")
CREATE TABLE IF NOT EXISTS `goal_cycles` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `name` VARCHAR(100) NOT NULL,
    `goal_setting_opens` DATETIME DEFAULT NULL,
    `q1_opens` DATETIME DEFAULT NULL,
    `q2_opens` DATETIME DEFAULT NULL,
    `q3_opens` DATETIME DEFAULT NULL,
    `q4_opens` DATETIME DEFAULT NULL,
    `is_active` TINYINT(1) DEFAULT 0,
    `created_by` INT DEFAULT NULL,
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_is_active` (`is_active`),
    CONSTRAINT `fk_goal_cycles_creator` FOREIGN KEY (`created_by`) REFERENCES `users`(`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Goal Sheets (one per employee per cycle)
CREATE TABLE IF NOT EXISTS `goal_sheets` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `user_id` INT NOT NULL,
    `cycle_id` INT NOT NULL,
    `status` ENUM('draft', 'submitted', 'approved', 'returned', 'locked') NOT NULL DEFAULT 'draft',
    `submitted_at` DATETIME DEFAULT NULL,
    `approved_at` DATETIME DEFAULT NULL,
    `approved_by` INT DEFAULT NULL,
    `returned_reason` TEXT DEFAULT NULL,
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_user_cycle` (`user_id`, `cycle_id`),
    KEY `idx_status` (`status`),
    KEY `idx_approved_by` (`approved_by`),
    CONSTRAINT `fk_goal_sheets_user` FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_goal_sheets_cycle` FOREIGN KEY (`cycle_id`) REFERENCES `goal_cycles`(`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_goal_sheets_approver` FOREIGN KEY (`approved_by`) REFERENCES `users`(`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Goals (individual goals within a sheet)
CREATE TABLE IF NOT EXISTS `goals` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `sheet_id` INT NOT NULL,
    `thrust_area_id` INT DEFAULT NULL,
    `title` VARCHAR(255) NOT NULL,
    `description` TEXT DEFAULT NULL,
    `uom_type` ENUM('min_numeric', 'max_numeric', 'min_percent', 'max_percent', 'timeline', 'zero') NOT NULL,
    `target_value` DOUBLE NOT NULL DEFAULT 0,
    `target_date` DATE DEFAULT NULL,
    `weightage` DOUBLE NOT NULL,
    `is_shared` TINYINT(1) DEFAULT 0,
    `shared_from_goal_id` INT DEFAULT NULL,
    `sort_order` INT DEFAULT 0,
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_sheet_id` (`sheet_id`),
    KEY `idx_shared_from` (`shared_from_goal_id`),
    CONSTRAINT `fk_goals_sheet` FOREIGN KEY (`sheet_id`) REFERENCES `goal_sheets`(`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_goals_thrust_area` FOREIGN KEY (`thrust_area_id`) REFERENCES `thrust_areas`(`id`) ON DELETE SET NULL,
    CONSTRAINT `fk_goals_shared_from` FOREIGN KEY (`shared_from_goal_id`) REFERENCES `goals`(`id`) ON DELETE SET NULL,
    CONSTRAINT `chk_weightage` CHECK (`weightage` >= 10 AND `weightage` <= 100)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Achievements (quarterly actuals per goal)
CREATE TABLE IF NOT EXISTS `achievements` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `goal_id` INT NOT NULL,
    `quarter` ENUM('q1', 'q2', 'q3', 'q4') NOT NULL,
    `actual_value` DOUBLE DEFAULT NULL,
    `actual_date` DATE DEFAULT NULL,
    `status` ENUM('not_started', 'on_track', 'completed') NOT NULL DEFAULT 'not_started',
    `computed_score` DOUBLE DEFAULT NULL,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_goal_quarter` (`goal_id`, `quarter`),
    CONSTRAINT `fk_achievements_goal` FOREIGN KEY (`goal_id`) REFERENCES `goals`(`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Check-in Comments (manager quarterly comments)
CREATE TABLE IF NOT EXISTS `checkin_comments` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `goal_sheet_id` INT NOT NULL,
    `quarter` ENUM('q1', 'q2', 'q3', 'q4') NOT NULL,
    `manager_id` INT NOT NULL,
    `comment` TEXT NOT NULL,
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_sheet_quarter` (`goal_sheet_id`, `quarter`),
    CONSTRAINT `fk_checkin_comments_sheet` FOREIGN KEY (`goal_sheet_id`) REFERENCES `goal_sheets`(`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_checkin_comments_manager` FOREIGN KEY (`manager_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Audit Log (tracks all data changes post-lock)
CREATE TABLE IF NOT EXISTS `audit_log` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `table_name` VARCHAR(100) NOT NULL,
    `record_id` INT NOT NULL,
    `field_name` VARCHAR(100) DEFAULT NULL,
    `old_value` TEXT DEFAULT NULL,
    `new_value` TEXT DEFAULT NULL,
    `changed_by` INT DEFAULT NULL,
    `changed_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_table_record` (`table_name`, `record_id`),
    KEY `idx_changed_by` (`changed_by`),
    CONSTRAINT `fk_audit_log_user` FOREIGN KEY (`changed_by`) REFERENCES `users`(`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Password Reset Tokens
CREATE TABLE IF NOT EXISTS `password_reset_tokens` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `user_id` INT NOT NULL,
    `token` VARCHAR(255) NOT NULL,
    `expires_at` DATETIME NOT NULL,
    `used` TINYINT(1) DEFAULT 0,
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_token` (`token`),
    KEY `idx_user_id` (`user_id`),
    CONSTRAINT `fk_password_reset_tokens_user` FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
