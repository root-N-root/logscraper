use std::io;

use ratatui::{
    Terminal,
    crossterm::event::{self, Event},
    prelude::Backend,
};

use crate::reader::file::read_from_paths;

pub mod app;
pub mod ui;

use tokio::task;

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut app::App, tx: tokio::sync::mpsc::UnboundedSender<String>) -> io::Result<bool> {
    let mut tail_handle: Option<task::JoinHandle<()>> = None;
    
    loop {
        terminal.draw(|frame| ui::ui(frame, app))?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                app.handle(key.code);
                
                // При переключении в Tail Mode, запускаем процесс хвоста
                if app.cur_mode == crate::common::enums::Mode::Tail && tail_handle.is_none() {
                    let memory = app.memory.clone();
                    let tx_clone = tx.clone();
                    tail_handle = Some(task::spawn(async move {
                        if let Err(e) = crate::reader::tail::tail_stream(memory, tx_clone).await {
                            eprintln!("Error in tail stream: {}", e);
                        }
                    }));
                }
                
                // При выходе из Tail Mode, останавливаем процесс хвоста
                if app.cur_mode != crate::common::enums::Mode::Tail && tail_handle.is_some() {
                    if let Some(handle) = tail_handle.take() {
                        handle.abort();
                    }
                }
                
                if app.exit_approved {
                    // Останавливаем хвост, если он запущен, перед выходом
                    if let Some(handle) = tail_handle.take() {
                        handle.abort();
                    }
                    return Ok(true);
                }
            }
        }
        
        // Обновляем логи из канала
        app.update_logs();
        
        // Проверяем изменения, требующие обновления логов
        if app.cur_mode == crate::common::enums::Mode::Page {
            // Проверяем, изменился ли какой-либо параметр
            let order_changed = app.cur_order != app.last_order;
            let size_changed = app.cur_size != app.last_size;
            let mode_changed = app.cur_mode != app.last_mode;
            let paths_changed = app.memory.paths.len() != app.last_paths_count;
            let filters_changed = app.memory.filters.len() != app.last_filters_count;
            
            if order_changed || size_changed || mode_changed || paths_changed || filters_changed {
                app.needs_refresh = true;
                // Обновляем последние значения
                app.last_order = app.cur_order.clone();
                app.last_mode = app.cur_mode.clone();
                app.last_size = app.cur_size;
                app.last_paths_count = app.memory.paths.len();
                app.last_filters_count = app.memory.filters.len();
            }
        }
        
        // Обновляем логи при необходимости (в Page Mode)
        if app.needs_refresh && app.cur_mode == crate::common::enums::Mode::Page {
            app.needs_refresh = false;
            
            // Вычисляем смещение на основе страницы и размера
            let offset = (app.cur_page.saturating_sub(1)) * app.cur_size;
            
            // Читаем логи из всех путей с заданным порядком
            match read_from_paths(
                app.memory.paths.clone(),
                app.cur_size,
                offset,
                Some(app.memory.filters.clone()),
                app.cur_order.clone(),
            ).await {
                Ok(logs) => {
                    app.logs = logs;
                }
                Err(e) => {
                    eprintln!("Error reading logs: {}", e);
                }
            }
        }
    }
}
