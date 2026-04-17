use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::{self, App, Screen};

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
        ])
        .split(frame.area());

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "Alpha CLI",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  AlphaSmart NEO backup utility"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    match app.screen {
        Screen::Waiting => draw_waiting(frame, chunks[1]),
        Screen::MainMenu => draw_main_menu(frame, chunks[1], app),
        Screen::Files => draw_files(frame, chunks[1], app),
    }

    let footer = Paragraph::new(app.status.as_str())
        .wrap(Wrap { trim: true })
        .block(Block::default().title("Status").borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);

    if let Some(error) = &app.error {
        draw_error(frame, error);
    }
}

fn draw_waiting(frame: &mut Frame<'_>, area: Rect) {
    let text = vec![
        Line::from("1. Connect the AlphaSmart NEO over USB."),
        Line::from("2. Leave it in the normal USB keyboard mode."),
        Line::from(
            "3. This app switches it to direct USB mode and initializes the updater protocol.",
        ),
        Line::from(""),
        Line::from("Press q to quit."),
    ];
    frame.render_widget(
        Paragraph::new(text).wrap(Wrap { trim: true }).block(
            Block::default()
                .title("Waiting for device")
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn draw_main_menu(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let items = vec![ListItem::new("Files on device")];
    let mut state = ListState::default();
    state.select(Some(app.main_selected));
    frame.render_stateful_widget(menu("Main menu", items), area, &mut state);
}

fn draw_files(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(5)])
        .split(area);
    let mut items = app
        .files
        .iter()
        .map(|entry| {
            let words = app::approximate_words_from_bytes(entry.attribute_bytes);
            ListItem::new(format!(
                "Slot {:>2}  {:<24}  {:>9}  ~{:>5} words",
                entry.slot,
                entry.name,
                app::human_bytes(entry.attribute_bytes),
                words
            ))
        })
        .collect::<Vec<_>>();
    items.push(ListItem::new("All files"));
    let mut state = ListState::default();
    state.select(Some(app.file_selected));
    frame.render_stateful_widget(menu("Files on device", items), rows[0], &mut state);

    let backup_hint = "Selecting an item downloads it to ~/alpha-cli/backups/{date-time}/contents. Each file is saved as raw bytes plus a host-readable .txt export. This never writes to the NEO.";
    frame.render_widget(
        Paragraph::new(backup_hint).wrap(Wrap { trim: true }).block(
            Block::default()
                .title("Backup destination")
                .borders(Borders::ALL),
        ),
        rows[1],
    );
}

fn menu<'a>(title: &'a str, items: Vec<ListItem<'a>>) -> List<'a> {
    List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
}

fn draw_error(frame: &mut Frame<'_>, error: &str) {
    let area = centered_rect(70, 40, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(format!(
            "{error}\n\nPress Esc or q. See ~/alpha-cli/logs/alpha-cli.log for details."
        ))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title("Error")
                .borders(Borders::ALL)
                .border_style(Color::Red),
        ),
        area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
