use std::collections::VecDeque;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AppAction {
    StartServer,
    StopServer,
    ViewLogs,
    BackupDatabase,
    CreateTables,
    ImportData,
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
}

pub struct App {
    pub menu_items: Vec<String>,
    pub selected_index: usize,
    pub server_status: ServerStatus,
    pub logs: VecDeque<String>,
    pub log_scroll: usize,
    pub action_tx: mpsc::UnboundedSender<AppAction>,
}

impl App {
    pub fn new(action_tx: mpsc::UnboundedSender<AppAction>) -> Self {
        Self {
            menu_items: vec![
                "Start Server".to_string(),
                "Stop Server".to_string(),
                "View Logs".to_string(),
                "Backup Database".to_string(),
                "Create Tables".to_string(),
                "Import Data".to_string(),
                "Quit".to_string(),
            ],
            selected_index: 0,
            server_status: ServerStatus::Stopped,
            logs: VecDeque::from(vec!["Application started".to_string()]),
            log_scroll: 0,
            action_tx,
        }
    }

    pub fn next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.menu_items.len();
    }

    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.menu_items.len() - 1;
        }
    }

    pub fn select(&mut self) -> Option<AppAction> {
        match self.selected_index {
            0 => Some(AppAction::StartServer),
            1 => Some(AppAction::StopServer),
            2 => Some(AppAction::ViewLogs),
            3 => Some(AppAction::BackupDatabase),
            4 => Some(AppAction::CreateTables),
            5 => Some(AppAction::ImportData),
            6 => Some(AppAction::Quit),
            _ => None,
        }
    }

    pub fn set_server_status(&mut self, status: ServerStatus) {
        self.server_status = status;
    }

    pub fn add_log(&mut self, log: String) {
        // Don't filter - show all logs
        // Truncate only extremely long messages
        let truncated = if log.len() > 500 {
            format!("{}...", &log[..497])
        } else {
            log
        };
        
        self.logs.push_back(truncated);
        
        // Keep last 100 logs
        if self.logs.len() > 100 {
            self.logs.pop_front();
        }
        
        // Auto-scroll to bottom (latest logs)
        self.log_scroll = 0;
    }

    pub fn scroll_logs_up(&mut self) {
        let available_logs = self.logs.len().saturating_sub(10);
        if self.log_scroll < available_logs {
            self.log_scroll += 1;
        }
    }

    pub fn scroll_logs_down(&mut self) {
        if self.log_scroll > 0 {
            self.log_scroll -= 1;
        }
    }
}

#[derive(Debug)]
pub enum TuiUpdate {
    ServerStatus(ServerStatus),
    Log(String),
}
