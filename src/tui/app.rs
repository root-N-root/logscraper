pub struct App {
    //TODO
    pub cur_screen: Screen,
    pub cur_modal: Option<Modal>,
}

impl App {
    pub fn new() -> App {
        App {
            cur_screen: Screen::Main,
            cur_modal: None,
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
    Log,
    Exit,
}
