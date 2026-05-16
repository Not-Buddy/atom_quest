use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use tracing::{error, info, instrument, warn};

use crate::db::{DbPool, get_student_by_ra};

#[derive(Deserialize, Debug)]
struct CodeforcesResponse {
    status: String,
    result: Option<Vec<UserInfo>>,
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    rating: Option<i32>,
    #[serde(rename = "maxRating")]
    max_rating: Option<i32>,
    rank: Option<String>,
}

#[derive(Deserialize, Debug)]
struct SubmissionsResponse {
    status: String,
    result: Option<Vec<Submission>>,
}

#[derive(Deserialize, Debug)]
struct Submission {
    #[serde(rename = "creationTimeSeconds")]
    creation_time_seconds: i64,
    verdict: Option<String>,
    problem: Problem,
}

#[derive(Deserialize, Debug)]
struct Problem {
    #[serde(rename = "contestId")]
    contest_id: Option<i32>,
    index: String,
}

#[derive(Debug)]
pub struct UserStats {
    pub total_solved: i32,
    pub last_30_days_solved: i32,
    pub rating: i32,
    pub max_rating: i32,
    pub rank: String,
}

/// Fetch user info and statistics from Codeforces
#[instrument]
pub async fn fetch_user_stats(
    client: &Client,
    username: &str,
) -> Result<Option<UserStats>, reqwest::Error> {
    info!(username = %username, "Fetching Codeforces stats");

    // Fetch user info
    let user_info_url = format!(
        "https://codeforces.com/api/user.info?handles={}",
        username
    );

    let user_resp = client.get(&user_info_url).send().await?;

    if !user_resp.status().is_success() {
        warn!(
            username = %username,
            status = %user_resp.status(),
            "Codeforces user.info API returned error"
        );
        return Ok(None);
    }

    let user_json: CodeforcesResponse = user_resp.json().await?;

    if user_json.status != "OK" || user_json.result.is_none() {
        warn!(username = %username, "Codeforces user not found");
        return Ok(None);
    }

    let user_info = &user_json.result.unwrap()[0];
    let rating = user_info.rating.unwrap_or(0);
    let max_rating = user_info.max_rating.unwrap_or(0);
    let rank = user_info.rank.clone().unwrap_or_else(|| "unrated".to_string());

    // Fetch user submissions
    let submissions_url = format!(
        "https://codeforces.com/api/user.status?handle={}&from=1&count=1000",
        username
    );

    let submissions_resp = client.get(&submissions_url).send().await?;

    if !submissions_resp.status().is_success() {
        warn!(
            username = %username,
            status = %submissions_resp.status(),
            "Codeforces user.status API returned error"
        );
        return Ok(None);
    }

    let submissions_json: SubmissionsResponse = submissions_resp.json().await?;

    if submissions_json.status != "OK" || submissions_json.result.is_none() {
        warn!(username = %username, "Failed to fetch Codeforces submissions");
        return Ok(None);
    }

    let submissions = submissions_json.result.unwrap();

    // Count unique problems solved (verdict = "OK")
    let mut solved_problems = std::collections::HashSet::new();
    let mut last_30_days_problems = std::collections::HashSet::new();

    let thirty_days_ago = Utc::now() - Duration::days(30);

    for submission in submissions {
        if let Some(verdict) = &submission.verdict
            && verdict == "OK" {
                // Create unique problem identifier
                let problem_id = format!(
                    "{}_{}",
                    submission.problem.contest_id.unwrap_or(0),
                    submission.problem.index
                );

                solved_problems.insert(problem_id.clone());

                // Check if submission is within last 30 days
                if let Some(submission_time) =
                    DateTime::from_timestamp(submission.creation_time_seconds, 0)
                    && submission_time > thirty_days_ago {
                        last_30_days_problems.insert(problem_id);
                    }
            }
    }

    let total_solved = solved_problems.len() as i32;
    let last_30_days_solved = last_30_days_problems.len() as i32;

    info!(
        username = %username,
        total_solved = total_solved,
        last_30_days = last_30_days_solved,
        rating = rating,
        rank = %rank,
        "Successfully fetched Codeforces stats"
    );

    Ok(Some(UserStats {
        total_solved,
        last_30_days_solved,
        rating,
        max_rating,
        rank,
    }))
}

/// Create HTTP client with proper headers
pub fn create_client() -> Client {
    Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("Failed to create HTTP client")
}

/// Sync single student's Codeforces stats
#[instrument(skip(pool))]
pub async fn sync_single_student(pool: &DbPool, ra_number: &str) -> Result<i32, sqlx::Error> {
    let student = get_student_by_ra(pool, ra_number).await?;
    let client = create_client();

    match fetch_user_stats(
        &client,
        student.codeforces_username.as_deref().unwrap_or_default(),
    )
    .await
    {
        Ok(Some(user_stats)) => {
            sqlx::query(
                "UPDATE STUDENTS SET
                codeforces_total_solved = ?,
                codeforces_solved_last_30_days = ?,
                codeforces_rating = ?,
                codeforces_max_rating = ?,
                codeforces_rank = ?
                WHERE registration_number = ?"
            )
            .bind(user_stats.total_solved)
            .bind(user_stats.last_30_days_solved)
            .bind(user_stats.rating)
            .bind(user_stats.max_rating)
            .bind(&user_stats.rank)
            .bind(ra_number)
            .execute(pool)
            .await?;

            info!(
                questions_solved = user_stats.total_solved,
                last_30_days_solved = user_stats.last_30_days_solved,
                rating = user_stats.rating,
                rank = %user_stats.rank,
                "Codeforces stats updated"
            );
            Ok(user_stats.total_solved)
        }
        Ok(None) => {
            warn!(
                username = %student.codeforces_username.as_deref().unwrap_or_default(),
                "User not found on Codeforces"
            );
            Err(sqlx::Error::RowNotFound)
        }
        Err(e) => {
            // Return the actual API error, not a fake RowNotFound error
            error!(error = %e, "Failed to fetch Codeforces stats");
            Err(sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Codeforces API error: {}", e)
            )))
        }
    }
}
