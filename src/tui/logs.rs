use super::app::{App, Focus, LogLevel};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Logs;

    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Event Stream ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = app
        .logs
        .iter()
        .rev() // Show most recent first
        .take(100) // Limit display
        .map(|entry| {
            let (level_str, level_color) = match entry.level {
                LogLevel::Info => ("INFO", Color::Cyan),
                LogLevel::Warning => ("WARN", Color::Yellow),
                LogLevel::Error => ("ERR ", Color::Red),
                LogLevel::Success => ("OK  ", Color::Green),
            };

            let timestamp = entry.timestamp.format("%H:%M:%S");

            let line = Line::from(vec![
                Span::styled(
                    format!("[{}] ", timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("[{}] ", level_str),
                    Style::default()
                        .fg(level_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&entry.message, Style::default().fg(Color::White)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
