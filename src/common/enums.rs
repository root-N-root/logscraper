use std::{error::Error, fmt};

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::common::structs::{DateFilter, RegexFilter, SearchFilter};

#[derive(Debug)]
pub enum MemoryError {
    FSError,
    SerdeError,
}
impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::FSError => write!(f, "File system error"),
            MemoryError::SerdeError => write!(f, "Serialization/deserialization error"),
        }
    }
}

impl Error for MemoryError {}

#[derive(PartialEq, Clone)]
pub enum Mode {
    Page,
    Tail,
    Stopped,
}

#[derive(PartialEq, Clone)]
pub enum Order {
    OrderByDate,
    OrderByDateReverse,
}

// Фильтры в один модуль
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Filter {
    Date(DateFilter),
    Regex(RegexFilter),
    Search(SearchFilter),
}

impl Filter {
    pub fn is_include(&self, line: &String) -> bool {
        match self {
            Filter::Date(f) => {
                let date_str = line.split_whitespace().next().unwrap_or("");
                let parsed_date = match DateTime::parse_from_str(date_str, &f.date_format) {
                    Ok(date) => date.with_timezone(&Utc),
                    Err(_) => return false,
                };

                match f.filter_type {
                    crate::common::structs::DateFilterType::Before => {
                        if let Some(end) = f.date_finish {
                            if parsed_date > end {
                                return false;
                            }
                        }
                    }
                    crate::common::structs::DateFilterType::After => {
                        if let Some(start) = f.date_start {
                            if parsed_date < start {
                                return false;
                            }
                        }
                    }
                    crate::common::structs::DateFilterType::Between => {
                        if let Some(start) = f.date_start {
                            if parsed_date < start {
                                return false;
                            }
                        }
                        if let Some(end) = f.date_finish {
                            if parsed_date > end {
                                return false;
                            }
                        }
                    }
                }
                true
            }
            Filter::Regex(f) => {
                let re = Regex::new(&f.pattern).expect("Invalid regex in filter");
                re.is_match(line)
            }
            Filter::Search(f) => line.contains(&f.substr),
        }
    }
}
