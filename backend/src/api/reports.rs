use crate::{
    api::ApiError,
    db::{
        goals,
        models::{AchievementReportEntry, Claims, CompletionDashboardEntry},
    },
};
use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use crate::api::auth::AppState;
use axum::extract::State;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct ReportQuery {
    pub format: Option<String>,
}

pub async fn achievement_report(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Query(query): Query<ReportQuery>,
) -> Result<Response, ApiError> {
    let entries: Vec<AchievementReportEntry> =
        goals::get_achievement_report(&state.db.pool).await?;

    if query.format.as_deref() == Some("excel") {
        let xlsx_bytes = generate_excel_achievement_report(&entries)?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            )
            .header(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"achievement_report.xlsx\"",
            )
            .body(axum::body::Body::from(xlsx_bytes))
            .unwrap())
    } else {
        Ok(Json(entries).into_response())
    }
}

pub async fn completion_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Query(query): Query<ReportQuery>,
) -> Result<Response, ApiError> {
    let entries: Vec<CompletionDashboardEntry> =
        goals::get_completion_dashboard(&state.db.pool).await?;

    if query.format.as_deref() == Some("excel") {
        let xlsx_bytes = generate_excel_completion_dashboard(&entries)?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            )
            .header(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"completion_dashboard.xlsx\"",
            )
            .body(axum::body::Body::from(xlsx_bytes))
            .unwrap())
    } else {
        Ok(Json(entries).into_response())
    }
}

fn generate_excel_achievement_report(
    entries: &[AchievementReportEntry],
) -> Result<Vec<u8>, ApiError> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let headers = [
        "User Name",
        "Department",
        "Cycle",
        "Sheet Status",
        "Goal Title",
        "UOM Type",
        "Target Value",
        "Weightage (%)",
        "Q1 Actual",
        "Q1 Score",
        "Q2 Actual",
        "Q2 Score",
        "Q3 Actual",
        "Q3 Score",
        "Q4 Actual",
        "Q4 Score",
    ];

    let header_format = Format::new().set_bold();

    for (col, header) in headers.iter().enumerate() {
        worksheet
            .write_with_format(0, col as u16, *header, &header_format)
            .map_err(|e| ApiError::InternalServerError(format!("Excel write error: {}", e)))?;
    }

    for (row, entry) in entries.iter().enumerate() {
        let r = (row + 1) as u32;
        worksheet
            .write(r, 0, &entry.user_name)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 1, entry.department.as_deref().unwrap_or(""))
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 2, &entry.cycle_name)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 3, &entry.sheet_status)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 4, &entry.goal_title)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 5, &entry.uom_type)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 6, entry.target_value)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 7, entry.weightage)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 8, entry.q1_actual)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 9, entry.q1_score)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 10, entry.q2_actual)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 11, entry.q2_score)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 12, entry.q3_actual)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 13, entry.q3_score)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 14, entry.q4_actual)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 15, entry.q4_score)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    }

    workbook
        .save_to_buffer()
        .map_err(|e| ApiError::InternalServerError(format!("Excel save error: {}", e)))
}

fn generate_excel_completion_dashboard(
    entries: &[CompletionDashboardEntry],
) -> Result<Vec<u8>, ApiError> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let headers = [
        "Department",
        "Total Sheets",
        "Draft",
        "Submitted",
        "Approved",
        "Returned",
        "Locked",
    ];

    let header_format = Format::new().set_bold();

    for (col, header) in headers.iter().enumerate() {
        worksheet
            .write_with_format(0, col as u16, *header, &header_format)
            .map_err(|e| ApiError::InternalServerError(format!("Excel write error: {}", e)))?;
    }

    for (row, entry) in entries.iter().enumerate() {
        let r = (row + 1) as u32;
        worksheet
            .write(r, 0, entry.department.as_deref().unwrap_or("N/A"))
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 1, entry.total_sheets)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 2, entry.draft_count)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 3, entry.submitted_count)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 4, entry.approved_count)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 5, entry.returned_count)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        worksheet
            .write(r, 6, entry.locked_count)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    }

    workbook
        .save_to_buffer()
        .map_err(|e| ApiError::InternalServerError(format!("Excel save error: {}", e)))
}
