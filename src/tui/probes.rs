use super::app::{App, Focus};
use crate::kernel::ProbeStatus;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Probes;

    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(format!(" Active Probes ({}) ", app.probes.len()))
        .borders(Borders::ALL)
        .border_style(border_style);

    if app.probes.is_empty() {
        let paragraph = ratatui::widgets::Paragraph::new(vec![
            Line::from(Span::styled("No active probes", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("Type 'i' and enter a", Style::default().fg(Color::Gray))),
            Line::from(Span::styled("command to start", Style::default().fg(Color::Gray))),
        ])
        .block(block);

        f.render_widget(paragraph, area);
    } else {
        let items: Vec<ListItem> = app
            .probes
            .iter()
            .map(|probe| {
                let status_color = match probe.status {
                    ProbeStatus::Active => Color::Green,
                    ProbeStatus::Failed => Color::Red,
                    ProbeStatus::Detached => Color::DarkGray,
                };

                let probe_type = format!("{:?}", probe.probe_type);
                let mut spans = vec![
                    Span::styled(&probe.id, Style::default().fg(Color::Yellow)),
                    Span::raw(" "),
                    Span::styled(probe_type, Style::default().fg(status_color)),
                ];

                if probe.is_mutation {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled("⚠", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
                }

                let line1 = Line::from(spans);

                let line2 = Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(&probe.target, Style::default().fg(Color::Cyan)),
                    Span::raw(if let Some(offset) = probe.offset {
                        format!("+{:#x}", offset)
                    } else {
                        String::new()
                    }),
                ]);

                ListItem::new(vec![line1, line2])
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }
}
