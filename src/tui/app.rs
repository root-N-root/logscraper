use ratatui::{
    crossterm::event::KeyCode,
    style::{Color, Style},
    text::Span,
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::common::enums::{Mode, Order};
use crate::common::structs::Memory;

pub struct App {
    pub cur_screen: Screen,
    pub cur_modal: Option<Modal>,
    pub cur_order: Order,
    pub cur_mode: Mode,
    pub cur_size: usize,
    pub cur_page: usize,
    pub exit_approved: bool,
    pub logs: Vec<String>,
    pub rx: UnboundedReceiver<String>,
    pub memory: Memory,
    // Поля для управления состоянием модальных окон
    pub selected_index: Option<usize>, // Индекс выбранного элемента в списках модальных окон
    pub editing_mode: bool,            // Находится ли приложение в режиме редактирования
    pub edit_buffer: String,           // Буфер для ввода текста во время редактирования
    pub filter_type: FilterType,       // Тип фильтра при добавлении/редактировании фильтров
    pub date_filter_type: crate::common::structs::DateFilterType, // Тип датового фильтра (до/после/между)
    pub date_start_buffer: String,     // Буфер для ввода начальной даты
    pub date_finish_buffer: String,    // Буфер для ввода конечной даты
    pub editing_date_field: DateField, // Поле, которое в данный момент редактируется
    // Поля для управления загрузкой логов
    pub needs_refresh: bool,           // Нужно ли обновить логи
    // Поля для отслеживания изменений
    pub last_order: Order,             // Последняя настройка сортировки 
    pub last_mode: Mode,               // Последняя настройка режима
    pub last_size: usize,              // Последний размер страницы
    pub last_paths_count: usize,       // Последнее количество путей
    pub last_filters_count: usize,     // Последнее количество фильтров
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    Date,
    Regex,
    Search,
}

impl App {
    pub fn new(rx: UnboundedReceiver<String>, memory: Memory) -> App {
        let paths_count = memory.paths.len();
        let filters_count = memory.filters.len();
        
        App {
            cur_screen: Screen::Main,
            cur_modal: None,
            cur_order: Order::OrderByDate,
            cur_mode: Mode::Page,
            cur_size: 30,
            exit_approved: false,
            cur_page: 1,
            logs: Vec::new(),
            rx,
            memory,
            selected_index: None,
            editing_mode: false,
            edit_buffer: String::new(),
            date_filter_type: crate::common::structs::DateFilterType::Between,
            date_start_buffer: String::new(),
            date_finish_buffer: String::new(),
            editing_date_field: DateField::None,
            filter_type: FilterType::Search,
            needs_refresh: true, // Обновляем логи при первом отображении
            last_order: Order::OrderByDate,
            last_mode: Mode::Page,
            last_size: 30,
            last_paths_count: paths_count,
            last_filters_count: filters_count,
        }
    }
    pub fn handle_additional(&mut self, key: KeyCode) {
        if self.editing_mode {
            // Обрабатываем ввод текста в режиме редактирования
            match key {
                KeyCode::Enter => {
                    // Save the edited value
                    if let (Some(_modal), Some(index)) = (&self.cur_modal, self.selected_index) {
                        match _modal {
                            Modal::Path => {
                                if index < self.memory.paths.len() {

                                    let updated_path = crate::common::structs::Path::new(
                                        self.edit_buffer.clone(),
                                        self.edit_buffer.clone(),
                                    );
                                    let _ = self.memory.update_path(index, updated_path);
                                }
                            }
                            Modal::Filter => {
                                if index < self.memory.filters.len() {

                                    let updated_filter = match self.filter_type {
                                        FilterType::Search => crate::common::enums::Filter::Search(
                                            crate::common::structs::SearchFilter {
                                                substr: self.edit_buffer.clone(),
                                            },
                                        ),
                                        FilterType::Regex => crate::common::enums::Filter::Regex(
                                            crate::common::structs::RegexFilter {
                                                pattern: self.edit_buffer.clone(),
                                            },
                                        ),
                                        FilterType::Date => {
                                            // Для датового фильтра парсим даты из буферов
                                            let date_start = if !self.date_start_buffer.is_empty() {
                                                chrono::DateTime::parse_from_str(&self.date_start_buffer, "%Y-%m-%dT%H:%M:%S%z")
                                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                                    .ok()
                                            } else {
                                                None
                                            };
                                            
                                            let date_finish = if !self.date_finish_buffer.is_empty() {
                                                chrono::DateTime::parse_from_str(&self.date_finish_buffer, "%Y-%m-%dT%H:%M:%S%z")
                                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                                    .ok()
                                            } else {
                                                None
                                            };
                                            
                                            crate::common::enums::Filter::Date(
                                                crate::common::structs::DateFilter {
                                                    date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
                                                    date_start,
                                                    date_finish,
                                                    filter_type: self.date_filter_type.clone(),
                                                },
                                            )
                                        }
                                    };
                                    let _ = self.memory.update_filter(index, updated_filter);
                                }
                            }
                        }
                    }
                    self.editing_mode = false;
                    self.edit_buffer.clear();
                    self.date_start_buffer.clear();
                    self.date_finish_buffer.clear();
                    self.editing_date_field = DateField::None;
                }
                KeyCode::Char(c) => {
                    // В зависимости от типа фильтра и поля редактирования, добавляем символ в соответствующий буфер
                    match self.filter_type {
                        FilterType::Date => {
                            match self.editing_date_field {
                                DateField::Start => self.date_start_buffer.push(c),
                                DateField::Finish => self.date_finish_buffer.push(c),
                                DateField::None => self.edit_buffer.push(c), // Резервный случай
                            }
                        },
                        _ => self.edit_buffer.push(c),
                    }
                }
                KeyCode::Backspace => {
                    // В зависимости от типа фильтра и поля редактирования, удаляем символ из соответствующего буфера
                    match self.filter_type {
                        FilterType::Date => {
                            match self.editing_date_field {
                                DateField::Start => { self.date_start_buffer.pop(); },
                                DateField::Finish => { self.date_finish_buffer.pop(); },
                                DateField::None => { self.edit_buffer.pop(); } // Резервный случай
                            }
                        },
                        _ => { self.edit_buffer.pop(); },
                    }
                }
                KeyCode::Esc => {
                    // Cancel editing
                    self.editing_mode = false;
                    self.edit_buffer.clear();
                }
                KeyCode::Tab => {
                    if let (Some(_modal), Some(index)) = (&self.cur_modal, self.selected_index) {
                        if index < self.memory.filters.len() {
                            match self.filter_type {
                                FilterType::Search => {
                                    let updated_filter = crate::common::enums::Filter::Regex(
                                        crate::common::structs::RegexFilter {
                                            pattern: self.edit_buffer.clone(),
                                        },
                                    );
                                    let _ = self.memory.update_filter(index, updated_filter);
                                    self.filter_type = FilterType::Regex;
                                },
                                FilterType::Regex => {
                                    let updated_filter = crate::common::enums::Filter::Date(
                                        crate::common::structs::DateFilter {
                                            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
                                            date_start: None,
                                            date_finish: None,
                                            filter_type: crate::common::structs::DateFilterType::Between,
                                        },
                                    );
                                    let _ = self.memory.update_filter(index, updated_filter);
                                    self.filter_type = FilterType::Date;
                                    self.date_filter_type = crate::common::structs::DateFilterType::Between;
                                    self.date_start_buffer = String::new();
                                    self.date_finish_buffer = String::new();
                                    self.editing_date_field = DateField::Start;
                                },
                                FilterType::Date => {
                                    // Для датового фильтра переключаемся между полями дат или между типами фильтров
                                    match self.editing_date_field {
                                        DateField::Start => {
                                            self.editing_date_field = DateField::Finish;
                                        },
                                        DateField::Finish => {
                                            // Если даты введены, сохраняем фильтр, иначе переключаемся к обычным фильтрам
                                            let date_start = if !self.date_start_buffer.is_empty() {
                                                // Попытка парсинга даты для проверки корректности
                                                chrono::DateTime::parse_from_str(&self.date_start_buffer, "%Y-%m-%dT%H:%M:%S%z")
                                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                                    .ok()
                                            } else {
                                                None
                                            };
                                            
                                            let date_finish = if !self.date_finish_buffer.is_empty() {
                                                chrono::DateTime::parse_from_str(&self.date_finish_buffer, "%Y-%m-%dT%H:%M:%S%z")
                                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                                    .ok()
                                            } else {
                                                None
                                            };
                                            
                                            let updated_filter = crate::common::enums::Filter::Date(
                                                crate::common::structs::DateFilter {
                                                    date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
                                                    date_start,
                                                    date_finish,
                                                    filter_type: self.date_filter_type.clone(),
                                                },
                                            );
                                            let _ = self.memory.update_filter(index, updated_filter);
                                            self.filter_type = FilterType::Search;
                                        },
                                        DateField::None => {
                                            self.editing_date_field = DateField::Start;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            return; // Return early to avoid other processing while editing
        }

        match key {
            KeyCode::Char('q') => {
                // Save memory when exiting modal
                let _ = self.memory.save();
                self.cur_screen = Screen::Main;
                self.cur_modal = None;
                self.selected_index = None; // Reset selection
            }
            KeyCode::Char('a') => {
                // Add new item
                match self.cur_modal {
                    Some(Modal::Path) => {
                        // Add a new empty path
                        self.memory.add_path(crate::common::structs::Path::new(
                            "".to_string(),
                            "".to_string(),
                        ));
                        // Select the newly added item
                        self.selected_index = Some(self.memory.paths.len().saturating_sub(1));
                    }
                    Some(Modal::Filter) => {
                        // Add a new empty search filter by default
                        let new_filter = crate::common::enums::Filter::Search(
                            crate::common::structs::SearchFilter {
                                substr: "".to_string(),
                            },
                        );
                        self.memory.add_filter(new_filter);
                        // Select the newly added item
                        self.selected_index = Some(self.memory.filters.len().saturating_sub(1));
                    }
                    None => {}
                }
            }
            KeyCode::Char('d') => {
                // Delete selected item
                match self.cur_modal {
                    Some(Modal::Path) => {
                        if let Some(index) = self.selected_index {
                            if index < self.memory.paths.len() {
                                let _ = self.memory.remove_path(index);
                                // Adjust selected index if needed
                                if index >= self.memory.paths.len() && !self.memory.paths.is_empty()
                                {
                                    self.selected_index = Some(self.memory.paths.len() - 1);
                                } else if self.memory.paths.is_empty() {
                                    self.selected_index = None;
                                }
                            }
                        }
                    }
                    Some(Modal::Filter) => {
                        if let Some(index) = self.selected_index {
                            if index < self.memory.filters.len() {
                                let _ = self.memory.remove_filter(index);
                                // Adjust selected index if needed
                                if index >= self.memory.filters.len()
                                    && !self.memory.filters.is_empty()
                                {
                                    self.selected_index = Some(self.memory.filters.len() - 1);
                                } else if self.memory.filters.is_empty() {
                                    self.selected_index = None;
                                }
                            }
                        }
                    }
                    None => {}
                }
            }
            KeyCode::Up => {
                // Move selection up in the list
                match self.cur_modal {
                    Some(Modal::Path) => {
                        if !self.memory.paths.is_empty() {
                            self.selected_index = match self.selected_index {
                                Some(0) | None => Some(self.memory.paths.len() - 1),
                                Some(i) => Some(i - 1),
                            };
                        }
                    }
                    Some(Modal::Filter) => {
                        if !self.memory.filters.is_empty() {
                            self.selected_index = match self.selected_index {
                                Some(0) | None => Some(self.memory.filters.len() - 1),
                                Some(i) => Some(i - 1),
                            };
                        }
                    }
                    None => {}
                }
            }
            KeyCode::Down => {
                // Move selection down in the list
                match self.cur_modal {
                    Some(Modal::Path) => {
                        if !self.memory.paths.is_empty() {
                            self.selected_index = match self.selected_index {
                                Some(i) if i >= self.memory.paths.len() - 1 => Some(0),
                                Some(i) => Some(i + 1),
                                None => Some(0),
                            };
                        }
                    }
                    Some(Modal::Filter) => {
                        if !self.memory.filters.is_empty() {
                            self.selected_index = match self.selected_index {
                                Some(i) if i >= self.memory.filters.len() - 1 => Some(0),
                                Some(i) => Some(i + 1),
                                None => Some(0),
                            };
                        }
                    }
                    None => {}
                }
            }
            KeyCode::Enter => {
                // Enter editing mode for the selected item
                if let (Some(_modal), Some(index)) = (&self.cur_modal, self.selected_index) {
                    match _modal {
                        Modal::Path => {
                            if index < self.memory.paths.len() {
                                // Set edit buffer to current path value
                                self.edit_buffer = self.memory.paths[index].path.clone();
                                self.editing_mode = true;
                            }
                        }
                        Modal::Filter => {
                            if index < self.memory.filters.len() {
                                // Set edit buffer based on filter type
                                match &self.memory.filters[index] {
                                    crate::common::enums::Filter::Search(f) => {
                                        self.edit_buffer = f.substr.clone();
                                        self.filter_type = FilterType::Search;
                                        self.editing_mode = true;
                                    },
                                    crate::common::enums::Filter::Regex(f) => {
                                        self.edit_buffer = f.pattern.clone();
                                        self.filter_type = FilterType::Regex;
                                        self.editing_mode = true;
                                    },
                                    crate::common::enums::Filter::Date(f) => {
                                        // Установим значения для датового фильтра
                                        self.filter_type = FilterType::Date;
                                        self.date_filter_type = f.filter_type.clone();
                                        self.date_start_buffer = f.date_start
                                            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%z").to_string())
                                            .unwrap_or_default();
                                        self.date_finish_buffer = f.date_finish
                                            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%z").to_string())
                                            .unwrap_or_default();
                                        self.editing_date_field = DateField::Start;
                                        self.editing_mode = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Char('1') => {
                // Switch to Search filter type or Date filter type 'Before'
                if self.cur_modal == Some(Modal::Filter) {
                    if self.filter_type == FilterType::Date {
                        self.date_filter_type = crate::common::structs::DateFilterType::Before;
                    } else {
                        self.filter_type = FilterType::Search;
                    }
                }
            }
            KeyCode::Char('2') => {
                // Switch to Regex filter type or Date filter type 'After'
                if self.cur_modal == Some(Modal::Filter) {
                    if self.filter_type == FilterType::Date {
                        self.date_filter_type = crate::common::structs::DateFilterType::After;
                    } else {
                        self.filter_type = FilterType::Regex;
                    }
                }
            }
            KeyCode::Char('3') => {
                // Switch to Date filter type or Date filter type 'Between'
                if self.cur_modal == Some(Modal::Filter) {
                    if self.filter_type == FilterType::Date {
                        self.date_filter_type = crate::common::structs::DateFilterType::Between;
                    } else {
                        self.filter_type = FilterType::Date;
                        // Установим начальные значения для датового фильтра
                        self.date_filter_type = crate::common::structs::DateFilterType::Between;
                        self.date_start_buffer = String::new();
                        self.date_finish_buffer = String::new();
                        self.editing_date_field = DateField::Start;
                    }
                }
            }
            _ => {}
        }
    }
    pub fn handle_exit(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('y') => {
                self.exit_approved = true;
            }
            _ => {
                self.cur_screen = Screen::Main;
            }
        }
    }
    pub fn handle_main(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.cur_screen = Screen::Exit;
            }
            KeyCode::Char('p') => {
                self.cur_screen = Screen::Additional;
                self.cur_modal = Some(Modal::Path);
            }
            KeyCode::Char('f') => {
                self.cur_screen = Screen::Additional;
                self.cur_modal = Some(Modal::Filter);
            }
            KeyCode::Char('o') => {
                self.cur_order = match self.cur_order {
                    Order::OrderByDate => Order::OrderByDateReverse,
                    Order::OrderByDateReverse => Order::OrderByDate,
                };
                // Mark that logs need to be refreshed when back in Page mode
                if self.cur_mode == Mode::Page {
                    self.needs_refresh = true;
                }
            }
            KeyCode::Char('m') => {
                self.cur_mode = match self.cur_mode {
                    Mode::Page => Mode::Tail,
                    Mode::Tail => Mode::Page,
                    Mode::Stopped => Mode::Page,
                };
                // Refresh logs when switching to Page mode
                if self.cur_mode == Mode::Page {
                    self.needs_refresh = true;
                }
            }
            KeyCode::Enter => {
                // Reload logs based on current mode:
                // Page mode: reload with current page and size settings
                if self.cur_mode == Mode::Page {
                    self.load_page_logs();
                }
            }
            //Mode::Tail

            //Mode::Page
            _ => {
                match self.cur_mode {
                    Mode::Page => {
                        match key {
                            KeyCode::Char('+') => {
                                // Increase page size
                                self.cur_size = std::cmp::min(self.cur_size.saturating_add(5), 1000);
                                if self.cur_mode == Mode::Page {
                                    self.needs_refresh = true;
                                }
                            }
                            KeyCode::Char('-') => {
                                // Decrease page size
                                self.cur_size = std::cmp::max(self.cur_size.saturating_sub(5), 5);
                                if self.cur_mode == Mode::Page {
                                    self.needs_refresh = true;
                                }
                            }
                            KeyCode::Char('j') => {
                                // Next page
                                self.cur_page += 1;
                                self.load_page_logs();
                            }
                            KeyCode::Char('h') => {
                                // Previous page, but not below 1
                                if self.cur_page > 1 {
                                    self.cur_page -= 1;
                                }
                                self.load_page_logs();
                            }
                            _ => {}
                        }
                    }
                    Mode::Tail => {
                        match key {
                            KeyCode::Char(' ') => {
                                // Stop stream
                                self.cur_mode = Mode::Stopped;
                            }
                            _ => {}
                        }
                    }
                    Mode::Stopped => {
                        match key {
                            KeyCode::Char(' ') => {
                                // Start stream
                                self.cur_mode = Mode::Tail;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    pub fn handle(&mut self, key: KeyCode) {
        match self.cur_screen {
            Screen::Main => self.handle_main(key),
            Screen::Additional => self.handle_additional(key),
            Screen::Exit => self.handle_exit(key),
        }
    }
    pub fn update_logs(&mut self) {
        while let Ok(log) = self.rx.try_recv() {
            self.logs.push(log);
        }
    }

    pub fn load_page_logs(&mut self) {
        // Для корректной загрузки нужной страницы, устанавливаем флаг обновления
        // и система в run_app сама загрузит нужные логи с учетом cur_page
        self.needs_refresh = true;
    }
}

pub enum Screen {
    Main,
    Additional,
    Exit,
}

impl Screen {
    pub fn keys_hint(&self) -> Span<'_> {
        match self {
            Screen::Main => Span::styled(
                "(q) - quit / (f/p) - add [filter/path] / (o/m) - change [order/mode]",
                Style::default().fg(Color::Red),
            ),
            Screen::Additional => {
                Span::styled("(q) - quit to Main / ", Style::default().fg(Color::Red))
            }
            Screen::Exit => Span::styled(
                "(y) - quit / (Any) - back to Main ",
                Style::default().fg(Color::Red),
            ),
        }
    }
}

#[derive(PartialEq)]
pub enum DateField {
    Start,
    Finish,
    None,
}

#[derive(PartialEq)]
pub enum Modal {
    Filter,
    Path,
}

impl Modal {
    pub fn title(&self) -> String {
        match self {
            Modal::Filter => "Create a new filter".to_string(),
            Modal::Path => "Create a new path".to_string(),
        }
    }
}

impl Mode {
    pub fn nav_text(&self) -> Span<'_> {
        match self {
            Mode::Page => Span::styled("Page Mode", Style::default().fg(Color::Green)),
            Mode::Tail => Span::styled("Tail Mode", Style::default().fg(Color::Yellow)),
            Mode::Stopped => Span::styled("Stopped", Style::default().fg(Color::Magenta)),
        }
    }
}

impl Order {
    pub fn order_text(&self) -> Span<'_> {
        match self {
            Order::OrderByDate => Span::styled("ASC Order", Style::default().fg(Color::Blue)),
            Order::OrderByDateReverse => {
                Span::styled("DESC Order", Style::default().fg(Color::LightBlue))
            }
        }
    }
}
