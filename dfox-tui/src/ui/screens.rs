use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use super::DatabaseClientUI;

pub async fn render_db_type_selection_screen(
    ui: &mut DatabaseClientUI,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let db_types = ["Postgres", "MySQL", "SQLite"];
    let db_type_list: Vec<ListItem> = db_types
        .iter()
        .enumerate()
        .map(|(i, &db)| {
            if i == ui.selected_db_type {
                ListItem::new(db).style(
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(db).style(Style::default().fg(Color::White))
            }
        })
        .collect();

    terminal.draw(|f| {
        let size = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(size);

        let horizontal_layout = centered_rect(50, chunks[1]);

        let block = Block::default()
            .title("Select Database Type")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);

        let db_type_widget = List::new(db_type_list).block(block).highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        f.render_widget(db_type_widget, horizontal_layout);

        let help_message = vec![Line::from(vec![
            Span::styled(
                "Up",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/"),
            Span::styled(
                "Down",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to navigate, "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to select, "),
            Span::styled(
                "q",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to quit"),
        ])];

        let help_paragraph = Paragraph::new(help_message)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(help_paragraph, chunks[2]);
    })?;

    Ok(())
}

pub async fn render_connection_input_screen(
    ui: &mut DatabaseClientUI,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(size);

        let horizontal_layout = centered_rect(50, vertical_chunks[1]);

        let block = Block::default()
            .title("Enter Connection Details")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);

        let mut content = [
            format!("Username: {}", ui.connection_input.username),
            format!(
                "Password: {}",
                "*".repeat(ui.connection_input.password.len())
            ),
            format!("Hostname: {}", ui.connection_input.hostname),
        ];

        content[ui.current_input_index()].push_str(" <");

        let input_paragraph = Paragraph::new(content.join("\n"))
            .block(block)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);

        f.render_widget(input_paragraph, horizontal_layout);

        // Раздел с описанием активных клавиш
        let help_message = vec![Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to confirm input, "),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to go back"),
        ])];

        let help_paragraph = Paragraph::new(help_message)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(help_paragraph, vertical_chunks[2]);
    })?;

    Ok(())
}

pub async fn render_database_selection_screen(
    ui: &mut DatabaseClientUI,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    match ui.fetch_databases().await {
        Ok(databases) => {
            ui.databases = databases;
        }
        Err(_) => {
            ui.databases = vec!["Error fetching databases".to_string()];
        }
    }

    let db_list: Vec<ListItem> = ui
        .databases
        .iter()
        .enumerate()
        .map(|(i, db)| {
            if i == ui.selected_db_type {
                ListItem::new(db.clone()).style(
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(db.clone()).style(Style::default().fg(Color::White))
            }
        })
        .collect();

    terminal.draw(|f| {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ]
                .as_ref(),
            )
            .split(size);

        let horizontal_layout = centered_rect(50, chunks[1]);

        let block = Block::default()
            .title("Select Database")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);

        let db_list_widget = List::new(db_list).block(block).highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        f.render_widget(db_list_widget, horizontal_layout);
    })?;

    Ok(())
}

pub async fn render_table_view_screen(
    ui: &mut DatabaseClientUI,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let tables = ui.fetch_tables().await.unwrap_or_else(|_| vec![]);

    terminal.draw(|f| {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(size);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[1]);

        let tables_block = Block::default().borders(Borders::ALL).title("Tables");

        let table_list: Vec<ListItem> = tables
            .iter()
            .map(|table| ListItem::new(table.to_string()))
            .collect();

        let tables_widget = List::new(table_list).block(tables_block);

        let sql_query_block = Block::default().borders(Borders::ALL).title("SQL Query");

        let sql_query_widget = Paragraph::new("SELECT * FROM ...")
            .block(sql_query_block)
            .style(Style::default().fg(Color::White));

        let sql_result_block = Block::default().borders(Borders::ALL).title("Query Result");

        let sql_result_widget = Paragraph::new("Results will be shown here...")
            .block(sql_result_block)
            .style(Style::default().fg(Color::White));

        f.render_widget(tables_widget, chunks[0]);
        f.render_widget(sql_query_widget, right_chunks[0]);
        f.render_widget(sql_result_widget, right_chunks[1]);
    })?;

    Ok(())
}

// Аналогично можно разбить render_database_selection_screen и render_table_view_screen.

fn centered_rect(percent_x: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    popup_layout[1]
}
