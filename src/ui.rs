use crate::app::{App, InputField, InputMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
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

    if app.input_mode == InputMode::Editing {
        render_input_popup(f, app);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_input_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area); //this clears out the background

    let block = Block::default()
        .title(" Create New SSH Key ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Name input
            Constraint::Length(3), // Email input
            Constraint::Length(2), // Error message
            Constraint::Min(1),    // Helper text
        ])
        .split(inner_area);

    let active_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let inactive_style = Style::default().fg(Color::DarkGray);

    let name_style = if app.input_field == InputField::Name { active_style } else { inactive_style };
    let email_style = if app.input_field == InputField::Email { active_style } else { inactive_style };

    // Name Input
    let name_text = format!("> {}", app.input_name);
    let name_widget = Paragraph::new(name_text)
        .style(name_style)
        .block(Block::default().borders(Borders::ALL).title("Key Name (e.g. id_ed25519)"));
    f.render_widget(name_widget, chunks[0]);

    // Email Input
    let email_text = format!("> {}", app.input_email);
    let email_widget = Paragraph::new(email_text)
        .style(email_style)
        .block(Block::default().borders(Borders::ALL).title("Email (e.g. user@example.com)"));
    f.render_widget(email_widget, chunks[1]);
    
    // Error Message
    if let Some(msg) = &app.popup_msg {
        let err_widget = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(err_widget, chunks[2]);
    }

    // Helper text
    let help_text = Paragraph::new("Tab to switch | Enter to submit | Esc to cancel")
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(help_text, chunks[3]);

    // Render cursor
    match app.input_mode {
        InputMode::Editing => {
            let cursor_x = match app.input_field {
                InputField::Name => chunks[0].x + app.input_name.len() as u16 + 2,
                InputField::Email => chunks[1].x + app.input_email.len() as u16 + 2,
            };
            let cursor_y = match app.input_field {
                InputField::Name => chunks[0].y + 1,
                InputField::Email => chunks[1].y + 1,
            };
            f.set_cursor_position((cursor_x, cursor_y));
        }
        _ => {}
    }
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
            Span::styled("n", Style::default().fg(Color::Yellow)),
            Span::raw(" to start creation | "),
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
