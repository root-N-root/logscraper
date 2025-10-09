use chrono::{DateTime, Utc};

pub trait Filter {
    fn is_include(&self, line: &String) -> bool;
}

pub trait LogTrait {
    fn get_date(&self) -> DateTime<Utc>;
}

pub trait Reader {
    async fn page();
    async fn tail();
}
