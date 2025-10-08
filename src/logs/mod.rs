mod filter;
mod source;

pub struct Log {
    date_time: String,
    data: String,
    source: source::Source,
    date_format: Option<String>,
}

impl Log {
    fn get_date_format(&self) -> String {
        self.date_format
            .clone()
            .unwrap_or_else(|| "%Y-%m-%d %H:%M:%S %z".to_string())
    }
}
