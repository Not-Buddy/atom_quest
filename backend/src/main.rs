mod api;
mod config;
mod cors;
mod db;
mod logging;
mod utils;

use api::auth::{AppState, ForgotPasswordRateLimiter, login, me, forgot_password, reset_password, reset_password_form_handler};
use axum::{
    extract::Request,
    middleware::{self, Next},
    routing::get,
    Router,
};
use config::Config;
use db::Database;
use logging::FileLogger;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use std::fs::OpenOptions;

use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing::Subscriber;

struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let level = event.metadata().level();

        write!(writer, "[{}] [{}] ", timestamp, level)?;
        _ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

#[tokio::main]
async fn main() {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log_path = format!("logs/{}.log", date);

    std::fs::create_dir_all("logs").expect("Failed to create logs directory");

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Failed to open log file for tracing");

    let (non_blocking, _guard) = tracing_appender::non_blocking(log_file);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .event_format(CustomFormatter)
        .init();

    std::mem::forget(_guard);

    let file_logger = Arc::new(
        FileLogger::new().expect("Failed to initialize file logger")
    );

    let config = Config::from_env();

    tracing::info!("AtomQuest Backend starting...");
    let _ = file_logger.log("AtomQuest Backend starting...");

    let db = Database::new(&config.database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to database");
    let _ = file_logger.log_with_level("INFO", "Connected to database");

    // Run migrations
    if let Err(e) = run_migrations(&db).await {
        tracing::error!("Migration error: {}", e);
        let _ = file_logger.log_with_level("ERROR", &format!("Migration error: {}", e));
    } else {
        tracing::info!("Migrations complete");
        let _ = file_logger.log_with_level("INFO", "Migrations complete");
    }

    // Seed admin user if configured
    if let Some(ref admin_password) = config.admin_seed_password {
        seed_admin_user(&db, admin_password).await;
    }

    let forgot_password_rate_limiter: ForgotPasswordRateLimiter =
        Arc::new(Mutex::new(HashMap::new()));

    let state = Arc::new(AppState {
        db,
        config: config.clone(),
        forgot_password_rate_limiter,
    });

    let config_arc = Arc::new(config.clone());
    let logger_for_middleware = file_logger.clone();

    // Public routes
    let public_routes = Router::new()
        .route("/", get(root_handler))
        .route("/auth/login", axum::routing::post(login))
        .route("/auth/forgot-password", axum::routing::post(forgot_password))
        .route("/auth/reset-password", axum::routing::post(reset_password))
        .route("/auth/reset-password-form", axum::routing::post(reset_password_form_handler));

    // Protected routes - employee
    let employee_routes = Router::new()
        .route("/auth/me", get(me))
        .route("/goals/sheets", axum::routing::post(api::goals::create_goal_sheet))
        .route("/goals/sheets", get(api::goals::list_goal_sheets))
        .route("/goals/sheets/{id}", get(api::goals::get_sheet_detail))
        .route("/goals/sheets/{id}/submit", axum::routing::put(api::goals::submit_sheet))
        .route("/goals/sheets/{id}/goals", axum::routing::post(api::goals::add_goal_to_sheet))
        .route("/goals/{id}", axum::routing::put(api::goals::update_goal))
        .route("/goals/{id}", axum::routing::delete(api::goals::delete_goal))
        .route("/achievements/sheet/{id}", get(api::achievements::get_achievements_for_sheet))
        .route("/achievements/{goal_id}/{quarter}", axum::routing::put(api::achievements::update_achievement))
        .layer(middleware::from_fn_with_state(
            config_arc.clone(),
            api::middleware::auth_middleware,
        ));

    // Protected routes - manager
    let manager_routes = Router::new()
        .route("/manager/team/sheets", get(api::manager::list_team_sheets))
        .route("/manager/sheets/{id}/approve", axum::routing::put(api::manager::approve_sheet))
        .route("/manager/sheets/{id}/return", axum::routing::put(api::manager::return_sheet))
        .route("/manager/sheets/{sheet_id}/goals/{goal_id}", axum::routing::put(api::manager::edit_goal))
        .route("/manager/shared-goals", axum::routing::post(api::manager::push_shared_goal))
        .route("/manager/team/checkins", get(api::manager::view_team_checkins))
        .route("/manager/checkins/{sheet_id}", axum::routing::post(api::manager::add_checkin_comment))
        .layer(middleware::from_fn_with_state(
            config_arc.clone(),
            api::middleware::auth_middleware,
        ));

    // Admin routes
    let admin_routes = Router::new()
        .route("/admin/cycles", get(api::admin::list_cycles))
        .route("/admin/cycles", axum::routing::post(api::admin::create_cycle))
        .route("/admin/cycles/{id}", axum::routing::put(api::admin::update_cycle))
        .route("/admin/departments", get(api::admin::list_departments))
        .route("/admin/departments", axum::routing::post(api::admin::create_department))
        .route("/admin/thrust-areas", get(api::admin::list_thrust_areas))
        .route("/admin/thrust-areas", axum::routing::post(api::admin::create_thrust_area))
        .route("/admin/users", get(api::admin::list_users))
        .route("/admin/users", axum::routing::post(api::admin::create_user))
        .route("/admin/users/{id}", axum::routing::put(api::admin::update_user))
        .route("/admin/users/{id}", axum::routing::delete(api::admin::delete_user))
        .route("/admin/sheets/{id}/unlock", axum::routing::put(api::admin::unlock_sheet))
        .route("/admin/audit-log", get(api::admin::view_audit_log))
        .layer(middleware::from_fn_with_state(
            config_arc.clone(),
            api::middleware::auth_middleware,
        ));

    // Reports routes (manager + admin)
    let reports_routes = Router::new()
        .route("/reports/achievement", get(api::reports::achievement_report))
        .route("/reports/completion-dashboard", get(api::reports::completion_dashboard))
        .layer(middleware::from_fn_with_state(
            config_arc.clone(),
            api::middleware::auth_middleware,
        ));

    let app = Router::new()
        .route("/health", get(health_check))
        .merge(public_routes)
        .merge(employee_routes)
        .merge(manager_routes)
        .merge(admin_routes)
        .merge(reports_routes)
        .layer(cors::create_cors_layer())
        .layer(middleware::from_fn(move |req: Request, next: Next| {
            let logger = logger_for_middleware.clone();
            async move {
                let method = req.method().clone();
                let uri = req.uri().clone();
                let path = uri.path().to_string();
                let start = Instant::now();

                let (parts, body) = req.into_parts();
                let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap_or_default();
                tracing::debug!("Request body: {}", String::from_utf8_lossy(&bytes));
                let req = Request::from_parts(parts, axum::body::Body::from(bytes));

                let response = next.run(req).await;

                let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
                let status = response.status();

                let _ = logger.log_request(
                    method.as_str(),
                    &path,
                    status.as_u16(),
                    duration_ms
                );

                tracing::info!(
                    "{} {} - {} ({:.0}ms)",
                    method,
                    path,
                    status.as_u16(),
                    duration_ms
                );

                response
            }
        }))
        .with_state(state);

    tracing::info!("Binding to 0.0.0.0:3000...");
    let _ = file_logger.log("Binding to 0.0.0.0:3000...");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    let addr = listener.local_addr().unwrap();
    tracing::info!("Server listening on {}", addr);
    let _ = file_logger.log_with_level("INFO", &format!("Server listening on {}", addr));

    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> &'static str {
    "AtomQuest Backend is UP and RUNNING"
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "message": "AtomQuest Backend is UP and RUNNING"
    }))
}

async fn run_migrations(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let sql = include_str!("../migrations/001_core_schema.sql");

    // Split by semicolons and execute each statement
    for statement in sql.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }
        sqlx::query(trimmed)
            .execute(&db.pool)
            .await?;
    }

    Ok(())
}

async fn seed_admin_user(db: &Database, password: &str) {
    use crate::utils::hash_password;
    use crate::db::users;

    let existing = users::find_by_email(&db.pool, "admin@atomquest.com").await;
    match existing {
        Ok(Some(_)) => {
            tracing::info!("Admin user already exists");
        }
        _ => {
            let password_hash = hash_password::hash(password);
            match users::create_user(
                &db.pool,
                "admin@atomquest.com",
                &password_hash,
                "System Admin",
                None,
                "admin",
                None,
            ).await {
                Ok(_) => tracing::info!("Admin user seeded successfully"),
                Err(e) => tracing::error!("Failed to seed admin user: {}", e),
            }
        }
    }
}
