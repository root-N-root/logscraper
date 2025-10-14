pub mod constants;
pub mod enums;
pub mod structs;
pub mod traits;

#[cfg(test)]
mod test {
    use chrono::{DateTime, Utc};
    use regex::Regex;

    use super::*;

    #[test]
    fn test_sort_empty() {
        let b = structs::Batch::new(10, enums::Order::OrderByDate, None, None);
        let mut logs: Vec<structs::Log> = Vec::new();
        b.sort(&mut logs);
        assert_eq!(logs.len(), 0)
    }

    #[test]
    fn test_sort_normal_order() {
        let b = structs::Batch::new(10, enums::Order::OrderByDate, None, None);
        let mut logs: Vec<structs::Log> = vec![
            structs::Log {
                date_time: DateTime::parse_from_str(
                    "2023-10-28T18:27:11+09:00",
                    "%Y-%m-%dT%H:%M:%S%z",
                )
                .expect("Ошибка формирования даты")
                .with_timezone(&Utc),
                data: "test".to_string(),
                source_name: "test".to_string(),
            },
            structs::Log {
                date_time: DateTime::parse_from_str(
                    "2021-10-28T18:27:11+09:00",
                    "%Y-%m-%dT%H:%M:%S%z",
                )
                .expect("Ошибка формирования даты")
                .with_timezone(&Utc),
                data: "test".to_string(),
                source_name: "test".to_string(),
            },
        ];
        b.sort(&mut logs);
        assert_eq!(logs.len(), 2);

        let first_el = logs[0].date_time;
        let second_el = logs[1].date_time;
        assert!(first_el < second_el);
    }

    #[test]
    fn test_sort_reverse_order() {
        let b = structs::Batch::new(10, enums::Order::OrderByDateReverse, None, None);
        let mut logs: Vec<structs::Log> = vec![
            structs::Log {
                date_time: DateTime::parse_from_str(
                    "2023-10-28T18:27:11+09:00",
                    "%Y-%m-%dT%H:%M:%S%z",
                )
                .expect("Ошибка формирования даты")
                .with_timezone(&Utc),
                data: "test".to_string(),
                source_name: "test".to_string(),
            },
            structs::Log {
                date_time: DateTime::parse_from_str(
                    "2021-10-28T18:27:11+09:00",
                    "%Y-%m-%dT%H:%M:%S%z",
                )
                .expect("Ошибка формирования даты")
                .with_timezone(&Utc),
                data: "test".to_string(),
                source_name: "test".to_string(),
            },
        ];
        b.sort(&mut logs);
        assert_eq!(logs.len(), 2);

        let first_el = logs[0].date_time;
        let second_el = logs[1].date_time;
        assert!(first_el > second_el);
    }

    #[test]
    fn test_batch_push_source() {
        let mut b = structs::Batch::new(10, enums::Order::OrderByDate, None, None);
        let p = structs::Path::new("test".to_string(), "test".to_string());
        b.add_path(p);

        assert_eq!(b.len_sources(), 1)
    }

    #[test]
    fn test_regex_filter_false() {
        let pattern = r"\d{10}".to_string();
        let f = enums::Filter::Regex(structs::RegexFilter { pattern: pattern });
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), false)
    }

    #[test]
    fn test_regex_filter_true() {
        let pattern = r"\d{10}".to_string();
        let f = enums::Filter::Regex(structs::RegexFilter { pattern: pattern });
        let line = "2022-01-08T11:27:44+09:00 test line (1234567890) in log".to_string();
        assert_eq!(f.is_include(&line), true)
    }

    #[test]
    fn test_search_filter_false() {
        let f = enums::Filter::Search(structs::SearchFilter {
            substr: "wrong".to_string(),
        });
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), false)
    }

    #[test]
    fn test_search_filter_true() {
        let f = enums::Filter::Search(structs::SearchFilter {
            substr: "test".to_string(),
        });
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), true)
    }

    #[test]
    fn test_date_filter_false_before() {
        let date = DateTime::parse_from_str("2025-10-08T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
            .expect("Не удалось сформировать DateTime для теста");

        let f = enums::Filter::Date(structs::DateFilter {
            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
            date_start: Some(date.with_timezone(&Utc)),
            date_finish: None,
        });
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), false)
    }

    #[test]
    fn test_date_filter_false_after() {
        let date = DateTime::parse_from_str("2021-10-08T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
            .expect("Не удалось сформировать DateTime для теста");

        let f = enums::Filter::Date(structs::DateFilter {
            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
            date_finish: Some(date.with_timezone(&Utc)),
            date_start: None,
        });
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

        let f = enums::Filter::Date(structs::DateFilter {
            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
            date_start: Some(date_start.with_timezone(&Utc)),
            date_finish: Some(date_finish.with_timezone(&Utc)),
        });
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), false)
    }

    #[test]
    fn test_date_filter_true_empty() {
        let f = enums::Filter::Date(structs::DateFilter {
            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
            date_start: None,
            date_finish: None,
        });
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
        let f = enums::Filter::Date(structs::DateFilter {
            date_format: "%Y-%m-%dT%H:%M:%S%z".to_string(),
            date_start: Some(date_start.with_timezone(&Utc)),
            date_finish: Some(date_finish.with_timezone(&Utc)),
        });
        let line = "2022-01-08T11:27:44+09:00 test line in log".to_string();
        assert_eq!(f.is_include(&line), true)
    }
}
