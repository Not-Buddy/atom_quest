use reqwest::Client;
use scraper::{Html, Selector};
use tracing::{error, info, instrument, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Instant, Duration};

use crate::db::{DbPool, get_student_by_ra};

#[derive(Debug)]
pub struct UserStats {
    pub total_solved: i32,
    pub last_30_days_solved: i32,
}

// Define a Send + Sync error type
#[derive(Debug)]
pub struct CodeChefError(String);

impl std::fmt::Display for CodeChefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CodeChefError {}

impl From<reqwest::Error> for CodeChefError {
    fn from(err: reqwest::Error) -> Self {
        CodeChefError(err.to_string())
    }
}

impl From<serde_json::Error> for CodeChefError {
    fn from(err: serde_json::Error) -> Self {
        CodeChefError(err.to_string())
    }
}

impl From<sqlx::Error> for CodeChefError {
    fn from(err: sqlx::Error) -> Self {
        CodeChefError(err.to_string())
    }
}

/// Global rate limiter for CodeChef requests
pub struct RateLimiter {
    last_request: Mutex<Option<Instant>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            last_request: Mutex::new(None),
        }
    }

    /// Wait for rate limit before making a request (10-20 seconds between requests)
    pub async fn wait(&self) {
        use rand::Rng;
        
        let mut last = self.last_request.lock().await;
        
        if let Some(last_time) = *last {
            let elapsed = last_time.elapsed();
            let min_delay = Duration::from_secs(10);
            
            if elapsed < min_delay {
                let wait_time = min_delay - elapsed;
                info!("Rate limiter: waiting {:?} before next request", wait_time);
                tokio::time::sleep(wait_time).await;
            }
        }
        
        // Update last request time
        *last = Some(Instant::now());
        
        // Add random jitter (0-10 seconds) to avoid patterns
        let jitter_secs = rand::thread_rng().gen_range(0..=10);
        if jitter_secs > 0 {
            tokio::time::sleep(Duration::from_secs(jitter_secs)).await;
        }
    }
}

/// Shared HTTP client and rate limiter
pub struct CodeChefClient {
    client: Client,
    rate_limiter: Arc<RateLimiter>,
}

impl CodeChefClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .pool_max_idle_per_host(1)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            rate_limiter: Arc::new(RateLimiter::new()),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn rate_limiter(&self) -> Arc<RateLimiter> {
        self.rate_limiter.clone()
    }
}

/// Fetch CodeChef stats by scraping the user profile page
#[instrument(skip(client, rate_limiter))]
pub async fn fetch_user_stats(
    client: &Client,
    rate_limiter: Arc<RateLimiter>,
    username: &str,
) -> Result<Option<UserStats>, CodeChefError> {
    info!(username = %username, "Fetching CodeChef stats by scraping profile");
    
    // Apply rate limiting
    rate_limiter.wait().await;
    
    let url = format!("https://www.codechef.com/users/{}", username);
    
    let resp = client
        .get(&url)
        .send()
        .await?;

    let status = resp.status();
    
    // Handle different HTTP status codes
    if !status.is_success() {
        let status_code = status.as_u16();
        let error_msg = format!(
            "HTTP {}: {}", 
            status_code,
            status.canonical_reason().unwrap_or("Unknown error")
        );
        
        warn!(
            username = %username,
            status = %status,
            "CodeChef profile page returned error"
        );
        
        // Return error for rate limiting (429) so it can be detected
        if status_code == 429 {
            return Err(CodeChefError(error_msg));
        }
        
        // Return error for server errors (5xx) so they can be retried
        if status.is_server_error() {
            return Err(CodeChefError(error_msg));
        }
        
        // For 404 and other client errors, return Ok(None) - profile doesn't exist
        return Ok(None);
    }

    let html_content = resp.text().await?;
    let document = Html::parse_document(&html_content);

    // Look for "Total Problems Solved: X" in the HTML
    let problems_selector = Selector::parse("h3").unwrap();
    
    let mut total_solved = 0;
    
    for element in document.select(&problems_selector) {
        let text = element.text().collect::<String>();
        
        // Match pattern like "Total Problems Solved: 6"
        if text.contains("Total Problems Solved:")
            && let Some(number_str) = text.split(':').nth(1) {
                total_solved = number_str.trim().parse::<i32>().unwrap_or(0);
                break;
            }
    }

    if total_solved == 0 {
        warn!(username = %username, "Could not find problems solved count");
        return Ok(None);
    }

    // For last 30 days, we can parse the heatmap data from the JavaScript
    // Look for: var userDailySubmissionsStats = [{"date":"2026-1-2","value":6}];
    let mut last_30_days_solved = 0;
    
    if let Some(start_idx) = html_content.find("var userDailySubmissionsStats = ")
        && let Some(json_start) = html_content[start_idx..].find('[')
            && let Some(json_end) = html_content[start_idx + json_start..].find("];") {
                let json_str = &html_content[start_idx + json_start..start_idx + json_start + json_end + 1];
                
                // Parse the JSON array
                if let Ok(submissions) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                    // Calculate last 30 days
                    let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);
                    
                    for submission in submissions {
                        if let Some(date_str) = submission["date"].as_str() {
                            // Parse date (format: "2026-1-2")
                            let parts: Vec<&str> = date_str.split('-').collect();
                            if parts.len() == 3
                                && let (Ok(year), Ok(month), Ok(day)) = (
                                    parts[0].parse::<i32>(),
                                    parts[1].parse::<u32>(),
                                    parts[2].parse::<u32>()
                                )
                                    && let Some(submission_date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                                        let submission_datetime = submission_date.and_hms_opt(0, 0, 0)
                                            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc));
                                        
                                        if let Some(dt) = submission_datetime
                                            && dt > thirty_days_ago
                                                && let Some(value) = submission["value"].as_i64() {
                                                    last_30_days_solved += value as i32;
                                                }
                                    }
                        }
                    }
                }
            }

    info!(
        username = %username,
        total_solved = total_solved,
        last_30_days = last_30_days_solved,
        "Successfully scraped CodeChef stats"
    );

    Ok(Some(UserStats {
        total_solved,
        last_30_days_solved,
    }))
}

/// Sync single student's CodeChef stats
#[instrument(skip(pool, codechef_client))]
pub async fn sync_single_student(
    pool: &DbPool,
    codechef_client: &CodeChefClient,
    ra_number: &str,
) -> Result<i32, CodeChefError> {
    let student = get_student_by_ra(pool, ra_number).await?;
    
    // Check if student has a CodeChef username set
    let codechef_username = match &student.codechef_username {
        Some(username) if !username.is_empty() => username.trim(),
        _ => {
            warn!(ra_number = %ra_number, "Student does not have a CodeChef username set");
            return Err(CodeChefError("No CodeChef username set".to_string()));
        }
    };
    
    match fetch_user_stats(
        codechef_client.client(),
        codechef_client.rate_limiter(),
        codechef_username
    ).await {
        Ok(Some(user_stats)) => {
            sqlx::query(
                "UPDATE STUDENTS SET codechef_total_solved = ?, codechef_solved_last_30_days = ? WHERE registration_number = ?"
            )
            .bind(user_stats.total_solved)
            .bind(user_stats.last_30_days_solved)
            .bind(ra_number)
            .execute(pool)
            .await?;

            info!(
                questions_solved = user_stats.total_solved,
                last_30_days_solved = user_stats.last_30_days_solved,
                "CodeChef stats updated"
            );
            Ok(user_stats.total_solved)
        }
        Ok(None) => {
            warn!(username = %codechef_username, "User not found on CodeChef");
            Err(CodeChefError("HTTP 404: Profile not found".to_string()))
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch CodeChef stats");
            Err(e)
        }
    }
}
