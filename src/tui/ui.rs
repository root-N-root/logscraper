use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

const TITLE: &str = "LogScraper on Rust by Devputat";

use crate::{
    common::enums::Mode,
    tui::app::{self, App},
};

pub fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    let title = Paragraph::new(Text::styled(
        TITLE.to_string(),
        Style::default().fg(ratatui::style::Color::Green),
    ))
    .block(title_block);
    frame.render_widget(title, chunks[0]);

    let mode_footer = Paragraph::new(Line::from(vec![
        app.cur_mode.nav_text(),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        app.cur_order.order_text(),
    ]))
    .block(Block::default().borders(Borders::ALL));
    let hint_footer = Paragraph::new(Line::from(app.cur_screen.keys_hint()))
        .block(Block::default().borders(Borders::ALL));
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);
    frame.render_widget(mode_footer, footer_chunks[0]);
    frame.render_widget(hint_footer, footer_chunks[1]);

    let display_logs = match app.cur_mode {
        Mode::Page => {
            // app.logs уже содержит логи для текущей страницы, 
            // загруженные с правильным offset в run_app
            &app.logs
        }
        Mode::Tail => {
            let start = app.logs.len().saturating_sub(app.cur_size);
            &app.logs[start..]
        }
        Mode::Stopped => &app.logs,
    };
    let text = Text::from(
        display_logs
            .iter()
            .map(|l| Line::from(l.clone()))
            .collect::<Vec<Line>>(),
    );
    let log_block = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(log_block, chunks[1]);

    if let Some(modal) = &app.cur_modal {
        let area = centered_rect(60, 25, frame.area());

        // Create a new chunk for the popup content
        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(1),    // List content
                Constraint::Length(3), // Input/edit area or instructions
            ])
            .split(area);

        // Render a semi-transparent overlay background
        let overlay = Paragraph::new("").block(
            Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0))), // Dark background
        );
        frame.render_widget(overlay, area);

        let popup_block = Block::default()
            .title(modal.title())
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
        frame.render_widget(Clear, area);
        frame.render_widget(popup_block, area);

        match modal {
            app::Modal::Path => {
                // Display list of paths
                let items: Vec<ListItem> = app
                    .memory
                    .paths
                    .iter()
                    .enumerate()
                    .map(|(i, path)| {
                        let is_selected = app.selected_index == Some(i);
                        let style = if is_selected {
                            Style::default().bg(Color::Blue).fg(Color::White)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        ListItem::new(Line::from(Span::styled(
                            format!("{}: {}", i, path.path),
                            style,
                        )))
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default().bg(Color::DarkGray)),
                    )
                    .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

                frame.render_widget(Clear, popup_chunks[1]);
                frame.render_widget(list, popup_chunks[1]);
            }
            app::Modal::Filter => {
                // Display list of filters
                let items: Vec<ListItem> = app
                    .memory
                    .filters
                    .iter()
                    .enumerate()
                    .map(|(i, filter)| {
                        let is_selected = app.selected_index == Some(i);
                        let style = if is_selected {
                            Style::default().bg(Color::Blue).fg(Color::White)
                        } else {
                            Style::default().fg(Color::White)
                        };

                        let filter_text = match filter {
                            crate::common::enums::Filter::Search(f) => {
                                format!("{}: Search '{}'", i, f.substr)
                            }
                            crate::common::enums::Filter::Regex(f) => {
                                format!("{}: Regex '{}'", i, f.pattern)
                            }
                            crate::common::enums::Filter::Date(f) => {
                                let date_filter_type_str = match &f.filter_type {
                                    crate::common::structs::DateFilterType::Before => "Before",
                                    crate::common::structs::DateFilterType::After => "After", 
                                    crate::common::structs::DateFilterType::Between => "Between",
                                };
                                
                                let start_str = f.date_start
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                    .unwrap_or_else(|| "N/A".to_string());
                                
                                let end_str = f.date_finish
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                    .unwrap_or_else(|| "N/A".to_string());
                                
                                format!("{}: Date ({}) {} - {}", i, date_filter_type_str, start_str, end_str)
                            }
                        };

                        ListItem::new(Line::from(Span::styled(filter_text, style)))
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default().bg(Color::DarkGray)),
                    )
                    .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

                frame.render_widget(Clear, popup_chunks[1]);
                frame.render_widget(list, popup_chunks[1]);
            }
        }

        // Show editing interface or instructions
        let instructions = if app.editing_mode {
            if app.filter_type == app::FilterType::Date {
                // Специальное отображение для датового фильтра
                let field_name = match app.editing_date_field {
                    app::DateField::Start => "Start date",
                    app::DateField::Finish => "End date", 
                    app::DateField::None => "General"
                };
                format!(
                    "{}: {} (Tab: switch field, Enter: save, Esc: cancel)",
                    field_name,
                    match app.editing_date_field {
                        app::DateField::Start => &app.date_start_buffer,
                        app::DateField::Finish => &app.date_finish_buffer,
                        app::DateField::None => &app.edit_buffer,
                    }
                )
            } else {
                format!(
                    "Editing: {} (Press Enter to save, Esc to cancel)",
                    app.edit_buffer
                )
            }
        } else {
            match modal {
                app::Modal::Path => "Use arrow keys to select, Enter to edit, 'a' to add, 'd' to delete, 'q' to quit".to_string(),
                app::Modal::Filter => {
                    if app.filter_type == app::FilterType::Date {
                        format!(
                            "Use arrow keys to select, Enter to edit, 'a' to add, 'd' to delete, 'q' to quit | Date filter type: 1-Before, 2-After, 3-Between (current: {:?})", 
                            app.date_filter_type
                        )
                    } else {
                        format!(
                            "Use arrow keys to select, Enter to edit, 'a' to add, 'd' to delete, 'q' to quit | Filter type: 1-Search, 2-Regex, 3-Date (current: {:?})", 
                            app.filter_type
                        )
                    }
                }
            }
        };

        let instruction_paragraph = Paragraph::new(instructions).block(
            Block::default()
                .borders(Borders::TOP)
                .style(Style::default().bg(Color::DarkGray)),
        );
        frame.render_widget(instruction_paragraph, popup_chunks[2]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popout_layout = Layout::default()
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
        .split(popout_layout[1])[1]
}
