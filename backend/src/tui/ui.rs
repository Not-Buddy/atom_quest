use super::app::{App, ServerStatus};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(12),
        ])
        .split(frame.area());

    // Title
    render_title(frame, chunks[0]);

    // Main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Menu
    render_menu(frame, app, main_chunks[0]);

    // Status panel
    render_status(frame, app, main_chunks[1]);

    // Logs
    render_logs(frame, app, chunks[2]);
}

fn render_title(frame: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new("🚀 Axum Server Manager")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_menu(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let content = if i == 0 {
                match app.server_status {
                    ServerStatus::Running => format!("{} ✓", item),
                    ServerStatus::Starting => format!("{} ⏳", item),
                    _ => item.clone(),
                }
            } else if i == 1 {
                match app.server_status {
                    ServerStatus::Stopped => format!("{} ✓", item),
                    ServerStatus::Stopping => format!("{} ⏳", item),
                    _ => item.clone(),
                }
            } else {
                item.clone()
            };

            ListItem::new(content)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(menu, area, &mut list_state);
}

fn render_status(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let status_text = match app.server_status {
        ServerStatus::Stopped => "Stopped",
        ServerStatus::Starting => "Starting...",
        ServerStatus::Running => "Running",
        ServerStatus::Stopping => "Stopping...",
    };

    let status_color = match app.server_status {
        ServerStatus::Running => Color::Green,
        ServerStatus::Stopped => Color::Red,
        _ => Color::Yellow,
    };

    let status_info = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::White)),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "http://localhost:3000",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(Span::styled("Endpoints:", Style::default().fg(Color::Yellow))),
        Line::from("  POST /auth/login"),
        Line::from("  GET  /auth/me"),
        Line::from("  POST /profile/links"),
        Line::from(""),
        Line::from(Span::styled("Controls:", Style::default().fg(Color::Yellow))),
        Line::from("  ↑/↓    Navigate menu"),
        Line::from("  PgUp/PgDn Scroll logs"),
        Line::from("  Enter  Select"),
        Line::from("  q/Esc  Quit"),
    ];

    let status_panel = Paragraph::new(status_info)
        .block(Block::default().borders(Borders::ALL).title("Server Info"))
        .style(Style::default().fg(Color::White));

    frame.render_widget(status_panel, area);
}

fn render_logs(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Calculate available height for logs
    let available_height = area.height.saturating_sub(2) as usize; // Subtract borders
    
    // Get the last N logs to display
    let log_items: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .skip(app.log_scroll)
        .take(available_height)
        .map(|log| {
            // Truncate to fit width
            let max_width = area.width.saturating_sub(4) as usize;
            let display_text = if log.len() > max_width {
                format!("{}…", &log[..max_width.saturating_sub(1)])
            } else {
                log.clone()
            };
            ListItem::new(display_text).style(Style::default().fg(Color::White))
        })
        .collect();

    let title = if app.log_scroll > 0 {
        format!("Logs (↑{} more | PgUp/PgDn to scroll)", app.log_scroll)
    } else {
        format!("Logs ({} total)", app.logs.len())
    };

    let logs = List::new(log_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Gray))
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(logs, area);
}
