use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

// ============================================================
// Database row structs
// ============================================================

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub department_id: Option<i32>,
    pub role: String,
    pub manager_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct Department {
    pub id: i32,
    pub name: String,
    pub short_name: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct ThrustArea {
    pub id: i32,
    pub name: String,
    pub department_id: Option<i32>,
    pub created_by: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct GoalCycle {
    pub id: i32,
    pub name: String,
    pub goal_setting_opens: Option<NaiveDateTime>,
    pub q1_opens: Option<NaiveDateTime>,
    pub q2_opens: Option<NaiveDateTime>,
    pub q3_opens: Option<NaiveDateTime>,
    pub q4_opens: Option<NaiveDateTime>,
    pub is_active: Option<bool>,
    pub created_by: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct GoalSheet {
    pub id: i32,
    pub user_id: i32,
    pub cycle_id: i32,
    pub status: String,
    pub submitted_at: Option<NaiveDateTime>,
    pub approved_at: Option<NaiveDateTime>,
    pub approved_by: Option<i32>,
    pub returned_reason: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct Goal {
    pub id: i32,
    pub sheet_id: i32,
    pub thrust_area_id: Option<i32>,
    pub title: String,
    pub description: Option<String>,
    pub uom_type: String,
    pub target_value: f64,
    pub target_date: Option<NaiveDate>,
    pub weightage: f64,
    pub is_shared: Option<bool>,
    pub shared_from_goal_id: Option<i32>,
    pub sort_order: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct Achievement {
    pub id: i32,
    pub goal_id: i32,
    pub quarter: String,
    pub actual_value: Option<f64>,
    pub actual_date: Option<NaiveDate>,
    pub status: String,
    pub computed_score: Option<f64>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct CheckinComment {
    pub id: i32,
    pub goal_sheet_id: i32,
    pub quarter: String,
    pub manager_id: i32,
    pub comment: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct AuditLogEntry {
    pub id: i32,
    pub table_name: String,
    pub record_id: i32,
    pub field_name: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Option<i32>,
    pub changed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct PasswordResetToken {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub expires_at: NaiveDateTime,
    pub used: bool,
    pub created_at: NaiveDateTime,
}

// ============================================================
// Request/Response DTOs
// ============================================================

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub full_name: String,
    pub department_id: Option<i32>,
    pub role: String,
    pub manager_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            full_name: u.full_name,
            department_id: u.department_id,
            role: u.role,
            manager_id: u.manager_id,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub user_id: i32,
    pub email: String,
    pub role: String,
    pub department_id: Option<i32>,
    pub exp: usize,
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

// ============================================================
// Goal DTOs
// ============================================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGoalRequest {
    pub thrust_area_id: Option<i32>,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub description: Option<String>,
    pub uom_type: String,
    pub target_value: f64,
    pub target_date: Option<String>,
    #[validate(range(min = 10.0, max = 100.0))]
    pub weightage: f64,
    pub is_shared: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateGoalRequest {
    pub thrust_area_id: Option<i32>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub uom_type: Option<String>,
    pub target_value: Option<f64>,
    pub target_date: Option<String>,
    pub weightage: Option<f64>,
    pub is_shared: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct GoalResponse {
    pub id: i32,
    pub sheet_id: i32,
    pub thrust_area_id: Option<i32>,
    pub thrust_area_name: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub uom_type: String,
    pub target_value: f64,
    pub target_date: Option<String>,
    pub weightage: f64,
    pub is_shared: bool,
    pub shared_from_goal_id: Option<i32>,
    pub sort_order: i32,
    pub achievements: Vec<AchievementResponse>,
}

#[derive(Debug, Serialize)]
pub struct GoalSheetResponse {
    pub id: i32,
    pub user_id: i32,
    pub user_name: Option<String>,
    pub cycle_id: i32,
    pub cycle_name: Option<String>,
    pub status: String,
    pub submitted_at: Option<String>,
    pub approved_at: Option<String>,
    pub approved_by: Option<i32>,
    pub returned_reason: Option<String>,
    pub goals: Vec<GoalResponse>,
    pub total_weightage: f64,
    pub checkins: Vec<CheckinCommentResponse>,
}

#[derive(Debug, Serialize)]
pub struct GoalSheetSummary {
    pub id: i32,
    pub user_id: i32,
    pub user_name: Option<String>,
    pub cycle_id: i32,
    pub cycle_name: Option<String>,
    pub status: String,
    pub goal_count: i64,
    pub total_weightage: f64,
}

// ============================================================
// Achievement DTOs
// ============================================================

#[derive(Debug, Deserialize, Validate)]
pub struct AchievementUpdateRequest {
    pub actual_value: Option<f64>,
    pub actual_date: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AchievementResponse {
    pub id: i32,
    pub goal_id: i32,
    pub quarter: String,
    pub actual_value: Option<f64>,
    pub actual_date: Option<String>,
    pub status: String,
    pub computed_score: Option<f64>,
}

impl From<Achievement> for AchievementResponse {
    fn from(a: Achievement) -> Self {
        Self {
            id: a.id,
            goal_id: a.goal_id,
            quarter: a.quarter,
            actual_value: a.actual_value,
            actual_date: a.actual_date.map(|d| d.format("%Y-%m-%d").to_string()),
            status: a.status,
            computed_score: a.computed_score,
        }
    }
}

// ============================================================
// Checkin DTOs
// ============================================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCheckinRequest {
    pub quarter: String,
    #[validate(length(min = 1))]
    pub comment: String,
}

#[derive(Debug, Serialize)]
pub struct CheckinCommentResponse {
    pub id: i32,
    pub goal_sheet_id: i32,
    pub quarter: String,
    pub manager_id: i32,
    pub manager_name: Option<String>,
    pub comment: String,
    pub created_at: Option<String>,
}

// ============================================================
// Cycle DTOs
// ============================================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCycleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub goal_setting_opens: Option<String>,
    pub q1_opens: Option<String>,
    pub q2_opens: Option<String>,
    pub q3_opens: Option<String>,
    pub q4_opens: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCycleRequest {
    pub name: Option<String>,
    pub goal_setting_opens: Option<String>,
    pub q1_opens: Option<String>,
    pub q2_opens: Option<String>,
    pub q3_opens: Option<String>,
    pub q4_opens: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================================
// Department / ThrustArea DTOs
// ============================================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDepartmentRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 20))]
    pub short_name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateThrustAreaRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub department_id: Option<i32>,
}

// ============================================================
// User management DTOs
// ============================================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 150))]
    pub full_name: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub department_id: Option<i32>,
    pub role: String,
    pub manager_id: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub department_id: Option<i32>,
    pub role: Option<String>,
    pub manager_id: Option<i32>,
    pub password: Option<String>,
}

// ============================================================
// Manager DTOs
// ============================================================

#[derive(Debug, Deserialize, Validate)]
pub struct ManagerEditGoalRequest {
    pub target_value: Option<f64>,
    #[validate(range(min = 10.0, max = 100.0))]
    pub weightage: Option<f64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReturnSheetRequest {
    #[validate(length(min = 1))]
    pub reason: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SharedGoalPushRequest {
    pub sheet_ids: Vec<i32>,
    pub thrust_area_id: Option<i32>,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub description: Option<String>,
    pub uom_type: String,
    pub target_value: f64,
    pub target_date: Option<String>,
    #[validate(range(min = 10.0, max = 100.0))]
    pub weightage: f64,
}

// ============================================================
// Report DTOs
// ============================================================

#[derive(Debug, Serialize)]
pub struct AchievementReportEntry {
    pub user_name: String,
    pub department: Option<String>,
    pub cycle_name: String,
    pub sheet_status: String,
    pub goal_title: String,
    pub uom_type: String,
    pub target_value: f64,
    pub weightage: f64,
    pub q1_actual: Option<f64>,
    pub q1_score: Option<f64>,
    pub q2_actual: Option<f64>,
    pub q2_score: Option<f64>,
    pub q3_actual: Option<f64>,
    pub q3_score: Option<f64>,
    pub q4_actual: Option<f64>,
    pub q4_score: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct CompletionDashboardEntry {
    pub department: Option<String>,
    pub total_sheets: i64,
    pub draft_count: i64,
    pub submitted_count: i64,
    pub approved_count: i64,
    pub returned_count: i64,
    pub locked_count: i64,
}
