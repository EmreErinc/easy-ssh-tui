use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    app.update_clipboard_msg_timeout();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),      // Header / messages
                Constraint::Percentage(30), // Keys list
                Constraint::Percentage(70), // Key details
            ]
            .as_ref(),
        )
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_keys_list(f, app, chunks[1]);
    render_key_details(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let msg = if let Some((msg, _)) = &app.clipboard_msg {
        Line::from(vec![Span::styled(
            msg,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )])
    } else {
        Line::from(vec![
            Span::raw("Use "),
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" or "),
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(" to navigate | "),
            Span::styled("c", Style::default().fg(Color::Yellow)),
            Span::raw(" to copy public key | "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" to quit"),
        ])
    };

    let header = Paragraph::new(msg)
        .block(Block::default().borders(Borders::ALL).title(" easy-ssh "))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(header, area);
}

fn render_keys_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .keys
        .iter()
        .map(|k| {
            ListItem::new(Line::from(vec![Span::styled(
                k.name.clone(),
                Style::default().fg(Color::White),
            )]))
        })
        .collect();

    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" SSH Keys "))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items_list, area, &mut app.list_state);
}

fn render_key_details(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(area);

    if let Some(selected) = app.list_state.selected() {
        if let Some(key) = app.keys.get(selected) {
            let pub_key_p = Paragraph::new(key.public_content.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Public Key (Selected) "),
                )
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(pub_key_p, chunks[0]);

            let priv_key_p = Paragraph::new(key.private_content.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Private Key "),
                )
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::LightMagenta));
            f.render_widget(priv_key_p, chunks[1]);
        }
    } else {
        let empty_p = Paragraph::new("No key selected")
            .block(Block::default().borders(Borders::ALL).title(" Details "))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(empty_p, area);
    }
}
