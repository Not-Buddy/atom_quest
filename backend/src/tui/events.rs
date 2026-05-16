use super::app::{App, AppAction, TuiUpdate};
use super::terminal::TerminalGuard;
use super::ui;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use tokio::sync::mpsc;

pub async fn run_tui(
    mut tui_rx: mpsc::UnboundedReceiver<TuiUpdate>,
    action_tx: mpsc::UnboundedSender<AppAction>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal_guard = TerminalGuard::new()?;
    let terminal = terminal_guard.get_mut();

    let mut app = App::new(action_tx);
    let mut should_quit = false;

    while !should_quit {
        terminal.draw(|f| ui::render(f, &mut app))?;

        tokio::select! {
            // Handle keyboard events
            result = tokio::task::spawn_blocking(|| {
                event::poll(std::time::Duration::from_millis(100))
            }) => {
                if let Ok(Ok(true)) = result &&
                   let Ok(event) = event::read() &&
                   let Event::Key(key) = event &&
                   key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            let _ = app.action_tx.send(AppAction::Quit);
                            should_quit = true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::PageUp => app.scroll_logs_up(),
                        KeyCode::PageDown => app.scroll_logs_down(),
                        KeyCode::Enter => {
                            if let Some(action) = app.select() {
                                if matches!(action, AppAction::Quit) {
                                    should_quit = true;
                                }
                                let _ = app.action_tx.send(action);
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Handle updates from server
            Some(update) = tui_rx.recv() => {
                match update {
                    TuiUpdate::ServerStatus(status) => {
                        app.set_server_status(status);
                    }
                    TuiUpdate::Log(log) => {
                        app.add_log(log);
                    }
                }
            }
        }
    }

    drop(terminal_guard);
    Ok(())
}
