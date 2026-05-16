mod api;
mod config;
mod cors;
mod db;
mod leetcode;
mod codechef;
mod codeforces;
mod logging;
mod tui;
mod utils;

use api::auth::{login, me, AppState, ForgotPasswordRateLimiter};
use axum::{
    extract::Request,
    middleware::{self, Next},
    routing::get,
    Router,
};
use config::Config;
use db::Database;
use logging::FileLogger;
use std::{collections::HashMap, sync::{Arc, Mutex}};
use std::time::Instant;
use tokio::sync::mpsc;
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

    let config = Config::from_env().expect("Failed to load configuration");

    let (tui_tx, tui_rx) = mpsc::unbounded_channel();
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let action_tx_clone = action_tx.clone();
    let tui_handle = tokio::spawn(async move {
        if let Err(e) = tui::run_tui(tui_rx, action_tx_clone).await {
            eprintln!("TUI error: {}", e);
        }
    });

    let _ = tui_tx.send(tui::TuiUpdate::Log("TUI Started".to_string()));
    let _ = file_logger.log("TUI Started");

    let mut server_handle: Option<tokio::task::JoinHandle<()>> = None;

    loop {
        if let Some(action) = action_rx.recv().await {
            match action {
                tui::AppAction::StartServer => {
                    if server_handle.is_none() {
                        let _ = tui_tx.send(tui::TuiUpdate::ServerStatus(tui::ServerStatus::Starting));
                        let _ = tui_tx.send(tui::TuiUpdate::Log("Starting server...".to_string()));
                        let _ = file_logger.log("Starting server...");

                        let tui_tx_clone = tui_tx.clone();
                        let config_clone = config.clone();
                        let logger_clone = file_logger.clone();
                        
                        let handle = tokio::spawn(async move {
                            match start_server(tui_tx_clone.clone(), config_clone, logger_clone).await {
                                Ok(_) => {
                                    let _ = tui_tx_clone.send(tui::TuiUpdate::Log("Server stopped cleanly".to_string()));
                                }
                                Err(e) => {
                                    let msg = format!("Server error: {}", e);
                                    let _ = tui_tx_clone.send(tui::TuiUpdate::Log(msg.clone()));
                                    let _ = tui_tx_clone.send(tui::TuiUpdate::ServerStatus(tui::ServerStatus::Stopped));
                                }
                            }
                        });

                        server_handle = Some(handle);
                    }
                }
                tui::AppAction::StopServer => {
                    if let Some(handle) = server_handle.take() {
                        let _ = tui_tx.send(tui::TuiUpdate::ServerStatus(tui::ServerStatus::Stopping));
                        let _ = tui_tx.send(tui::TuiUpdate::Log("Stopping server...".to_string()));
                        let _ = file_logger.log("Stopping server...");
                        handle.abort();
                        let _ = tui_tx.send(tui::TuiUpdate::ServerStatus(tui::ServerStatus::Stopped));
                        let _ = tui_tx.send(tui::TuiUpdate::Log("Server stopped".to_string()));
                        let _ = file_logger.log("Server stopped");
                    }
                }
                tui::AppAction::BackupDatabase => {
                    let _ = tui_tx.send(tui::TuiUpdate::Log("Starting database backup...".to_string()));
                    let _ = file_logger.log("Starting database backup...");
                    
                    let tui_tx_clone = tui_tx.clone();
                    let logger_clone = file_logger.clone();
                    let config_clone = config.clone();
                    
                    tokio::spawn(async move {
                        match Database::new(&config_clone.database_url).await {
                            Ok(db) => {
                                match db::backup::backup_database(&db.pool, &config_clone.database_url).await {
                                    Ok(backup_file) => {
                                        let msg = format!("✓ Database backed up to: {}", backup_file);
                                        let _ = tui_tx_clone.send(tui::TuiUpdate::Log(msg.clone()));
                                        let _ = logger_clone.log_with_level("INFO", &msg);
                                    }
                                    Err(e) => {
                                        let msg = format!("❌ Backup failed: {}", e);
                                        let _ = tui_tx_clone.send(tui::TuiUpdate::Log(msg.clone()));
                                        let _ = logger_clone.log_with_level("ERROR", &msg);
                                    }
                                }
                            }
                            Err(e) => {
                                let msg = format!("❌ Database connection failed: {}", e);
                                let _ = tui_tx_clone.send(tui::TuiUpdate::Log(msg.clone()));
                                let _ = logger_clone.log_with_level("ERROR", &msg);
                            }
                        }
                    });
                }
                tui::AppAction::CreateTables => {
                    let _ = tui_tx.send(tui::TuiUpdate::Log("Creating STUDENTS table...".to_string()));
                    let _ = file_logger.log("Creating STUDENTS table...");
                    
                    let tui_tx_clone = tui_tx.clone();
                    let logger_clone = file_logger.clone();
                    let config_clone = config.clone();
                    
                    tokio::spawn(async move {
                        match Database::new(&config_clone.database_url).await {
                            Ok(db) => {
                                match db::backup::create_students_table(&db.pool).await {
                                    Ok(_) => {
                                        let msg = "✓ STUDENTS table created successfully";
                                        let _ = tui_tx_clone.send(tui::TuiUpdate::Log(msg.to_string()));
                                        let _ = logger_clone.log_with_level("INFO", msg);
                                    }
                                    Err(e) => {
                                        let msg = format!("❌ Table creation failed: {}", e);
                                        let _ = tui_tx_clone.send(tui::TuiUpdate::Log(msg.clone()));
                                        let _ = logger_clone.log_with_level("ERROR", &msg);
                                    }
                                }
                            }
                            Err(e) => {
                                let msg = format!("❌ Database connection failed: {}", e);
                                let _ = tui_tx_clone.send(tui::TuiUpdate::Log(msg.clone()));
                                let _ = logger_clone.log_with_level("ERROR", &msg);
                            }
                        }
                    });
                }
                tui::AppAction::ViewLogs => {
                    let _ = tui_tx.send(tui::TuiUpdate::Log("Viewing logs...".to_string()));
                }
                tui::AppAction::ImportData => {
                    let _ = tui_tx.send(tui::TuiUpdate::Log("Importing students from data/ ...".to_string()));
                    let _ = file_logger.log("Importing students from data/ ...");

                    let tui_tx_clone = tui_tx.clone();
                    let logger_clone = file_logger.clone();
                    let config_clone = config.clone();

                    tokio::spawn(async move {
                        match Database::new(&config_clone.database_url).await {
                            Ok(db) => {
                                match db::import::import_students_from_data_dir(&db.pool).await {
                                    Ok(count) => {
                                        let msg = format!("✓ Import completed. Rows affected: {}", count);
                                        let _ = tui_tx_clone.send(tui::TuiUpdate::Log(msg.clone()));
                                        let _ = logger_clone.log_with_level("INFO", &msg);
                                    }
                                    Err(e) => {
                                        let msg = format!("❌ Import failed: {}", e);
                                        let _ = tui_tx_clone.send(tui::TuiUpdate::Log(msg.clone()));
                                        let _ = logger_clone.log_with_level("ERROR", &msg);
                                    }
                                }
                            }
                            Err(e) => {
                                let msg = format!("❌ Database connection failed: {}", e);
                                let _ = tui_tx_clone.send(tui::TuiUpdate::Log(msg.clone()));
                                let _ = logger_clone.log_with_level("ERROR", &msg);
                            }
                        }
                    });
                }
                tui::AppAction::Quit => {
                    if let Some(handle) = server_handle.take() {
                        handle.abort();
                    }
                    let _ = file_logger.log("Application quit");
                    break;
                }
            }
        }
    }

    tui_handle.abort();
}


async fn root_handler() -> &'static str {
    "Rust Backend is UP and RUNNING"
}

async fn start_server(
    tui_tx: mpsc::UnboundedSender<tui::TuiUpdate>,
    config: Config,
    logger: Arc<FileLogger>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = tui_tx.send(tui::TuiUpdate::Log("Connecting to database...".to_string()));
    let _ = logger.log("Connecting to database...");

    let db = Database::new(&config.database_url).await?;

    let _ = tui_tx.send(tui::TuiUpdate::Log("✓ Connected to database".to_string()));
    let _ = logger.log_with_level("INFO", "Connected to database");

    let _ = tui_tx.send(tui::TuiUpdate::Log("Skipping migrations".to_string()));
    let _ = logger.log("Skipping migrations");

    let pool = Arc::new(db.pool.clone());

    // Create channels for the LeetCode worker
    let (leetcode_sync_tx, leetcode_sync_rx) = mpsc::channel(100);
    let (leetcode_priority_sync_tx, leetcode_priority_sync_rx) = mpsc::channel(100);

    // Create channels for the CodeChef worker
    let (codechef_sync_tx, codechef_sync_rx) = mpsc::channel(100);
    let (codechef_priority_sync_tx, codechef_priority_sync_rx) = mpsc::channel(100);

    // Create channels for the Codeforces worker
    let (codeforces_sync_tx, codeforces_sync_rx) = mpsc::channel(100);
    let (codeforces_priority_sync_tx, codeforces_priority_sync_rx) = mpsc::channel(100);

    // Start the LeetCode background worker
    let leetcode_worker = leetcode::worker::BackgroundWorker::new(
        pool.clone(),
        leetcode_sync_rx,
        leetcode_priority_sync_rx,
        leetcode_sync_tx.clone(),
        leetcode_priority_sync_tx.clone(),
        logger.clone(),
    );

    tokio::spawn(async move {
        leetcode_worker.run().await;
    });

    // Start the CodeChef background worker
    let codechef_worker = codechef::worker::BackgroundWorker::new(
        pool.clone(),
        codechef_sync_rx,
        codechef_priority_sync_rx,
        codechef_sync_tx.clone(),
        codechef_priority_sync_tx.clone(),
        logger.clone(),
    );

    tokio::spawn(async move {
        codechef_worker.run().await;
    });

    // Start the Codeforces background worker
    let codeforces_worker = codeforces::worker::BackgroundWorker::new(
        pool.clone(),
        codeforces_sync_rx,
        codeforces_priority_sync_rx,
        codeforces_sync_tx.clone(),
        codeforces_priority_sync_tx.clone(),
        logger.clone(),
    );

    tokio::spawn(async move {
        codeforces_worker.run().await;
    });

    // Create the sync queues to be used by the API
    let leetcode_sync_queue = leetcode::worker::SyncQueue::new(
        leetcode_sync_tx.clone(),
        leetcode_priority_sync_tx.clone(),
        logger.clone()
    );

    let codechef_sync_queue = codechef::worker::SyncQueue::new(
        codechef_sync_tx.clone(),
        codechef_priority_sync_tx.clone(),
        logger.clone()
    );

    let codeforces_sync_queue = codeforces::worker::SyncQueue::new(
        codeforces_sync_tx.clone(),
        codeforces_priority_sync_tx.clone(),
        logger.clone()
    );

    let forgot_password_rate_limiter: ForgotPasswordRateLimiter =
    Arc::new(Mutex::new(HashMap::new()));


    let state = Arc::new(AppState {
        db,
        config: config.clone(),
        leetcode_sync_queue,
        codechef_sync_queue,
        codeforces_sync_queue,
        forgot_password_rate_limiter,
    });

    let config_arc = Arc::new(config);

    // Clone logger and tui_tx for request logging middleware
    let logger_for_middleware = logger.clone();
    let tui_tx_for_middleware = tui_tx.clone();

    // Public routes (NO authentication required)
    let public_routes = Router::new()
        .route("/", get(root_handler))
        .route("/leaderboard", get(api::leaderboard::get_leaderboard))
        .route("/auth/login", axum::routing::post(login))
        .route("/auth/forgot-password", axum::routing::post(api::auth::forgot_password))
        .route("/auth/reset-password", axum::routing::post(api::auth::reset_password))
        .route("/auth/reset-password-form", axum::routing::post(api::auth::reset_password_form_handler))
        .route("/faculty/login", axum::routing::post(api::faculty_auth::faculty_login));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/auth/me", get(me))
        .route("/faculty/me", get(api::faculty_auth::faculty_me))
        .route("/profile/links", get(api::auth::get_profile_links))
        .route("/profile/links", axum::routing::post(api::auth::update_profile_links))
        // .route("/students", get(api::students::get_students))
        .route("/faculty/reports/submissions", get(api::faculty_reports::get_student_submissions_report))
        .route("/faculty/reports/defaulters", get(api::faculty_reports::get_defaulters_report))
        .route("/faculty/stats", get(api::stats::get_stats))
        .layer(middleware::from_fn_with_state(
            config_arc.clone(),
            api::middleware::auth_middleware,
        ));

    let app = Router::new()
        .route("/health", get(health_check))
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors::create_cors_layer())
        .layer(middleware::from_fn(move |req: Request, next: Next| {
            let logger = logger_for_middleware.clone();
            let tui_tx = tui_tx_for_middleware.clone();
            async move {
                let method = req.method().clone();
                let uri = req.uri().clone();
                let path = uri.path().to_string();
                let start = Instant::now();

                // Read the body
                let (parts, body) = req.into_parts();
                let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap_or_default();
                tracing::debug!("Request body: {}", String::from_utf8_lossy(&bytes));
                let req = Request::from_parts(parts, axum::body::Body::from(bytes));


                let response = next.run(req).await;

                let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
                let status = response.status();

                // Log to file
                let _ = logger.log_request(
                    method.as_str(),
                    &path,
                    status.as_u16(),
                    duration_ms
                );

                // Log to TUI
                let log_msg = format!(
                    "{} {} - {} ({:.0}ms)",
                    method, path, status.as_u16(), duration_ms
                );
                let _ = tui_tx.send(tui::TuiUpdate::Log(log_msg));

                response
            }
        }))
        .with_state(state);

    let _ = tui_tx.send(tui::TuiUpdate::Log("Binding to 0.0.0.0:3000...".to_string()));
    let _ = logger.log("Binding to 0.0.0.0:3000...");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    let addr = listener.local_addr()?;

    let msg = format!("✓ Server listening on {}", addr);
    let _ = tui_tx.send(tui::TuiUpdate::Log(msg.clone()));
    let _ = logger.log_with_level("INFO", &msg);
    let _ = tui_tx.send(tui::TuiUpdate::ServerStatus(tui::ServerStatus::Running));

    axum::serve(listener, app).await?;

    Ok(())
}



async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "message": "Rust Backend is UP and RUNNING"
    }))
}
