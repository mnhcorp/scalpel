use super::app::{App, Focus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),     // Main content
            Constraint::Length(3),  // Input
        ])
        .split(size);

    // Draw header
    draw_header(f, chunks[0]);

    // Draw main content area
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Left panel (Vitals + Probes)
            Constraint::Percentage(70), // Right panel (Disassembly + Logs)
        ])
        .split(chunks[1]);

    // Left panel
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Vitals
            Constraint::Min(5),      // Probes
        ])
        .split(content_chunks[0]);

    super::vitals::draw(f, left_chunks[0], app);
    super::probes::draw(f, left_chunks[1], app);

    // Right panel
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30), // Disassembly
            Constraint::Percentage(70), // Logs
        ])
        .split(content_chunks[1]);

    super::disassembly::draw(f, right_chunks[0], app);
    super::logs::draw(f, right_chunks[1], app);

    // Draw input
    super::input::draw(f, chunks[2], app);
}

fn draw_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("╔═══════════════════════════════════════════════════════════╗", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("║  ", Style::default().fg(Color::Cyan)),
            Span::styled("SCALPEL", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" v1.0.0 - Kernel Surgeon Edition", Style::default().fg(Color::White)),
            Span::raw("                  "),
            Span::styled("║", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("╚═══════════════════════════════════════════════════════════╝", Style::default().fg(Color::Cyan)),
        ]),
    ]);

    f.render_widget(header, area);
}
