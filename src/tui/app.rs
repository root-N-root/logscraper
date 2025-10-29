use ratatui::crossterm::event::KeyCode;

use crate::common::enums::{Mode, Order};

pub struct App {
    //TODO
    pub cur_screen: Screen,
    pub cur_modal: Option<Modal>,
    pub cur_order: Order,
    pub cur_mode: Mode,
    pub cur_size: usize,
    pub cur_page: usize,
}

impl App {
    pub fn new() -> App {
        App {
            cur_screen: Screen::Main,
            cur_modal: None,
            cur_order: Order::OrderByDate,
            cur_mode: Mode::Page,
            cur_size: 10,
            cur_page: 1,
        }
    }
    pub fn handle_additional(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.cur_screen = Screen::Main;
                self.cur_modal = None;
            }
            _ => match self.cur_modal {
                Some(Modal::Path) => {
                    match key {
                        _ => {
                            //TODO:: add new path
                        }
                    }
                }
                Some(Modal::Filter) => {
                    match key {
                        _ => {
                            //TODO:: add new path
                        }
                    }
                }
                None => {}
            },
        }
    }
    pub fn handle_exit(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('y') => {}
            _ => {}
        }
    }
    pub fn handle_main(&mut self, key: KeyCode) {
        match key {
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
}

pub enum Screen {
    Main,
    Additional,
    Exit,
}

pub enum Modal {
    Filter,
    Path,
}
