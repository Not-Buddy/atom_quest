use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{error, info, instrument, warn};

use crate::db::{DbPool, get_student_by_ra};

#[derive(Deserialize, Debug)]
struct LeetCodeResponse {
    data: Data,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Data {
    matched_user: Option<MatchedUser>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MatchedUser {
    submit_stats: SubmitStats,
    submission_calendar: String, // JSON string containing the heatmap
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SubmitStats {
    ac_submission_num: Vec<AcSubmissionNum>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AcSubmissionNum {
    difficulty: String,
    count: i32,
}

#[derive(Debug)]
pub struct UserStats {
    pub total_solved: i32,
    pub last_30_days_solved: i32,
}

/// Fetch both total solved and last 30 days solved for a LeetCode username
#[instrument]
pub async fn fetch_user_stats(
    client: &Client,
    username: &str,
) -> Result<Option<UserStats>, reqwest::Error> {
    info!(username = %username, "Fetching LeetCode stats");
    
    let query = json!({
        "query": r#"
            query getUserProfile($username: String!) {
                matchedUser(username: $username) {
                    submitStats: submitStatsGlobal {
                        acSubmissionNum { difficulty count }
                    }
                    submissionCalendar
                }
            }
        "#,
        "variables": { "username": username }
    });

    let resp = client
        .post("https://leetcode.com/graphql")
        .json(&query)
        .send()
        .await?;

    if !resp.status().is_success() {
        warn!(
            username = %username,
            status = %resp.status(),
            "LeetCode API returned error"
        );
        return Ok(None);
    }

    let json: LeetCodeResponse = resp.json().await?;

    // Get total solved
    let matched_user = match json.data.matched_user {
        Some(user) => user,
        None => return Ok(None),
    };

    let total_solved = matched_user
        .submit_stats
        .ac_submission_num
        .into_iter()
        .find(|x| x.difficulty == "All")
        .map(|x| x.count)
        .unwrap_or(0);

    // Calculate last 30 days solved from submission calendar
    let last_30_days_solved = calculate_last_30_days(&matched_user.submission_calendar);

    info!(
        username = %username,
        total_solved = total_solved,
        last_30_days = last_30_days_solved,
        "Successfully fetched stats"
    );

    Ok(Some(UserStats {
        total_solved,
        last_30_days_solved,
    }))
}

fn calculate_last_30_days(calendar_json: &str) -> i32 {
    // Parse the submission calendar JSON string
    let calendar: HashMap<String, i32> = match serde_json::from_str(calendar_json) {
        Ok(cal) => cal,
        Err(_) => return 0,
    };

    // Calculate cutoff timestamp (30 days ago)
    let thirty_days_ago = Utc::now() - Duration::days(30);
    let cutoff_timestamp = thirty_days_ago.timestamp();

    // Sum submissions from last 30 days
    calendar
        .iter()
        .filter_map(|(timestamp_str, count)| {
            timestamp_str.parse::<i64>().ok().and_then(|ts| {
                if ts > cutoff_timestamp {
                    Some(count)
                } else {
                    None
                }
            })
        })
        .sum()
}

/// Create HTTP client with proper headers
pub fn create_client() -> Client {
    Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
}

// Sync single student's LeetCode stats
#[instrument(skip(pool))]
pub async fn sync_single_student(pool: &DbPool, ra_number: &str) -> Result<i32, sqlx::Error> {
    let student = get_student_by_ra(pool, ra_number).await?;
    let client = create_client();
    
    match fetch_user_stats(&client, student.leetcode_username.as_deref().unwrap_or_default()).await {
        Ok(Some(user_stats)) => {
            sqlx::query(
                "UPDATE STUDENTS SET leetcode_total_solved = ?, leetcode_solved_last_30_days = ? WHERE registration_number = ?"
            )
            .bind(user_stats.total_solved)
            .bind(user_stats.last_30_days_solved)
            .bind(ra_number)
            .execute(pool)
            .await?;

            info!(
                questions_solved = user_stats.total_solved,
                last_30_days_solved = user_stats.last_30_days_solved,
                "Stats updated"
            );
            
            Ok(user_stats.total_solved)
        }
        Ok(None) => {
            warn!(username = %student.leetcode_username.as_deref().unwrap_or_default(), "User not found on LeetCode");
            Err(sqlx::Error::RowNotFound)
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch stats");
            Err(sqlx::Error::RowNotFound)
        }
    }
}
