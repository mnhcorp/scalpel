use super::app::{App, Focus};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Disassembly;

    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Disassembly View ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let content = if app.disassembly.is_empty() {
        vec![
            Line::from(Span::styled("No function selected", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("Disassembly will appear here", Style::default().fg(Color::Gray))),
            Line::from(Span::styled("when analyzing a function", Style::default().fg(Color::Gray))),
        ]
    } else {
        app.disassembly
            .iter()
            .map(|line| {
                // Color-code different parts of assembly
                if line.contains("call") || line.contains("jmp") || line.contains("je") || line.contains("jne") {
                    Line::from(Span::styled(line, Style::default().fg(Color::Yellow)))
                } else if line.contains("ret") {
                    Line::from(Span::styled(line, Style::default().fg(Color::Red)))
                } else if line.contains("mov") || line.contains("lea") {
                    Line::from(Span::styled(line, Style::default().fg(Color::Cyan)))
                } else {
                    Line::from(Span::styled(line, Style::default().fg(Color::White)))
                }
            })
            .collect()
    };

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}
