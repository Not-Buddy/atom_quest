use crate::{
    api::{auth::AppState, ApiError},
    db::models::Claims,
};
use axum::{
    extract::State,
    Extension,
    response::IntoResponse,
    http::{header, StatusCode},
};
use rust_xlsxwriter::{Workbook, Format, Color};
use std::sync::Arc;
use std::path::Path;
use std::fs;
use serde::Serialize;
use sqlx::FromRow;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Debug)]
struct StudentSubmissionData {
    registration_number: String,
    full_name: String,
    specialization: Option<String>,
    academic_year: Option<String>,
    github_submitted: bool,
    linkedin_submitted: bool,
    leetcode_submitted: bool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StudentDefaulterData {
    pub registration_number: String,
    pub full_name: String,
    pub specialization: Option<String>,
    pub academic_year: Option<String>,
    pub leetcode_solved_last_30_days: i32,
    pub codechef_solved_last_30_days: Option<i32>,
    pub codeforces_solved_last_30_days: Option<i32>,
    pub total_solved_last_30_days: Option<i32>,
}

// ============================================================================
// ENDPOINT: STUDENT SUBMISSIONS REPORT
// ============================================================================

use axum::extract::Query;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub specialization: Option<String>,
}

pub async fn get_student_submissions_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ReportQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let faculty_specialization = claims.specialization
        .ok_or_else(|| ApiError::Unauthorized("Not a faculty token".to_string()))?;

    let faculty_academic_year = claims.academic_year
        .as_ref()
        .ok_or_else(|| ApiError::Unauthorized("Faculty academic year not found in token".to_string()))?;

    // Determine which specialization to query
    let query_specialization = if faculty_academic_year == "VI" {
        // Admin (Year VI) can query specific specialization via parameter
        // If no parameter provided, they get all specializations
        params.specialization.as_ref().unwrap_or(&faculty_specialization)
    } else if faculty_academic_year == "V" {
        // Department coordinator (Year V) can only see their own specialization
        &faculty_specialization
    } else {
        // Regular faculty can only see their own specialization
        &faculty_specialization
    };

    // Get students based on academic year logic
    let students = if faculty_academic_year == "VI" {
        // Admin gets all years (I-IV) for the specified specialization
        get_all_students_by_specialization(&state.db.pool, query_specialization).await?
    } else if faculty_academic_year == "V" {
        // Department coordinator gets all years (I-IV) for their specialization
        get_all_students_by_specialization(&state.db.pool, query_specialization).await?
    } else {
        // Regular faculty gets their specific year and specialization
        get_students_by_specialization_and_academic_year(&state.db.pool, query_specialization, faculty_academic_year).await?
    };

    let reports_dir = "reports";
    if !Path::new(reports_dir).exists() {
        fs::create_dir_all(reports_dir)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create reports directory: {}", e)))?;
    }

    let filename = format!(
        "student_submissions_{}_{}.xlsx",
        query_specialization.replace(" ", "_").replace("/", "-"),
        faculty_academic_year.replace(" ", "_").replace("/", "-")
    );

    let filepath = format!("{}/{}", reports_dir, filename);

    generate_excel_report_to_file(&students, query_specialization, faculty_academic_year, &filepath)?;

    let file_bytes = fs::read(&filepath)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to read generated file: {}", e)))?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename)),
        ],
        file_bytes,
    ))
}


// ============================================================================
// ENDPOINT: DEFAULTERS REPORT
// ============================================================================

pub async fn get_defaulters_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ReportQuery>,  // ADD THIS LINE
) -> Result<impl IntoResponse, ApiError> {
    let faculty_specialization = claims.specialization
        .ok_or_else(|| ApiError::Unauthorized("Not a faculty token".to_string()))?;

    let faculty_academic_year = claims.academic_year
        .as_ref()
        .ok_or_else(|| ApiError::Unauthorized("Faculty academic year not found in token".to_string()))?;

    // Determine which specialization to query (same logic as submissions)
    let query_specialization = if faculty_academic_year == "VI" {
        // Admin (Year VI) can query specific specialization via parameter
        params.specialization.as_ref().unwrap_or(&faculty_specialization)
    } else if faculty_academic_year == "V" {
        // Department coordinator (Year V) can only see their own specialization
        &faculty_specialization
    } else {
        // Regular faculty can only see their own specialization
        &faculty_specialization
    };

    // Get students based on academic year logic
    let students = if faculty_academic_year == "VI" {
        // Admin gets all years (I-IV) for the specified specialization
        get_all_students_with_question_counts(&state.db.pool, query_specialization).await?
    } else if faculty_academic_year == "V" {
        // Department coordinator gets all years (I-IV) for their specialization
        get_all_students_with_question_counts(&state.db.pool, query_specialization).await?
    } else {
        // Regular faculty gets their specific year and specialization
        get_students_with_question_counts(&state.db.pool, query_specialization, faculty_academic_year).await?
    };

    let reports_dir = "reports";
    if !Path::new(reports_dir).exists() {
        fs::create_dir_all(reports_dir)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create reports directory: {}", e)))?;
    }

    let filename = format!(
        "defaulters_report_{}_{}.xlsx",
        query_specialization.replace(" ", "_").replace("/", "-"),  // CHANGED FROM specialization
        faculty_academic_year.replace(" ", "_").replace("/", "-")
    );

    let filepath = format!("{}/{}", reports_dir, filename);

    generate_defaulters_report_to_file(&students, query_specialization, faculty_academic_year, &filepath)?;  // CHANGED FROM &specialization

    let file_bytes = fs::read(&filepath)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to read generated file: {}", e)))?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename)),
        ],
        file_bytes,
    ))
}

// ============================================================================
// DATABASE QUERIES: SUBMISSIONS
// ============================================================================

async fn get_students_by_specialization_and_academic_year(
    pool: &sqlx::MySqlPool,
    specialization: &str,
    academic_year: &str,
) -> Result<Vec<StudentSubmissionData>, sqlx::Error> {
    let students = sqlx::query_as!(
        StudentSubmissionData,
        r#"
        SELECT
            registration_number,
            full_name,
            specialization,
            academic_year,
            (github_username IS NOT NULL) as `github_submitted: bool`,
            (linkedin_url IS NOT NULL) as `linkedin_submitted: bool`,
            (leetcode_username IS NOT NULL) as `leetcode_submitted: bool`
        FROM STUDENTS
        WHERE specialization = ? AND academic_year = ?
        ORDER BY registration_number ASC
        "#,
        specialization,
        academic_year
    )
    .fetch_all(pool)
    .await?;

    Ok(students)
}

async fn get_all_students_by_specialization(
    pool: &sqlx::MySqlPool,
    specialization: &str,
) -> Result<Vec<StudentSubmissionData>, sqlx::Error> {
    let students = sqlx::query_as!(
        StudentSubmissionData,
        r#"
        SELECT
            registration_number,
            full_name,
            specialization,
            academic_year,
            (github_username IS NOT NULL) as `github_submitted: bool`,
            (linkedin_url IS NOT NULL) as `linkedin_submitted: bool`,
            (leetcode_username IS NOT NULL) as `leetcode_submitted: bool`
        FROM STUDENTS
        WHERE specialization = ? 
        AND academic_year IN ('I', 'II', 'III', 'IV')
        ORDER BY registration_number ASC
        "#,
        specialization
    )
    .fetch_all(pool)
    .await?;

    Ok(students)
}

// ============================================================================
// DATABASE QUERIES: DEFAULTERS
// ============================================================================

async fn get_students_with_question_counts(
    pool: &sqlx::MySqlPool,
    specialization: &str,
    academic_year: &str,
) -> Result<Vec<StudentDefaulterData>, sqlx::Error> {
    let students = sqlx::query_as!(
        StudentDefaulterData,
        r#"
        SELECT
            registration_number,
            full_name,
            specialization,
            academic_year,
            COALESCE(leetcode_solved_last_30_days, 0) as `leetcode_solved_last_30_days: i32`,
            COALESCE(codechef_solved_last_30_days, 0) as `codechef_solved_last_30_days: i32`,
            COALESCE(codeforces_solved_last_30_days, 0) as `codeforces_solved_last_30_days: i32`,
            (COALESCE(leetcode_solved_last_30_days, 0) +
             COALESCE(codechef_solved_last_30_days, 0) +
             COALESCE(codeforces_solved_last_30_days, 0)) as `total_solved_last_30_days: i32`
        FROM STUDENTS
        WHERE specialization = ? AND academic_year = ?
        ORDER BY
            (COALESCE(leetcode_solved_last_30_days, 0) +
             COALESCE(codechef_solved_last_30_days, 0) +
             COALESCE(codeforces_solved_last_30_days, 0)) ASC,
            registration_number ASC
        "#,
        specialization,
        academic_year
    )
    .fetch_all(pool)
    .await?;

    Ok(students)
}

async fn get_all_students_with_question_counts(
    pool: &sqlx::MySqlPool,
    specialization: &str,
) -> Result<Vec<StudentDefaulterData>, sqlx::Error> {
    let students = sqlx::query_as!(
        StudentDefaulterData,
        r#"
        SELECT
            registration_number,
            full_name,
            specialization,
            academic_year,
            COALESCE(leetcode_solved_last_30_days, 0) as `leetcode_solved_last_30_days: i32`,
            COALESCE(codechef_solved_last_30_days, 0) as `codechef_solved_last_30_days: i32`,
            COALESCE(codeforces_solved_last_30_days, 0) as `codeforces_solved_last_30_days: i32`,
            (COALESCE(leetcode_solved_last_30_days, 0) +
             COALESCE(codechef_solved_last_30_days, 0) +
             COALESCE(codeforces_solved_last_30_days, 0)) as `total_solved_last_30_days: i32`
        FROM STUDENTS
        WHERE specialization = ? 
        AND academic_year IN ('I', 'II', 'III', 'IV')
        ORDER BY
            (COALESCE(leetcode_solved_last_30_days, 0) +
             COALESCE(codechef_solved_last_30_days, 0) +
             COALESCE(codeforces_solved_last_30_days, 0)) ASC,
            registration_number ASC
        "#,
        specialization
    )
    .fetch_all(pool)
    .await?;

    Ok(students)
}

// ============================================================================
// EXCEL GENERATION: SUBMISSIONS REPORT
// ============================================================================

fn generate_excel_report_to_file(
    students: &[StudentSubmissionData],
    specialization: &str,
    academic_year: &str,
    filepath: &str,
) -> Result<(), ApiError> {
    // Year V or VI both get multi-year reports
    if academic_year == "V" || academic_year == "VI" {
        return generate_all_years_excel_report(students, specialization, filepath);
    }
    generate_single_year_excel_report(students, specialization, academic_year, filepath)
}

fn generate_single_year_excel_report(
    students: &[StudentSubmissionData],
    specialization: &str,
    academic_year: &str,
    filepath: &str,
) -> Result<(), ApiError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x4472C4))
        .set_font_color(Color::White)
        .set_border(rust_xlsxwriter::FormatBorder::Thin);

    let yes_format = Format::new()
        .set_background_color(Color::RGB(0xC6EFCE))
        .set_font_color(Color::RGB(0x006100))
        .set_align(rust_xlsxwriter::FormatAlign::Center);

    let no_format = Format::new()
        .set_background_color(Color::RGB(0xFFC7CE))
        .set_font_color(Color::RGB(0x9C0006))
        .set_align(rust_xlsxwriter::FormatAlign::Center);

    worksheet.write_string_with_format(0, 0, format!("Student Submission Report - {} (Year {})", specialization, academic_year), &header_format)
        .map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.merge_range(0, 0, 0, 6, "", &Format::new())
        .map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

    let headers = ["RA Number", "Student Name", "Specialization", "Academic Year", "GitHub", "LinkedIn", "LeetCode"];

    for (col, &header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(2, col as u16, header, &header_format)
            .map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    }

    worksheet.set_column_width(0, 18).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(1, 30).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(2, 40).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(3, 15).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(4, 12).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(5, 12).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(6, 12).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

    for (idx, student) in students.iter().enumerate() {
        let row = (idx + 3) as u32;
        worksheet.write_string(row, 0, &student.registration_number).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_string(row, 1, &student.full_name).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_string(row, 2, student.specialization.as_deref().unwrap_or("N/A")).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_string(row, 3, student.academic_year.as_deref().unwrap_or("N/A")).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

        let github_status = if student.github_submitted { "✓ Yes" } else { "✗ No" };
        let github_fmt = if student.github_submitted { &yes_format } else { &no_format };
        worksheet.write_string_with_format(row, 4, github_status, github_fmt).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

        let linkedin_status = if student.linkedin_submitted { "✓ Yes" } else { "✗ No" };
        let linkedin_fmt = if student.linkedin_submitted { &yes_format } else { &no_format };
        worksheet.write_string_with_format(row, 5, linkedin_status, linkedin_fmt).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

        let leetcode_status = if student.leetcode_submitted { "✓ Yes" } else { "✗ No" };
        let leetcode_fmt = if student.leetcode_submitted { &yes_format } else { &no_format };
        worksheet.write_string_with_format(row, 6, leetcode_status, leetcode_fmt).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    }

    let summary_row = (students.len() + 5) as u32;
    let total_students = students.len();
    let github_count = students.iter().filter(|s| s.github_submitted).count();
    let linkedin_count = students.iter().filter(|s| s.linkedin_submitted).count();
    let leetcode_count = students.iter().filter(|s| s.leetcode_submitted).count();

    worksheet.write_string_with_format(summary_row, 0, "Summary:", &header_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.write_string(summary_row + 1, 0, "Total Students:").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.write_number(summary_row + 1, 1, total_students as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

    worksheet.write_string(summary_row + 2, 0, "GitHub Submitted:").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.write_number(summary_row + 2, 1, github_count as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    if total_students > 0 {
        worksheet.write_string(summary_row + 2, 2, format!("({:.1}%)", (github_count as f64 / total_students as f64) * 100.0)).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    }

    worksheet.write_string(summary_row + 3, 0, "LinkedIn Submitted:").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.write_number(summary_row + 3, 1, linkedin_count as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    if total_students > 0 {
        worksheet.write_string(summary_row + 3, 2, format!("({:.1}%)", (linkedin_count as f64 / total_students as f64) * 100.0)).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    }

    worksheet.write_string(summary_row + 4, 0, "LeetCode Submitted:").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.write_number(summary_row + 4, 1, leetcode_count as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    if total_students > 0 {
        worksheet.write_string(summary_row + 4, 2, format!("({:.1}%)", (leetcode_count as f64 / total_students as f64) * 100.0)).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    }

    workbook.save(filepath).map_err(|e| ApiError::InternalServerError(format!("Failed to save Excel file: {}", e)))?;
    Ok(())
}

fn generate_all_years_excel_report(
    students: &[StudentSubmissionData],
    specialization: &str,
    filepath: &str,
) -> Result<(), ApiError> {
    let mut workbook = Workbook::new();
    let years = vec!["I", "II", "III", "IV"];

    let header_format = Format::new().set_bold().set_background_color(Color::RGB(0x4472C4)).set_font_color(Color::White).set_border(rust_xlsxwriter::FormatBorder::Thin);
    let yes_format = Format::new().set_background_color(Color::RGB(0xC6EFCE)).set_font_color(Color::RGB(0x006100)).set_align(rust_xlsxwriter::FormatAlign::Center);
    let no_format = Format::new().set_background_color(Color::RGB(0xFFC7CE)).set_font_color(Color::RGB(0x9C0006)).set_align(rust_xlsxwriter::FormatAlign::Center);
    let headers = ["RA Number", "Student Name", "Specialization", "Academic Year", "GitHub", "LinkedIn", "LeetCode"];

    for &year in &years {
        let year_students: Vec<_> = students.iter().filter(|s| match &s.academic_year { Some(y) => y == year, None => false }).collect();
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(year).map_err(|e| ApiError::InternalServerError(format!("Failed to set worksheet name: {}", e)))?;

        worksheet.write_string_with_format(0, 0, format!("Student Submission Report - {} (Year {})", specialization, year), &header_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.merge_range(0, 0, 0, 6, "", &Format::new()).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

        for (col, &header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(2, col as u16, header, &header_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        }

        worksheet.set_column_width(0, 18).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(1, 30).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(2, 40).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(3, 15).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(4, 12).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(5, 12).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(6, 12).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

        for (idx, student) in year_students.iter().enumerate() {
            let row = (idx + 3) as u32;
            worksheet.write_string(row, 0, &student.registration_number).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
            worksheet.write_string(row, 1, &student.full_name).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
            worksheet.write_string(row, 2, student.specialization.as_deref().unwrap_or("N/A")).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
            worksheet.write_string(row, 3, student.academic_year.as_deref().unwrap_or("N/A")).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

            let github_fmt = if student.github_submitted { &yes_format } else { &no_format };
            worksheet.write_string_with_format(row, 4, if student.github_submitted { "✓ Yes" } else { "✗ No" }, github_fmt).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

            let linkedin_fmt = if student.linkedin_submitted { &yes_format } else { &no_format };
            worksheet.write_string_with_format(row, 5, if student.linkedin_submitted { "✓ Yes" } else { "✗ No" }, linkedin_fmt).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

            let leetcode_fmt = if student.leetcode_submitted { &yes_format } else { &no_format };
            worksheet.write_string_with_format(row, 6, if student.leetcode_submitted { "✓ Yes" } else { "✗ No" }, leetcode_fmt).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        }

        let summary_row = (year_students.len() + 5) as u32;
        let total = year_students.len();
        let github_count = year_students.iter().filter(|s| s.github_submitted).count();
        let linkedin_count = year_students.iter().filter(|s| s.linkedin_submitted).count();
        let leetcode_count = year_students.iter().filter(|s| s.leetcode_submitted).count();

        worksheet.write_string_with_format(summary_row, 0, "Summary:", &header_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_string(summary_row + 1, 0, "Total Students:").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_number(summary_row + 1, 1, total as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_string(summary_row + 2, 0, "GitHub Submitted:").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_number(summary_row + 2, 1, github_count as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        if total > 0 { worksheet.write_string(summary_row + 2, 2, format!("({:.1}%)", (github_count as f64 / total as f64) * 100.0)).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?; }
        worksheet.write_string(summary_row + 3, 0, "LinkedIn Submitted:").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_number(summary_row + 3, 1, linkedin_count as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        if total > 0 { worksheet.write_string(summary_row + 3, 2, format!("({:.1}%)", (linkedin_count as f64 / total as f64) * 100.0)).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?; }
        worksheet.write_string(summary_row + 4, 0, "LeetCode Submitted:").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_number(summary_row + 4, 1, leetcode_count as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        if total > 0 { worksheet.write_string(summary_row + 4, 2, format!("({:.1}%)", (leetcode_count as f64 / total as f64) * 100.0)).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?; }
    }

    workbook.save(filepath).map_err(|e| ApiError::InternalServerError(format!("Failed to save Excel file: {}", e)))?;
    Ok(())
}

// ============================================================================
// EXCEL GENERATION: DEFAULTERS REPORT
// ============================================================================

fn generate_defaulters_report_to_file(
    students: &[StudentDefaulterData],
    specialization: &str,
    academic_year: &str,
    filepath: &str,
) -> Result<(), ApiError> {
    // Year V or VI both get multi-year reports
    if academic_year == "V" || academic_year == "VI" {
        return generate_all_years_defaulters_report(students, specialization, filepath);
    }
    generate_single_year_defaulters_report(students, specialization, academic_year, filepath)
}
fn generate_single_year_defaulters_report(
    students: &[StudentDefaulterData],
    specialization: &str,
    academic_year: &str,
    filepath: &str,
) -> Result<(), ApiError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let header_format = Format::new().set_bold().set_background_color(Color::RGB(0x4472C4)).set_font_color(Color::White).set_border(rust_xlsxwriter::FormatBorder::Thin);
    let pass_format = Format::new().set_background_color(Color::RGB(0xC6EFCE)).set_font_color(Color::RGB(0x006100)).set_align(rust_xlsxwriter::FormatAlign::Center);
    let fail_format = Format::new().set_background_color(Color::RGB(0xFFC7CE)).set_font_color(Color::RGB(0x9C0006)).set_align(rust_xlsxwriter::FormatAlign::Center);
    let number_format = Format::new().set_align(rust_xlsxwriter::FormatAlign::Center);

    worksheet.write_string_with_format(0, 0, format!("Defaulters Report - {} (Year {}) - Minimum: 15 Questions (All Platforms)", specialization, academic_year), &header_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.merge_range(0, 0, 0, 8, "", &Format::new()).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

    let headers = ["Registration Number", "Full Name", "Specialization", "Academic Year", "LeetCode (30d)", "CodeChef (30d)", "Codeforces (30d)", "Total (30d)", "Status"];
    for (col, &header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(2, col as u16, header, &header_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    }

    worksheet.set_column_width(0, 18).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(1, 30).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(2, 40).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(3, 15).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(4, 16).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(5, 16).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(6, 16).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(7, 16).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.set_column_width(8, 15).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

    let mut defaulter_count = 0;
    for (idx, student) in students.iter().enumerate() {
        let row = (idx + 3) as u32;
        let total = student.total_solved_last_30_days.unwrap_or(student.leetcode_solved_last_30_days + student.codechef_solved_last_30_days.unwrap_or(0) + student.codeforces_solved_last_30_days.unwrap_or(0));
        let is_defaulter = total < 15;
        if is_defaulter { defaulter_count += 1; }

        worksheet.write_string(row, 0, &student.registration_number).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_string(row, 1, &student.full_name).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_string(row, 2, student.specialization.as_deref().unwrap_or("N/A")).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_string(row, 3, student.academic_year.as_deref().unwrap_or("N/A")).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_number_with_format(row, 4, student.leetcode_solved_last_30_days as f64, &number_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_number_with_format(row, 5, student.codechef_solved_last_30_days.unwrap_or(0) as f64, &number_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_number_with_format(row, 6, student.codeforces_solved_last_30_days.unwrap_or(0) as f64, &number_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

        let bold_format = Format::new().set_align(rust_xlsxwriter::FormatAlign::Center).set_bold();
        worksheet.write_number_with_format(row, 7, total as f64, &bold_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

        let status_fmt = if is_defaulter { &fail_format } else { &pass_format };
        worksheet.write_string_with_format(row, 8, if is_defaulter { "✗ Defaulter" } else { "✓ Pass" }, status_fmt).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    }

    let summary_row = (students.len() + 5) as u32;
    let total = students.len();
    let pass_count = total - defaulter_count;

    worksheet.write_string_with_format(summary_row, 0, "Summary:", &header_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.write_string(summary_row + 1, 0, "Total Students:").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.write_number(summary_row + 1, 1, total as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.write_string(summary_row + 2, 0, "Defaulters (<15 questions):").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.write_number(summary_row + 2, 1, defaulter_count as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    if total > 0 { worksheet.write_string(summary_row + 2, 2, format!("({:.1}%)", (defaulter_count as f64 / total as f64) * 100.0)).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?; }
    worksheet.write_string(summary_row + 3, 0, "Pass (≥15 questions):").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    worksheet.write_number(summary_row + 3, 1, pass_count as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
    if total > 0 { worksheet.write_string(summary_row + 3, 2, format!("({:.1}%)", (pass_count as f64 / total as f64) * 100.0)).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?; }

    workbook.save(filepath).map_err(|e| ApiError::InternalServerError(format!("Failed to save Excel file: {}", e)))?;
    Ok(())
}

fn generate_all_years_defaulters_report(
    students: &[StudentDefaulterData],
    specialization: &str,
    filepath: &str,
) -> Result<(), ApiError> {
    let mut workbook = Workbook::new();
    let years = vec!["I", "II", "III", "IV"];

    let header_format = Format::new().set_bold().set_background_color(Color::RGB(0x4472C4)).set_font_color(Color::White).set_border(rust_xlsxwriter::FormatBorder::Thin);
    let pass_format = Format::new().set_background_color(Color::RGB(0xC6EFCE)).set_font_color(Color::RGB(0x006100)).set_align(rust_xlsxwriter::FormatAlign::Center);
    let fail_format = Format::new().set_background_color(Color::RGB(0xFFC7CE)).set_font_color(Color::RGB(0x9C0006)).set_align(rust_xlsxwriter::FormatAlign::Center);
    let number_format = Format::new().set_align(rust_xlsxwriter::FormatAlign::Center);
    let headers = ["Registration Number", "Full Name", "Specialization", "Academic Year", "LeetCode (30d)", "CodeChef (30d)", "Codeforces (30d)", "Total (30d)", "Status"];

    for &year in &years {
        let year_students: Vec<_> = students.iter().filter(|s| match &s.academic_year { Some(y) => y == year, None => false }).collect();
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(year).map_err(|e| ApiError::InternalServerError(format!("Failed to set worksheet name: {}", e)))?;

        worksheet.write_string_with_format(0, 0, format!("Defaulters Report - {} (Year {}) - Minimum: 15 Questions (All Platforms)", specialization, year), &header_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.merge_range(0, 0, 0, 8, "", &Format::new()).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

        for (col, &header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(2, col as u16, header, &header_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        }

        worksheet.set_column_width(0, 18).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(1, 30).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(2, 40).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(3, 15).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(4, 16).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(5, 16).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(6, 16).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(7, 16).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.set_column_width(8, 15).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

        let mut defaulter_count = 0;
        for (idx, student) in year_students.iter().enumerate() {
            let row = (idx + 3) as u32;
            let total = student.total_solved_last_30_days.unwrap_or(student.leetcode_solved_last_30_days + student.codechef_solved_last_30_days.unwrap_or(0) + student.codeforces_solved_last_30_days.unwrap_or(0));
            let is_defaulter = total < 15;
            if is_defaulter { defaulter_count += 1; }

            worksheet.write_string(row, 0, &student.registration_number).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
            worksheet.write_string(row, 1, &student.full_name).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
            worksheet.write_string(row, 2, student.specialization.as_deref().unwrap_or("N/A")).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
            worksheet.write_string(row, 3, student.academic_year.as_deref().unwrap_or("N/A")).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
            worksheet.write_number_with_format(row, 4, student.leetcode_solved_last_30_days as f64, &number_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
            worksheet.write_number_with_format(row, 5, student.codechef_solved_last_30_days.unwrap_or(0) as f64, &number_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
            worksheet.write_number_with_format(row, 6, student.codeforces_solved_last_30_days.unwrap_or(0) as f64, &number_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

            let bold_format = Format::new().set_align(rust_xlsxwriter::FormatAlign::Center).set_bold();
            worksheet.write_number_with_format(row, 7, total as f64, &bold_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;

            let status_fmt = if is_defaulter { &fail_format } else { &pass_format };
            worksheet.write_string_with_format(row, 8, if is_defaulter { "✗ Defaulter" } else { "✓ Pass" }, status_fmt).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        }

        let summary_row = (year_students.len() + 5) as u32;
        let total = year_students.len();
        let pass_count = total - defaulter_count;

        worksheet.write_string_with_format(summary_row, 0, "Summary:", &header_format).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_string(summary_row + 1, 0, "Total Students:").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_number(summary_row + 1, 1, total as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_string(summary_row + 2, 0, "Defaulters (<15 questions):").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_number(summary_row + 2, 1, defaulter_count as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        if total > 0 { worksheet.write_string(summary_row + 2, 2, format!("({:.1}%)", (defaulter_count as f64 / total as f64) * 100.0)).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?; }
        worksheet.write_string(summary_row + 3, 0, "Pass (≥15 questions):").map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        worksheet.write_number(summary_row + 3, 1, pass_count as f64).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?;
        if total > 0 { worksheet.write_string(summary_row + 3, 2, format!("({:.1}%)", (pass_count as f64 / total as f64) * 100.0)).map_err(|e| ApiError::InternalServerError(format!("Excel error: {}", e)))?; }
    }

    workbook.save(filepath).map_err(|e| ApiError::InternalServerError(format!("Failed to save Excel file: {}", e)))?;
    Ok(())
}