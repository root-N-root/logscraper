use chrono::DateTime;
use chrono::Utc;
use regex::Regex;

use crate::common::traits::Filter;

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

    #[test]
    fn test_regex_filter_false() {
        let pattern = Regex::new(r"\d{10}").unwrap();
        let f = RegexFilter { pattern: pattern };
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), false)
    }

    #[test]
    fn test_regex_filter_true() {
        let pattern = Regex::new(r"\d{10}").unwrap();
        let f = RegexFilter { pattern: pattern };
        let line = "2022-01-08T11:27:44+09:00 test line (1234567890) in log".to_string();
        assert_eq!(f.is_include(&line), true)
    }

    #[test]
    fn test_search_filter_false() {
        let f = SearchFilter {
            substr: "wrong".to_string(),
        };
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), false)
    }

    #[test]
    fn test_search_filter_true() {
        let f = SearchFilter {
            substr: "test".to_string(),
        };
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), true)
    }

    #[test]
    fn test_date_filter_false_before() {
        let date = DateTime::parse_from_str("2025-10-08T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
            .expect("Не удалось сформировать DateTime для теста");
        let f = DateFilter {
            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
            date_start: Some(date.with_timezone(&Utc)),
            date_finish: None,
        };
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), false)
    }

    #[test]
    fn test_date_filter_false_after() {
        let date = DateTime::parse_from_str("2021-10-08T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
            .expect("Не удалось сформировать DateTime для теста");
        let f = DateFilter {
            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
            date_finish: Some(date.with_timezone(&Utc)),
            date_start: None,
        };
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), false)
    }

    #[test]
    fn test_date_filter_false_on_one_param() {
        let date_start =
            DateTime::parse_from_str("2021-10-08T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
                .expect("Не удалось сформировать DateTime для теста");
        let date_finish =
            DateTime::parse_from_str("2021-10-18T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
                .expect("Не удалось сформировать DateTime для теста");
        let f = DateFilter {
            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
            date_start: Some(date_start.with_timezone(&Utc)),
            date_finish: Some(date_finish.with_timezone(&Utc)),
        };
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), false)
    }

    #[test]
    fn test_date_filter_true_empty() {
        let f = DateFilter {
            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
            date_start: None,
            date_finish: None,
        };
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), true)
    }

    #[test]
    fn test_date_filter_true_before_and_after() {
        let date_start =
            DateTime::parse_from_str("2021-10-08T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
                .expect("Не удалось сформировать DateTime для теста");
        let date_finish =
            DateTime::parse_from_str("2023-10-28T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
                .expect("Не удалось сформировать DateTime для теста");
        let f = DateFilter {
            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
            date_start: Some(date_start.with_timezone(&Utc)),
            date_finish: Some(date_finish.with_timezone(&Utc)),
        };
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), true)
    }
}
