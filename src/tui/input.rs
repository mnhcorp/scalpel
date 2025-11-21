use super::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let border_style = if app.input_mode {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if app.input_mode {
        " Input [ESC to exit] "
    } else {
        " Press 'i' to enter command mode "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let input_text = if app.input_mode {
        vec![Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(&app.input, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Green).add_modifier(Modifier::SLOW_BLINK)),
        ])]
    } else {
        vec![Line::from(Span::styled(
            "Tab: switch focus | i: input | q: quit | ↑↓: scroll",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(input_text).block(block);
    f.render_widget(paragraph, area);
}
