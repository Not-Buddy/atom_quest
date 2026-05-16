use crate::db::{self, DbPool};
use crate::codechef::api::{self, CodeChefClient};
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
    codechef_client: Arc<CodeChefClient>,
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
            codechef_client: Arc::new(CodeChefClient::new()),
        }
    }

    pub async fn run(self) {
        let _ = self.logger.log("CodeChef background worker started");

        let scheduler_pool = self.pool.clone();
        let scheduler_tx = self.tx.clone();
        let priority_scheduler_tx = self.priority_tx.clone();
        let scheduler_logger = self.logger.clone();

        // Spawn scheduler for periodic sync (6 hours)
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(6 * 60 * 60));
            loop {
                interval.tick().await;
                let _ = scheduler_logger.log("Running 6-hour CodeChef sync job");
                
                match db::get_all_students(&scheduler_pool).await {
                    Ok(students) => {
                        for student in students {
                            if student.codechef_username.is_some() {
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
                        let _ = scheduler_logger.log_with_level("ERROR", &format!("Failed to get students for CodeChef scheduler: {}", e));
                    }
                }
            }
        });

        let mut rx = self.rx;
        let mut priority_rx = self.priority_rx;
        let pool = self.pool;
        let logger = self.logger;
        let codechef_client = self.codechef_client;
        let tx = self.tx;
        let priority_tx = self.priority_tx;

        loop {
            tokio::select! {
                Some(ra_number) = priority_rx.recv() => {
                    process_sync_job(
                        &pool,
                        &logger,
                        &codechef_client,
                        &tx,
                        &priority_tx,
                        ra_number,
                        true
                    ).await;
                }

                Some(ra_number) = rx.recv() => {
                    process_sync_job(
                        &pool,
                        &logger,
                        &codechef_client,
                        &tx,
                        &priority_tx,
                        ra_number,
                        false
                    ).await;
                }

                else => {
                    let _ = logger.log("CodeChef worker channels closed. Shutting down.");
                    break;
                }
            }
        }
    }
}

async fn process_sync_job(
    pool: &DbPool,
    logger: &FileLogger,
    codechef_client: &CodeChefClient,
    tx: &Sender<String>,
    priority_tx: &Sender<String>,
    ra_number: String,
    is_priority: bool,
) {
    let job_type = if is_priority { "high priority" } else { "normal" };
    
    if !should_sync_student(pool, &ra_number, logger).await {
        let _ = logger.log_with_level("INFO", &format!("Skipping CodeChef sync for {} (synced within last 6 hours)", ra_number));
        return;
    }

    let _ = logger.log_with_level("INFO", &format!("Processing {} CodeChef sync job for {}", job_type, ra_number));
    
    match api::sync_single_student(pool, codechef_client, &ra_number).await {
        Ok(total_solved) => {
            update_last_synced(pool, &ra_number).await;
            
            // Fetch student to get last 30 days count
            match crate::db::get_student_by_ra(pool, &ra_number).await {
                Ok(student) => {
                    let _ = logger.log_with_level("INFO", &format!(
                        "✅ Successfully synced CodeChef for student {}: {} total questions solved, {} in last 30 days",
                        ra_number,
                        total_solved,
                        student.codechef_solved_last_30_days.unwrap_or(0)
                    ));
                }
                Err(e) => {
                    let _ = logger.log_with_level("WARNING", &format!(
                        "Synced CodeChef for {} ({} total) but failed to fetch updated details: {}",
                        ra_number, total_solved, e
                    ));
                }
            }
        },
        Err(e) => {
            let error_msg = e.to_string();
            
            // Check for rate limit (429)
            if error_msg.contains("429") || error_msg.to_lowercase().contains("rate limit") {
                let cooldown_mins = rand::thread_rng().gen_range(20..=25);
                let cooldown_secs = cooldown_mins * 60;
                let _ = logger.log_with_level("WARNING", &format!("⚠️ Rate limit detected for CodeChef student {}", ra_number));
                let _ = logger.log_with_level("WARNING", &format!("🛑 Pausing CodeChef sync for {} minutes ({} seconds)...", cooldown_mins, cooldown_secs));
                
                tokio::time::sleep(std::time::Duration::from_secs(cooldown_secs)).await;
                let _ = logger.log_with_level("INFO", "✅ CodeChef cooldown completed. Resuming sync.");
                
                // Requeue the failed job
                if is_priority {
                    let _ = priority_tx.send(ra_number).await;
                } else {
                    let _ = tx.send(ra_number).await;
                }
            }
            // Check for profile not found
            else if error_msg.contains("404") || error_msg.contains("Profile not found") {
                let _ = logger.log_with_level("WARNING", &format!(
                    "⚠️ CodeChef profile not found for student {} (username may be invalid or profile doesn't exist)",
                    ra_number
                ));
            }
            // Other errors
            else {
                let _ = logger.log_with_level("ERROR", &format!("❌ Failed to sync CodeChef for {}: {}", ra_number, error_msg));
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
            let _ = self.logger.log_with_level("ERROR", &format!("Failed to add CodeChef sync job: {}", e));
        }
    }

    pub async fn add_priority_job(&self, ra_number: String) {
        if let Err(e) = self.priority_tx.send(ra_number).await {
            let _ = self.logger.log_with_level("ERROR", &format!("Failed to add priority CodeChef sync job: {}", e));
        }
    }
}

async fn should_sync_student(pool: &sqlx::MySqlPool, ra_number: &str, logger: &FileLogger) -> bool {
    let result = sqlx::query!(
        "SELECT codechef_last_synced_at FROM STUDENTS WHERE registration_number = ?",
        ra_number
    )
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(record)) => {
            if let Some(last_synced) = record.codechef_last_synced_at {
                let now = Utc::now();
                let last_synced_utc = last_synced.and_utc();
                let time_since_sync = now.signed_duration_since(last_synced_utc);

                if time_since_sync < Duration::hours(6) {
                    let hours_ago = time_since_sync.num_hours();
                    let minutes_ago = time_since_sync.num_minutes() % 60;
                    let _ = logger.log_with_level("INFO", &format!(
                        "Student {} was synced {}h {}m ago (within 6-hour window)",
                        ra_number, hours_ago, minutes_ago
                    ));
                    return false;
                } else {
                    let hours_ago = time_since_sync.num_hours();
                    let _ = logger.log_with_level("INFO", &format!(
                        "Student {} last synced {}h ago (outside 6-hour window) - will sync",
                        ra_number, hours_ago
                    ));
                    return true;
                }
            } else {
                let _ = logger.log_with_level("INFO", &format!(
                    "Student {} has never been synced - will sync",
                    ra_number
                ));
                return true;
            }
        }
        Ok(None) => {
            let _ = logger.log_with_level("WARN", &format!(
                "Student {} not found in database - will attempt sync anyway",
                ra_number
            ));
            return true;
        }
        Err(e) => {
            let _ = logger.log_with_level("ERROR", &format!(
                "Failed to check sync status for {}: {} - will attempt sync anyway",
                ra_number, e
            ));
            return true;
        }
    }
}

async fn update_last_synced(pool: &sqlx::MySqlPool, ra_number: &str) {
    let _ = sqlx::query("UPDATE STUDENTS SET codechef_last_synced_at = ? WHERE registration_number = ?")
        .bind(Utc::now())
        .bind(ra_number)
        .execute(pool)
        .await;
}
