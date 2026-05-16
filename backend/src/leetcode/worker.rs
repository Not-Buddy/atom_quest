use crate::db::{self, DbPool};
use crate::leetcode::api;
use crate::logging::FileLogger;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use rand::Rng;
use chrono::{Utc, Duration};

pub struct BackgroundWorker {
    pool: Arc<DbPool>,
    rx: Receiver<String>,
    priority_rx: Receiver<String>,
    tx: Sender<String>,
    priority_tx: Sender<String>,
    logger: Arc<FileLogger>,
}

impl BackgroundWorker {
    pub fn new(
        pool: Arc<DbPool>,
        rx: Receiver<String>,
        priority_rx: Receiver<String>,
        tx: Sender<String>,
        priority_tx: Sender<String>,
        logger: Arc<FileLogger>,
    ) -> Self {
        Self {
            pool,
            rx,
            priority_rx,
            tx,
            priority_tx,
            logger,
        }
    }
    
    pub async fn run(self) {
        // Log that the background worker started
        let _ = self.logger.log("Background worker started");

        // Clone the senders to pass to the scheduler
        let scheduler_pool = self.pool.clone();
        let scheduler_tx = self.tx.clone();
        let priority_scheduler_tx = self.priority_tx.clone();
        let scheduler_logger = self.logger.clone();

        // Spawn the scheduler for periodic sync jobs
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(6 * 60 * 60)); // 6 hours
            loop {
                interval.tick().await;
                let _ = scheduler_logger.log("Running 6-hour LeetCode sync job");
                match db::get_all_students(&scheduler_pool).await {
                    Ok(students) => {
                        for student in students {
                            if student.leetcode_username.is_some() {
                                // Create a temporary queue just for sending the job
                                let temp_queue = SyncQueue {
                                    tx: scheduler_tx.clone(),
                                    priority_tx: priority_scheduler_tx.clone(),
                                    logger: scheduler_logger.clone(),
                                };
                                temp_queue.add_job(student.registration_number).await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = scheduler_logger.log_with_level("ERROR", &format!("Failed to get students for scheduler: {}", e));
                    }
                }
            }
        });

        // Now run the main loop with the original receivers
        let mut rx = self.rx;
        let mut priority_rx = self.priority_rx;
        let pool = self.pool;
        let logger = self.logger;

        loop {
            tokio::select! {
                Some(ra_number) = priority_rx.recv() => {
                    if !should_sync_student(&pool, &ra_number, &logger).await {
                        let _ = logger.log_with_level("INFO", &format!("Skipping LeetCode sync for {} (synced within last 6 hours)", ra_number));
                        let delay_secs = rand::thread_rng().gen_range(2..=10);
                        let _ = logger.log_with_level("INFO", &format!("Waiting {} seconds before next request", delay_secs));
                        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                        continue;
                    }

                    let _ = logger.log_with_level("INFO", &format!("Processing high priority sync job for {}", ra_number));
                    
                    match api::sync_single_student(&pool, &ra_number).await {
                        Ok(total_solved) => {
                            // Update the last_synced timestamp after successful sync
                            update_last_synced(&pool, &ra_number).await;

                            let student = match crate::db::get_student_by_ra(&pool, &ra_number).await {
                                Ok(student) => student,
                                Err(_) => {
                                    // If we can't get the student details, just log the total solved
                                    let _ = logger.log_with_level("INFO", &format!("Successfully synced LeetCode for student {}: {} total questions solved", ra_number, total_solved));
                                    let delay_secs = rand::thread_rng().gen_range(2..=10);
                                    let _ = logger.log_with_level("INFO", &format!("Waiting {} seconds before next request", delay_secs));
                                    tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                                    continue;
                                }
                            };

                            let _ = logger.log_with_level("INFO", &format!(
                                "Successfully synced LeetCode for student {}: {} total questions solved, {} in last 30 days",
                                ra_number,
                                total_solved,
                                student.leetcode_solved_last_30_days.unwrap_or(0)
                            ));

                            // Normal delay after successful sync
                            let delay_secs = rand::thread_rng().gen_range(2..=10);
                            let _ = logger.log_with_level("INFO", &format!("Waiting {} seconds before next request", delay_secs));
                            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                        },
                        Err(e) => {
                            let error_msg = e.to_string();
                            
                            // Check if error is a 429 rate limit error
                            if error_msg.contains("429") || error_msg.to_lowercase().contains("rate limit") || error_msg.to_lowercase().contains("too many requests") {
                                // Random cooldown between 20-25 minutes
                                let cooldown_mins = rand::thread_rng().gen_range(20..=25);
                                let cooldown_secs = cooldown_mins * 60;
                                
                                let _ = logger.log_with_level("WARNING", &format!("⚠️ Rate limit (429) detected for student {}", ra_number));
                                let _ = logger.log_with_level("WARNING", &format!("🛑 Pausing all LeetCode sync operations for {} minutes ({} seconds) due to rate limiting...", cooldown_mins, cooldown_secs));
                                
                                // Sleep for the random cooldown period
                                tokio::time::sleep(std::time::Duration::from_secs(cooldown_secs)).await;
                                
                                let _ = logger.log_with_level("INFO", &format!("✅ {}-minute rate limit cooldown completed. Resuming sync operations.", cooldown_mins));
                                
                                // After cooldown, wait normal delay before next request
                                let delay_secs = rand::thread_rng().gen_range(2..=10);
                                let _ = logger.log_with_level("INFO", &format!("Waiting {} seconds before next request", delay_secs));
                                tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                            } else {
                                let _ = logger.log_with_level("ERROR", &format!("Failed to sync student {}: {}", ra_number, error_msg));
                                
                                // Wait before next request even on error
                                let delay_secs = rand::thread_rng().gen_range(2..=10);
                                let _ = logger.log_with_level("INFO", &format!("Waiting {} seconds before next request", delay_secs));
                                tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                            }
                        }
                    }
                }

                Some(ra_number) = rx.recv() => {
                    if !should_sync_student(&pool, &ra_number, &logger).await {
                        let _ = logger.log_with_level("INFO", &format!("Skipping LeetCode sync for {} (synced within last 6 hours)", ra_number));
                        let delay_secs = rand::thread_rng().gen_range(2..=10);
                        let _ = logger.log_with_level("INFO", &format!("Waiting {} seconds before next request", delay_secs));
                        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                        continue;
                    }

                    let _ = logger.log_with_level("INFO", &format!("Processing sync job for {}", ra_number));
                    
                    match api::sync_single_student(&pool, &ra_number).await {
                        Ok(total_solved) => {
                            // Update the last_synced timestamp after successful sync
                            update_last_synced(&pool, &ra_number).await;

                            let student = match crate::db::get_student_by_ra(&pool, &ra_number).await {
                                Ok(student) => student,
                                Err(_) => {
                                    // If we can't get the student details, just log the total solved
                                    let _ = logger.log_with_level("INFO", &format!("Successfully synced LeetCode for student {}: {} total questions solved", ra_number, total_solved));
                                    let delay_secs = rand::thread_rng().gen_range(2..=10);
                                    let _ = logger.log_with_level("INFO", &format!("Waiting {} seconds before next request", delay_secs));
                                    tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                                    continue;
                                }
                            };

                            let _ = logger.log_with_level("INFO", &format!(
                                "Successfully synced LeetCode for student {}: {} total questions solved, {} in last 30 days",
                                ra_number,
                                total_solved,
                                student.leetcode_solved_last_30_days.unwrap_or(0)
                            ));

                            // Normal delay after successful sync
                            let delay_secs = rand::thread_rng().gen_range(2..=10);
                            let _ = logger.log_with_level("INFO", &format!("Waiting {} seconds before next request", delay_secs));
                            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                        },
                        Err(e) => {
                            let error_msg = e.to_string();
                            
                            // Check if error is a 429 rate limit error
                            if error_msg.contains("429") || error_msg.to_lowercase().contains("rate limit") || error_msg.to_lowercase().contains("too many requests") {
                                // Random cooldown between 20-25 minutes
                                let cooldown_mins = rand::thread_rng().gen_range(20..=25);
                                let cooldown_secs = cooldown_mins * 60;
                                
                                let _ = logger.log_with_level("WARNING", &format!("⚠️ Rate limit (429) detected for student {}", ra_number));
                                let _ = logger.log_with_level("WARNING", &format!("🛑 Pausing all LeetCode sync operations for {} minutes ({} seconds) due to rate limiting...", cooldown_mins, cooldown_secs));
                                
                                // Sleep for the random cooldown period
                                tokio::time::sleep(std::time::Duration::from_secs(cooldown_secs)).await;
                                
                                let _ = logger.log_with_level("INFO", &format!("✅ {}-minute rate limit cooldown completed. Resuming sync operations.", cooldown_mins));
                                
                                // After cooldown, wait normal delay before next request
                                let delay_secs = rand::thread_rng().gen_range(2..=10);
                                let _ = logger.log_with_level("INFO", &format!("Waiting {} seconds before next request", delay_secs));
                                tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                            } else {
                                let _ = logger.log_with_level("ERROR", &format!("Failed to sync student {}: {}", ra_number, error_msg));
                                
                                // Wait before next request even on error
                                let delay_secs = rand::thread_rng().gen_range(2..=10);
                                let _ = logger.log_with_level("INFO", &format!("Waiting {} seconds before next request", delay_secs));
                                tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                            }
                        }
                    }
                }

                else => {
                    let _ = logger.log("All sync job channels closed. Worker shutting down.");
                    break;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct SyncQueue {
    tx: Sender<String>,
    priority_tx: Sender<String>,
    logger: Arc<FileLogger>,
}

impl SyncQueue {
    pub fn new(tx: Sender<String>, priority_tx: Sender<String>, logger: Arc<FileLogger>) -> Self {
        Self { tx, priority_tx, logger }
    }

    pub async fn add_job(&self, ra_number: String) {
        if let Err(e) = self.tx.send(ra_number).await {
            let _ = self.logger.log_with_level("ERROR", &format!("Failed to add sync job to queue: {}", e));
        }
    }

    pub async fn add_priority_job(&self, ra_number: String) {
        if let Err(e) = self.priority_tx.send(ra_number).await {
            let _ = self.logger.log_with_level("ERROR", &format!("Failed to add priority sync job to queue: {}", e));
        }
    }
}

async fn should_sync_student(pool: &sqlx::MySqlPool, ra_number: &str, logger: &FileLogger) -> bool {
    // Query the student record with leetcode_last_synced_at
    let result = sqlx::query!(
        "SELECT leetcode_last_synced_at FROM STUDENTS WHERE registration_number = ?",
        ra_number
    )
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(record)) => {
            // Check if leetcode_last_synced_at is NULL (never synced)
            if let Some(last_synced) = record.leetcode_last_synced_at {
                let now = Utc::now();
                
                // Convert NaiveDateTime to DateTime<Utc>
                let last_synced_utc = last_synced.and_utc();
                let time_since_sync = now.signed_duration_since(last_synced_utc);
                
                // If synced within last 6 hours (21600 seconds), skip
                if time_since_sync < Duration::hours(6) {
                    let hours_ago = time_since_sync.num_hours();
                    let minutes_ago = time_since_sync.num_minutes() % 60;
                    let _ = logger.log_with_level("INFO", &format!(
                        "Student {} was synced {}h {}m ago (within 6-hour window)",
                        ra_number, hours_ago, minutes_ago
                    ));
                    return false; // Don't sync
                } else {
                    let hours_ago = time_since_sync.num_hours();
                    let _ = logger.log_with_level("INFO", &format!(
                        "Student {} last synced {}h ago (outside 6-hour window) - will sync",
                        ra_number, hours_ago
                    ));
                    return true; // Sync needed
                }
            } else {
                // Never synced before
                let _ = logger.log_with_level("INFO", &format!(
                    "Student {} has never been synced - will sync",
                    ra_number
                ));
                return true; // Sync needed
            }
        }
        Ok(None) => {
            // Student not found - shouldn't happen but allow sync to proceed
            let _ = logger.log_with_level("WARN", &format!(
                "Student {} not found in database - will attempt sync anyway",
                ra_number
            ));
            return true;
        }
        Err(e) => {
            // Database error - log and allow sync to proceed
            let _ = logger.log_with_level("ERROR", &format!(
                "Failed to check sync status for {}: {} - will attempt sync anyway",
                ra_number, e
            ));
            return true;
        }
    }
}

async fn update_last_synced(pool: &sqlx::MySqlPool, ra_number: &str) {
    use chrono::Utc;

    // Try to update leetcode_last_synced_at, but ignore errors in case the column doesn't exist yet
    let _ = sqlx::query("UPDATE STUDENTS SET leetcode_last_synced_at = ? WHERE registration_number = ?")
        .bind(Utc::now())
        .bind(ra_number)
        .execute(pool)
        .await;
}
