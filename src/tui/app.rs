use ratatui::{
    crossterm::event::KeyCode,
    style::{Color, Style},
    text::Span,
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::common::enums::{Mode, Order};
use crate::common::structs::Memory;

pub struct App {
    //TODO
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
    // Fields for managing modal states
    pub selected_index: Option<usize>, // Index of selected item in modal lists
    pub editing_mode: bool,            // Whether we're currently editing an item
    pub edit_buffer: String,           // Buffer for text input during editing
    pub filter_type: FilterType,       // Current filter type when adding/editing filters
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    Date,
    Regex,
    Search,
}

impl App {
    pub fn new(rx: UnboundedReceiver<String>, memory: Memory) -> App {
        App {
            cur_screen: Screen::Main,
            cur_modal: None,
            cur_order: Order::OrderByDate,
            cur_mode: Mode::Page,
            cur_size: 10,
            exit_approved: false,
            cur_page: 1,
            logs: Vec::new(),
            rx,
            memory,
            selected_index: None,
            editing_mode: false,
            edit_buffer: String::new(),
            filter_type: FilterType::Search,
        }
    }
    pub fn handle_additional(&mut self, key: KeyCode) {
        if self.editing_mode {
            // Handle text input when in editing mode
            match key {
                KeyCode::Enter => {
                    // Save the edited value
                    if let (Some(modal), Some(index)) = (&self.cur_modal, self.selected_index) {
                        match modal {
                            Modal::Path => {
                                if index < self.memory.paths.len() {
                                    // For now, just update the path field with the edit_buffer
                                    // In a real implementation, you'd want separate fields for path and name
                                    let updated_path = crate::common::structs::Path::new(
                                        self.edit_buffer.clone(), 
                                        self.edit_buffer.clone() // Using same value for name as placeholder
                                    );
                                    let _ = self.memory.update_path(index, updated_path);
                                }
                            }
                            Modal::Filter => {
                                if index < self.memory.filters.len() {
                                    // For now, just update based on current filter type
                                    let updated_filter = match self.filter_type {
                                        FilterType::Search => crate::common::enums::Filter::Search(
                                            crate::common::structs::SearchFilter {
                                                substr: self.edit_buffer.clone(),
                                            }
                                        ),
                                        FilterType::Regex => crate::common::enums::Filter::Regex(
                                            crate::common::structs::RegexFilter {
                                                pattern: self.edit_buffer.clone(),
                                            }
                                        ),
                                        FilterType::Date => {
                                            // Date filters are more complex, using a placeholder
                                            crate::common::enums::Filter::Search(
                                                crate::common::structs::SearchFilter {
                                                    substr: self.edit_buffer.clone(),
                                                }
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
                }
                KeyCode::Char(c) => {
                    self.edit_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.edit_buffer.pop();
                }
                KeyCode::Esc => {
                    // Cancel editing
                    self.editing_mode = false;
                    self.edit_buffer.clear();
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
                        self.memory.add_path(crate::common::structs::Path::new("".to_string(), "".to_string()));
                        // Select the newly added item
                        self.selected_index = Some(self.memory.paths.len().saturating_sub(1));
                    }
                    Some(Modal::Filter) => {
                        // Add a new empty search filter by default
                        let new_filter = crate::common::enums::Filter::Search(
                            crate::common::structs::SearchFilter {
                                substr: "".to_string(),
                            }
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
                                if index >= self.memory.paths.len() && !self.memory.paths.is_empty() {
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
                                if index >= self.memory.filters.len() && !self.memory.filters.is_empty() {
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
                if let (Some(modal), Some(index)) = (&self.cur_modal, self.selected_index) {
                    match modal {
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
                                self.edit_buffer = match &self.memory.filters[index] {
                                    crate::common::enums::Filter::Search(f) => f.substr.clone(),
                                    crate::common::enums::Filter::Regex(f) => f.pattern.clone(),
                                    crate::common::enums::Filter::Date(_) => "".to_string(), // Date filters are more complex
                                };
                                self.editing_mode = true;
                            }
                        }
                    }
                }
            }
            KeyCode::Char('1') => {
                // Switch to Search filter type
                if self.cur_modal == Some(Modal::Filter) {
                    self.filter_type = FilterType::Search;
                }
            }
            KeyCode::Char('2') => {
                // Switch to Regex filter type
                if self.cur_modal == Some(Modal::Filter) {
                    self.filter_type = FilterType::Regex;
                }
            }
            KeyCode::Char('3') => {
                // Switch to Date filter type
                if self.cur_modal == Some(Modal::Filter) {
                    self.filter_type = FilterType::Date;
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
                }
            }
            KeyCode::Char('m') => {
                self.cur_mode = match self.cur_mode {
                    Mode::Page => Mode::Tail,
                    Mode::Tail => Mode::Page,
                    Mode::Stopped => Mode::Page,
                }
            }
            KeyCode::Enter => {
                //TODO:: search by mode:
                //Page (offset = cur_size * cur_page)
            }
            //Mode::Tail

            //Mode::Page
            _ => {
                match self.cur_mode {
                    Mode::Page => {
                        match key {
                            KeyCode::Char('j') => {
                                //TODO:: new search
                                self.cur_page += 1
                            }

                            KeyCode::Char('h') => {
                                //TODO:: new search
                                self.cur_page -= 1
                            }
                            _ => {}
                        }
                    }
                    Mode::Tail => {
                        match key {
                            KeyCode::Char(' ') => {
                                //TODO:: stop stream
                                self.cur_mode = Mode::Stopped;
                            }
                            _ => {}
                        }
                    }
                    Mode::Stopped => {
                        match key {
                            KeyCode::Char(' ') => {
                                //TODO:: start stream
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
}

pub enum Screen {
    Main,
    Additional,
    Exit,
}

impl Screen {
    pub fn keys_hint(&self) -> Span {
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
    pub fn nav_text(&self) -> Span {
        match self {
            Mode::Page => Span::styled("Page Mode", Style::default().fg(Color::Green)),
            Mode::Tail => Span::styled("Tail Mode", Style::default().fg(Color::Yellow)),
            Mode::Stopped => Span::styled("Stopped", Style::default().fg(Color::Magenta)),
        }
    }
}

impl Order {
    pub fn order_text(&self) -> Span {
        match self {
            Order::OrderByDate => Span::styled("ASC Order", Style::default().fg(Color::Blue)),
            Order::OrderByDateReverse => {
                Span::styled("DESC Order", Style::default().fg(Color::LightBlue))
            }
        }
    }
}
