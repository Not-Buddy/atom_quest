use serde::{Deserialize, Serialize};
use validator::Validate;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Student {
    pub id: i32,
    pub serial_number: Option<i32>,
    pub registration_number: String,
    pub password: String,
    pub full_name: String,
    pub college: Option<String>,
    pub course: Option<String>,
    pub specialization: Option<String>,
    pub academic_year: Option<String>,
    pub email: Option<String>,

    // GitHub and LinkedIn
    pub github_username: Option<String>,
    pub linkedin_url: Option<String>,

    // LeetCode fields
    pub leetcode_username: Option<String>,
    pub leetcode_total_solved: Option<i32>,
    pub leetcode_solved_last_30_days: Option<i32>,
    pub has_leetcode_account: Option<bool>,
    pub leetcode_prev_month_solved: Option<i32>,
    pub leetcode_last_synced_at: Option<chrono::NaiveDateTime>,

    // CodeChef fields
    pub codechef_username: Option<String>,
    pub codechef_total_solved: Option<i32>,
    pub codechef_solved_last_30_days: Option<i32>,
    pub has_codechef_account: Option<bool>,
    pub codechef_prev_month_solved: Option<i32>,
    pub codechef_last_synced_at: Option<chrono::NaiveDateTime>,

    // Codeforces fields
    pub codeforces_username: Option<String>,
    pub codeforces_total_solved: Option<i32>,
    pub codeforces_solved_last_30_days: Option<i32>,
    pub has_codeforces_account: Option<bool>,
    pub codeforces_prev_month_solved: Option<i32>,
    pub codeforces_rating: Option<i32>,
    pub codeforces_max_rating: Option<i32>,
    pub codeforces_rank: Option<String>,
    pub codeforces_last_synced_at: Option<chrono::NaiveDateTime>,

    // Aggregate fields (automatically calculated by database)
    pub total_platforms_solved: Option<i32>,
    pub total_solved_last_30_days: Option<i32>,

    // Metadata
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, FromRow)]
#[allow(unused)]
pub struct PasswordResetToken {
    pub id: i32,
    pub student_id: i32,
    pub token: String,
    pub expires_at: chrono::NaiveDateTime,
    pub used: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordFormBody {
    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub student: StudentResponse,
}

#[derive(Debug, Serialize)]
pub struct StudentResponse {
    pub id: i32,
    pub serial_number: Option<i32>,
    pub registration_number: String,
    pub full_name: String,
    pub college: Option<String>,
    pub course: Option<String>,
    pub specialization: Option<String>,
    pub academic_year: Option<String>,
    pub email: Option<String>,

    // GitHub and LinkedIn
    pub github_username: Option<String>,
    pub linkedin_url: Option<String>,

    // LeetCode fields
    pub leetcode_username: Option<String>,
    pub leetcode_total_solved: Option<i32>,
    pub leetcode_solved_last_30_days: Option<i32>,
    pub has_leetcode_account: Option<bool>,
    pub leetcode_prev_month_solved: Option<i32>,
    pub leetcode_last_synced_at: Option<chrono::NaiveDateTime>,

    // CodeChef fields
    pub codechef_username: Option<String>,
    pub codechef_total_solved: Option<i32>,
    pub codechef_solved_last_30_days: Option<i32>,
    pub has_codechef_account: Option<bool>,
    pub codechef_prev_month_solved: Option<i32>,
    pub codechef_last_synced_at: Option<chrono::NaiveDateTime>,

    // Codeforces fields
    pub codeforces_username: Option<String>,
    pub codeforces_total_solved: Option<i32>,
    pub codeforces_solved_last_30_days: Option<i32>,
    pub has_codeforces_account: Option<bool>,
    pub codeforces_prev_month_solved: Option<i32>,
    pub codeforces_rating: Option<i32>,
    pub codeforces_max_rating: Option<i32>,
    pub codeforces_rank: Option<String>,
    pub codeforces_last_synced_at: Option<chrono::NaiveDateTime>,

    // Aggregate fields (automatically calculated by database)
    pub total_platforms_solved: Option<i32>,
    pub total_solved_last_30_days: Option<i32>,

    // Metadata
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

impl From<Student> for StudentResponse {
    fn from(s: Student) -> Self {
        Self {
            id: s.id,
            serial_number: s.serial_number,
            registration_number: s.registration_number,
            full_name: s.full_name,
            college: s.college,
            course: s.course,
            specialization: s.specialization,
            academic_year: s.academic_year,
            email: s.email,

            github_username: s.github_username,
            linkedin_url: s.linkedin_url,

            leetcode_username: s.leetcode_username,
            leetcode_total_solved: s.leetcode_total_solved,
            leetcode_solved_last_30_days: s.leetcode_solved_last_30_days,
            has_leetcode_account: s.has_leetcode_account,
            leetcode_prev_month_solved: s.leetcode_prev_month_solved,
            leetcode_last_synced_at: s.leetcode_last_synced_at,

            codechef_username: s.codechef_username,
            codechef_total_solved: s.codechef_total_solved,
            codechef_solved_last_30_days: s.codechef_solved_last_30_days,
            has_codechef_account: s.has_codechef_account,
            codechef_prev_month_solved: s.codechef_prev_month_solved,
            codechef_last_synced_at: s.codechef_last_synced_at,

            codeforces_username: s.codeforces_username,
            codeforces_total_solved: s.codeforces_total_solved,
            codeforces_solved_last_30_days: s.codeforces_solved_last_30_days,
            has_codeforces_account: s.has_codeforces_account,
            codeforces_prev_month_solved: s.codeforces_prev_month_solved,
            codeforces_rating: s.codeforces_rating,
            codeforces_max_rating: s.codeforces_max_rating,
            codeforces_rank: s.codeforces_rank,
            codeforces_last_synced_at: s.codeforces_last_synced_at,

            // Map the new aggregate fields
            total_platforms_solved: s.total_platforms_solved,
            total_solved_last_30_days: s.total_solved_last_30_days,

            updated_at: s.updated_at,
            created_at: s.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLinksRequest {
    pub github_link: Option<String>,
    pub linkedin_link: Option<String>,
    pub leetcode_link: Option<String>,
    pub codechef_link: Option<String>,
    pub codeforces_link: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProfileLinksResponse {
    pub github_link: Option<String>,
    pub leetcode_link: Option<String>,
    pub linkedin_link: Option<String>,
    pub codechef_link: Option<String>,
    pub codeforces_link: Option<String>,
}
