use crate::app::{ActiveTab, App, ConfigEditField, InputField, InputMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    app.update_clipboard_msg_timeout();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Header / tab bar
                Constraint::Min(0),   // Content area
            ]
            .as_ref(),
        )
        .split(f.area());

    render_header(f, app, main_chunks[0]);

    match app.active_tab {
        ActiveTab::Keys => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(30),
                        Constraint::Percentage(70),
                    ]
                    .as_ref(),
                )
                .split(main_chunks[1]);
            render_keys_list(f, app, chunks[0]);
            render_key_details(f, app, chunks[1]);
        }
        ActiveTab::SshConfig => {
            render_ssh_config(f, app, main_chunks[1]);
        }
        ActiveTab::KnownHosts => {
            render_known_hosts(f, app, main_chunks[1]);
        }
    }

    match app.input_mode {
        InputMode::Editing => render_input_popup(f, app),
        InputMode::FileBrowser => render_file_browser(f, app),
        InputMode::ImportAction => render_action_popup(f, app),
        InputMode::PasswordPrompt => render_password_popup(f, app),
        InputMode::ConfigEditing => render_config_edit_popup(f, app),
        InputMode::Normal => {}
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
                InputField::Name => chunks[0].x + app.input_name.len() as u16 + 3,
                InputField::Email => chunks[1].x + app.input_email.len() as u16 + 3,
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

fn render_file_browser(f: &mut Frame, app: &mut App) {
    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .file_entries
        .iter()
        .map(|p| {
            let mut name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            if p.is_dir() {
                name.push('/');
                ListItem::new(Line::from(vec![Span::styled(
                    name,
                    Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD),
                )]))
            } else {
                ListItem::new(Line::from(vec![Span::raw(name)]))
            }
        })
        .collect();

    let title = format!(" Browser: {} ", app.current_dir.display());
    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items_list, area, &mut app.file_list_state);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let msg = if let Some((msg, _)) = &app.clipboard_msg {
        Line::from(vec![Span::styled(
            msg,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )])
    } else {
        // Tab indicators
        let tab_style = |tab: &ActiveTab, label: &str, key: &str| -> Vec<Span> {
            let is_active = *tab == app.active_tab;
            let style = if is_active {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            vec![
                Span::styled(format!("[{}] ", key), Style::default().fg(Color::Yellow)),
                Span::styled(label.to_string(), style),
            ]
        };

        let mut spans = Vec::new();
        spans.extend(tab_style(&ActiveTab::Keys, "Keys", "1"));
        spans.push(Span::raw("  "));
        spans.extend(tab_style(&ActiveTab::SshConfig, "Config", "2"));
        spans.push(Span::raw("  "));
        spans.extend(tab_style(&ActiveTab::KnownHosts, "Hosts", "3"));
        spans.push(Span::raw("  │  "));

        // Context-sensitive hints
        match app.active_tab {
            ActiveTab::Keys => {
                spans.push(Span::styled("n", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(":new "));
                spans.push(Span::styled("i", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(":import "));
                spans.push(Span::styled("c", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(":copy "));
            }
            ActiveTab::SshConfig => {
                spans.push(Span::styled("a", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(":add "));
                spans.push(Span::styled("e", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(":edit "));
                spans.push(Span::styled("d", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(":delete "));
            }
            ActiveTab::KnownHosts => {
                spans.push(Span::styled("d", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(":delete "));
            }
        }
        spans.push(Span::styled("q", Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(":quit"));

        Line::from(spans)
    };

    let header = Paragraph::new(msg)
        .block(Block::default().borders(Borders::ALL).title(" easy-ssh-tui "))
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

fn render_action_popup(f: &mut Frame, _app: &App) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Import Action ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(inner_area);

    let question = Paragraph::new(Line::from(vec![
        Span::raw("Do you want to "),
        Span::styled("Move", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" or "),
        Span::styled("Copy", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" this file?"),
    ])).alignment(ratatui::layout::Alignment::Center);
    f.render_widget(question, chunks[0]);

    let options = Paragraph::new("Press [m] for Move, [c] for Copy")
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(options, chunks[1]);
}

fn render_password_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Encrypted Key ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Intro
            Constraint::Length(3), // Input
            Constraint::Length(2), // Error
        ])
        .split(inner_area);

    let intro = Paragraph::new("This key requires a passphrase:")
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(intro, chunks[0]);

    // Mask password
    let masked: String = app.password_input.chars().map(|_| '*').collect();
    let input_text = format!("> {}", masked);
    
    let input_widget = Paragraph::new(input_text)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Passphrase"));
    f.render_widget(input_widget, chunks[1]);

    if let Some(msg) = &app.popup_msg {
        let err_widget = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(err_widget, chunks[2]);
    }

    // Set cursor
    f.set_cursor_position((chunks[1].x + app.password_input.len() as u16 + 3, chunks[1].y + 1));
}

fn render_ssh_config(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    // Config entries list
    let items: Vec<ListItem> = app
        .config_entries
        .iter()
        .map(|e| {
            let detail = if !e.hostname.is_empty() {
                format!("{} → {}@{}:{}", e.host, if e.user.is_empty() { "*" } else { &e.user }, e.hostname, e.port)
            } else {
                e.host.clone()
            };
            ListItem::new(Line::from(vec![Span::styled(
                detail,
                Style::default().fg(Color::White),
            )]))
        })
        .collect();

    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" SSH Config Entries "))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items_list, chunks[0], &mut app.config_list_state);

    // Config details for selected entry
    if let Some(idx) = app.config_list_state.selected() {
        if let Some(entry) = app.config_entries.get(idx) {
            let detail_text = vec![
                Line::from(vec![
                    Span::styled("Host:          ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(&entry.host),
                ]),
                Line::from(vec![
                    Span::styled("HostName:      ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(if entry.hostname.is_empty() { "-" } else { &entry.hostname }),
                ]),
                Line::from(vec![
                    Span::styled("User:          ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(if entry.user.is_empty() { "-" } else { &entry.user }),
                ]),
                Line::from(vec![
                    Span::styled("Port:          ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(&entry.port),
                ]),
                Line::from(vec![
                    Span::styled("IdentityFile:  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(if entry.identity_file.is_empty() { "-" } else { &entry.identity_file }),
                ]),
            ];
            let detail_p = Paragraph::new(detail_text)
                .block(Block::default().borders(Borders::ALL).title(" Entry Details "))
                .wrap(Wrap { trim: true });
            f.render_widget(detail_p, chunks[1]);
        }
    } else {
        let empty_p = Paragraph::new("No config file found or no entries")
            .block(Block::default().borders(Borders::ALL).title(" Entry Details "))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(empty_p, chunks[1]);
    }
}

fn render_known_hosts(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .known_hosts
        .iter()
        .map(|e| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<40}", e.hostname),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:<12}", e.key_type),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  "),
                Span::styled(&e.fingerprint, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let title = format!(" Known Hosts ({}) ", app.known_hosts.len());
    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items_list, area, &mut app.known_hosts_list_state);
}

fn render_config_edit_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 55, f.area());
    f.render_widget(Clear, area);

    let title = if app.config_edit_index.is_some() {
        " Edit SSH Config Entry "
    } else {
        " Add SSH Config Entry "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Host
            Constraint::Length(3), // Hostname
            Constraint::Length(3), // User
            Constraint::Length(3), // Port
            Constraint::Length(3), // IdentityFile
            Constraint::Length(2), // Error
            Constraint::Min(1),   // Help
        ])
        .split(inner_area);

    if let Some(entry) = &app.editing_config {
        let active_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let inactive_style = Style::default().fg(Color::DarkGray);

        let fields = [
            ("Host (alias)", &entry.host, ConfigEditField::Host),
            ("HostName (IP/domain)", &entry.hostname, ConfigEditField::Hostname),
            ("User", &entry.user, ConfigEditField::User),
            ("Port", &entry.port, ConfigEditField::Port),
            ("IdentityFile", &entry.identity_file, ConfigEditField::IdentityFile),
        ];

        for (i, (label, value, field_enum)) in fields.iter().enumerate() {
            let style = if app.config_edit_field == *field_enum { active_style } else { inactive_style };
            let text = format!("> {}", value);
            let widget = Paragraph::new(text)
                .style(style)
                .block(Block::default().borders(Borders::ALL).title(*label));
            f.render_widget(widget, chunks[i]);
        }

        // Error message
        if let Some(msg) = &app.popup_msg {
            let err_w = Paragraph::new(msg.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(err_w, chunks[5]);
        }

        // Help
        let help = Paragraph::new("Tab to switch | Enter to save | Esc to cancel")
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(help, chunks[6]);

        // Cursor
        let field_idx = match app.config_edit_field {
            ConfigEditField::Host => 0,
            ConfigEditField::Hostname => 1,
            ConfigEditField::User => 2,
            ConfigEditField::Port => 3,
            ConfigEditField::IdentityFile => 4,
        };
        let current_value = fields[field_idx].1;
        f.set_cursor_position((
            chunks[field_idx].x + current_value.len() as u16 + 3,
            chunks[field_idx].y + 1,
        ));
    }
}
