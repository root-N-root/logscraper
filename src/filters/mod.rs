use chrono::DateTime;
use chrono::Utc;
use regex::Regex;

use crate::common::traits::Filter;

#[derive(Debug)]
pub enum Filters {
    DateFilter,
    RegexFilter,
    SearchFilter,
}

pub struct DateFilter {
    date_format: String,
    date_start: Option<DateTime<Utc>>,
    date_finish: Option<DateTime<Utc>>,
}

impl Filter for DateFilter {
    fn is_include(&self, line: &String) -> bool {
        let date_str = line.split_whitespace().next().unwrap_or("");
        let parsed_date = match DateTime::parse_from_str(date_str, &self.date_format) {
            Ok(date) => date.with_timezone(&Utc),
            Err(_) => return false,
        };

        if let Some(start) = self.date_start {
            if parsed_date < start {
                return false;
            }
        }

        if let Some(end) = self.date_finish {
            if parsed_date > end {
                return false;
            }
        }
        true
    }
}

pub struct RegexFilter {
    pattern: Regex,
}

impl Filter for RegexFilter {
    fn is_include(&self, line: &String) -> bool {
        self.pattern.is_match(line)
    }
}

pub struct SearchFilter {
    substr: String,
}

impl Filter for SearchFilter {
    fn is_include(&self, line: &String) -> bool {
        line.contains(&self.substr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use regex::Regex;
}
