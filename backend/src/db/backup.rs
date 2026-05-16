use chrono::Local;
use std::fs;
use std::process::Command;
use sqlx::MySqlPool;

/// Backup the database referenced by DATABASE_URL into backups/backup_YYYY-MM-DD_HH-MM-SS.sql
///
/// database_url comes from Config::from_env().database_url (DATABASE_URL in .env)
pub async fn backup_database(
    _pool: &MySqlPool,
    database_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create backups directory if it doesn't exist
    fs::create_dir_all("backups")?;

    // Parse database URL to extract credentials
    let url_parts = parse_database_url(database_url)?;

    // Generate backup filename with timestamp
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let backup_file = format!("backups/backup_{}.sql", timestamp);

    // Build mysqldump command
    let mut cmd = Command::new("mysqldump");
    cmd.arg("-h").arg(&url_parts.host);
    cmd.arg("-P").arg(&url_parts.port);
    cmd.arg("-u").arg(&url_parts.user);

    // Use MYSQL_PWD environment variable instead of -p flag (more secure)
    cmd.env("MYSQL_PWD", &url_parts.password);

    cmd.arg(&url_parts.database);
    cmd.arg("--result-file").arg(&backup_file);
    cmd.arg("--single-transaction"); // Better for InnoDB
    cmd.arg("--quick");              // For large tables
    cmd.arg("--lock-tables=false");  // Don't lock tables

    // Run the command
    let output = cmd.output()?;

    if output.status.success() {
        Ok(backup_file)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let error_msg = format!(
            "mysqldump failed\nDatabase: {}\nHost: {}:{}\nUser: {}\nStderr: {}\nStdout: {}",
            url_parts.database, url_parts.host, url_parts.port, url_parts.user, stderr, stdout
        );

        Err(error_msg.into())
    }
}

pub async fn create_students_table(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let create_table_sql = r#"
        CREATE TABLE `STUDENTS` (
            `id` int NOT NULL AUTO_INCREMENT,
            `serial_number` int DEFAULT NULL,
            `registration_number` varchar(20) NOT NULL,
            `full_name` varchar(100) NOT NULL,
            `college` varchar(50) DEFAULT NULL,
            `course` varchar(50) DEFAULT NULL,
            `specialization` varchar(100) DEFAULT NULL,
            `academic_year` varchar(10) DEFAULT NULL,
            `email` varchar(100) DEFAULT NULL,

            -- GitHub and LinkedIn
            `github_username` varchar(100) DEFAULT NULL,
            `linkedin_url` varchar(255) DEFAULT NULL,

            -- LeetCode fields
            `leetcode_username` varchar(50) DEFAULT NULL,
            `leetcode_total_solved` int DEFAULT 0,
            `leetcode_solved_last_30_days` int DEFAULT 0,
            `has_leetcode_account` tinyint(1) DEFAULT 0,
            `leetcode_prev_month_solved` int DEFAULT 0,
            `leetcode_last_synced_at` datetime DEFAULT NULL,

            -- CodeChef fields
            `codechef_username` varchar(50) DEFAULT NULL,
            `codechef_total_solved` int DEFAULT 0,
            `codechef_solved_last_30_days` int DEFAULT 0,
            `has_codechef_account` tinyint(1) DEFAULT 0,
            `codechef_prev_month_solved` int DEFAULT 0,
            `codechef_last_synced_at` datetime DEFAULT NULL,

            -- Codeforces fields
            `codeforces_username` varchar(50) DEFAULT NULL,
            `codeforces_total_solved` int DEFAULT 0,
            `codeforces_solved_last_30_days` int DEFAULT 0,
            `has_codeforces_account` tinyint(1) DEFAULT 0,
            `codeforces_prev_month_solved` int DEFAULT 0,
            `codeforces_rating` int DEFAULT 0,
            `codeforces_max_rating` int DEFAULT 0,
            `codeforces_rank` varchar(50) DEFAULT NULL,
            `codeforces_last_synced_at` datetime DEFAULT NULL,

            -- Generated columns for totals (across all platforms)
            `total_platforms_solved` int GENERATED ALWAYS AS (
                COALESCE(`leetcode_total_solved`, 0) +
                COALESCE(`codechef_total_solved`, 0) +
                COALESCE(`codeforces_total_solved`, 0)
            ) STORED,
            `total_solved_last_30_days` int GENERATED ALWAYS AS (
                COALESCE(`leetcode_solved_last_30_days`, 0) +
                COALESCE(`codechef_solved_last_30_days`, 0) +
                COALESCE(`codeforces_solved_last_30_days`, 0)
            ) STORED,

            -- Metadata
            `updated_at` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            `created_at` datetime DEFAULT CURRENT_TIMESTAMP,

            -- Primary key
            PRIMARY KEY (`id`),

            -- Unique constraints
            UNIQUE KEY `registration_number` (`registration_number`),
            UNIQUE KEY `codechef_username` (`codechef_username`),
            UNIQUE KEY `codeforces_username` (`codeforces_username`),

            -- Indexes for common queries
            KEY `idx_serial_number` (`serial_number`),
            KEY `idx_college` (`college`),
            KEY `idx_course` (`course`),
            KEY `idx_specialization` (`specialization`),
            KEY `idx_academic_year` (`academic_year`),
            KEY `idx_has_leetcode_account` (`has_leetcode_account`),
            KEY `idx_has_codechef_account` (`has_codechef_account`),
            KEY `idx_has_codeforces_account` (`has_codeforces_account`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
    "#;

    sqlx::query(create_table_sql).execute(pool).await?;
    Ok(())
}

struct DatabaseUrlParts {
    user: String,
    password: String,
    host: String,
    port: String,
    database: String,
}

/// Parse URLs like:
/// mysql://srm:SRM@1234567890@localhost:3306/LeetCodeProfiles
fn parse_database_url(url: &str) -> Result<DatabaseUrlParts, Box<dyn std::error::Error>> {
    // Strip scheme
    let url = url
        .trim_start_matches("mysql://")
        .trim_start_matches("mariadb://");

    // Drop query params if present
    let url = url.split('?').next().unwrap_or(url);

    // Split at last '@' to handle '@' in password
    let at_pos = url
        .rfind('@')
        .ok_or("Invalid DATABASE_URL: missing '@' before host")?;

    let credentials_part = &url[..at_pos];
    let host_and_db = &url[at_pos + 1..];

    // credentials_part = user:password (password can contain @ and :)
    let colon_pos = credentials_part
        .find(':')
        .ok_or("Invalid DATABASE_URL: missing ':' between user and password")?;

    let user = credentials_part[..colon_pos].to_string();
    let password = credentials_part[colon_pos + 1..].to_string();

    // host_and_db = host:port/database
    let slash_pos = host_and_db
        .find('/')
        .ok_or("Invalid DATABASE_URL: missing '/' before database name")?;

    let host_port_part = &host_and_db[..slash_pos];
    let database = host_and_db[slash_pos + 1..].to_string();

    // host:port
    let (host_raw, port_raw) = if let Some(colon_pos) = host_port_part.find(':') {
        (
            &host_port_part[..colon_pos],
            &host_port_part[colon_pos + 1..],
        )
    } else {
        (host_port_part, "3306")
    };

    // Normalize localhost → 127.0.0.1
    let host = if host_raw.is_empty() || host_raw == "localhost" {
        "127.0.0.1".to_string()
    } else {
        host_raw.to_string()
    };

    let port = port_raw.to_string();

    Ok(DatabaseUrlParts {
        user,
        password,
        host,
        port,
        database,
    })
}