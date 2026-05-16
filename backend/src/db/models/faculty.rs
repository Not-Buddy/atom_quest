use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Faculty {
    pub id: i32,
    pub specialization: String,
    pub academic_year: Option<String>,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct FacultyLoginRequest {
    pub specialization: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct FacultyAuthResponse {
    pub token: String,
    pub faculty: FacultyResponse,
}

#[derive(Debug, Serialize)]
pub struct FacultyResponse {
    pub id: i32,
    pub specialization: String,
    pub academic_year: Option<String>,
    pub username: String,
}

impl From<Faculty> for FacultyResponse {
    fn from(f: Faculty) -> Self {
        Self {
            id: f.id,
            specialization: f.specialization,
            academic_year: f.academic_year,
            username: f.username,
        }
    }
}
