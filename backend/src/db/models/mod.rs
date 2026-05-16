use serde::{Deserialize, Serialize};

// Module declarations
pub mod faculty;
pub mod student;

// Re-export student models
pub use student::{
    LoginRequest, AuthResponse, StudentResponse, UpdateLinksRequest,
    ProfileLinksResponse, ForgotPasswordRequest, ResetPasswordRequest, ResetPasswordFormBody
};

// Re-export faculty models
pub use faculty::{Faculty, FacultyLoginRequest, FacultyAuthResponse, FacultyResponse};

// Update Claims to support both students and faculty
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Claims {
    pub sub: String, // email for students, username for faculty
    pub student_id: i32, // student_id for students, faculty_id for faculty
    pub registration_number: Option<String>, // Only for students
    pub specialization: Option<String>, // Only for faculty
    pub academic_year: Option<String>, // Academic year for faculty
    pub exp: usize,
}
