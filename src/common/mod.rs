use chrono::{DateTime, Utc};
pub mod enums;
pub mod structs;
pub mod traits;

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_sort_empty() {
        let b = structs::Batch::new(10, enums::Order::OrderByDate, None, None);
        let mut logs: Vec<Box<dyn traits::LogTrait>> = Vec::new();
        b.sort(&mut logs);
        assert_eq!(logs.len(), 0)
    }

    #[test]
    fn test_sort_normal_order() {
        let b = structs::Batch::new(10, enums::Order::OrderByDate, None, None);
        let mut logs: Vec<Box<dyn traits::LogTrait>> = vec![
            Box::new(MockLog {
                date: DateTime::parse_from_str("2023-10-28T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
                    .expect("Ошибка формирования даты")
                    .with_timezone(&Utc),
            }),
            Box::new(MockLog {
                date: DateTime::parse_from_str("2021-10-28T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
                    .expect("Ошибка формирования даты")
                    .with_timezone(&Utc),
            }),
        ];
        b.sort(&mut logs);
        assert_eq!(logs.len(), 2);

        let first_el = logs[0].get_date();
        let second_el = logs[1].get_date();
        assert!(first_el < second_el);
    }

    #[test]
    fn test_sort_reverse_order() {
        let b = structs::Batch::new(10, enums::Order::OrderByDateReverse, None, None);
        let mut logs: Vec<Box<dyn traits::LogTrait>> = vec![
            Box::new(MockLog {
                date: DateTime::parse_from_str("2023-10-28T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
                    .expect("Ошибка формирования даты")
                    .with_timezone(&Utc),
            }),
            Box::new(MockLog {
                date: DateTime::parse_from_str("2021-10-28T18:27:11+09:00", "%Y-%m-%dT%H:%M:%S%z")
                    .expect("Ошибка формирования даты")
                    .with_timezone(&Utc),
            }),
        ];
        b.sort(&mut logs);
        assert_eq!(logs.len(), 2);

        let first_el = logs[0].get_date();
        let second_el = logs[1].get_date();
        assert!(first_el > second_el);
    }

    #[test]
    fn test_batch_push_source() {
        let mut b = structs::Batch::new(10, enums::Order::OrderByDate, None, None);
        let p = structs::Path::new("test".to_string(), "test".to_string());
        b.add_path(p);

        assert_eq!(b.len_sources(), 1)
    }

    struct MockLog {
        date: DateTime<Utc>,
    }
    impl traits::LogTrait for MockLog {
        fn get_date(&self) -> DateTime<Utc> {
            self.date
        }
    }
}
