use chrono::{DateTime, Utc};

use crate::common::traits::LogTrait;

mod filter;

pub struct Log {
    date_time: DateTime<Utc>,
    data: String,
    source_name: String,
}

impl LogTrait for Log {
    fn get_date(&self) -> DateTime<Utc> {
        return self.date_time;
    }
}
