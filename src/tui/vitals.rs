use super::app::{App, Focus};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Vitals;

    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Kernel Vitals ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let content = if let Some(vitals) = &app.vitals {
        vec![
            Line::from(vec![
                Span::styled("Load Avg: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.2} {:.2} {:.2}", vitals.load_avg_1m, vitals.load_avg_5m, vitals.load_avg_15m),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Interrupts: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}/s", format_number(vitals.interrupts_per_sec)),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Context Sw: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}/s", format_number(vitals.context_switches_per_sec)),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Processes:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} running", vitals.processes_running),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(Span::styled("Loading vitals...", Style::default().fg(Color::Gray))),
        ]
    };

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
