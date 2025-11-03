use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path as StdPath;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use crate::common::{
    constants::MEMORY_FILE,
    enums::{Filter, MemoryError},
};

use super::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Memory {
    pub paths: Vec<Path>,
    pub filters: Vec<Filter>,
}

#[allow(dead_code)]
impl Memory {
    pub fn load() -> Result<Memory, MemoryError> {
        if !StdPath::new(MEMORY_FILE).exists() {
            return Ok(Memory {
                paths: Vec::new(),
                filters: Vec::new(),
            });
        }

        let json = fs::read_to_string(MEMORY_FILE).map_err(|_| MemoryError::FSError)?;
        let data: Memory = serde_json::from_str(&json).map_err(|_| MemoryError::SerdeError)?;
        Ok(data)
    }

    pub fn save(&self) -> Result<(), MemoryError> {
        let json = serde_json::to_string(self).map_err(|_| MemoryError::SerdeError)?;
        fs::write(MEMORY_FILE, json).map_err(|_| MemoryError::FSError)?;
        Ok(())
    }

    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    pub fn add_path(&mut self, path: Path) {
        self.paths.push(path);
    }

    pub fn remove_filter(&mut self, index: usize) -> Result<(), MemoryError> {
        if index < self.filters.len() {
            self.filters.remove(index);
            Ok(())
        } else {
            Err(MemoryError::FSError) // Index out of bounds
        }
    }

    pub fn remove_path(&mut self, index: usize) -> Result<(), MemoryError> {
        if index < self.paths.len() {
            self.paths.remove(index);
            Ok(())
        } else {
            Err(MemoryError::FSError) // Index out of bounds
        }
    }

    pub fn get_paths(&self) -> &Vec<Path> {
        &self.paths
    }

    pub fn get_filters(&self) -> &Vec<Filter> {
        &self.filters
    }

    pub fn update_path(&mut self, index: usize, path: Path) -> Result<(), MemoryError> {
        if index < self.paths.len() {
            self.paths[index] = path;
            Ok(())
        } else {
            Err(MemoryError::FSError) // Index out of bounds
        }
    }

    pub fn update_filter(&mut self, index: usize, filter: Filter) -> Result<(), MemoryError> {
        if index < self.filters.len() {
            self.filters[index] = filter;
            Ok(())
        } else {
            Err(MemoryError::FSError) // Index out of bounds
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Log {
    pub date_time: DateTime<Utc>,
    pub data: String,
    pub source_name: String,
}

#[allow(dead_code)]
pub struct Stream {
    pub batch: Batch,
    tx: Option<UnboundedSender<String>>,
}

#[allow(dead_code)]
impl Stream {
    pub fn new(batch: Batch, tx: UnboundedSender<String>) -> Self {
        Self {
            batch,
            tx: Some(tx),
        }
    }

    pub async fn send(&self, log: String) -> Result<(), SendError<String>> {
        if let Some(ref tx) = self.tx {
            if self.batch.get_filters().iter().all(|f| f.is_include(&log)) {
                return tx.send(log);
            }
        }
        Ok(())
    }

    pub fn page(&self, logs: &mut Vec<Log>) {
        self.batch.sort(logs);
        if logs.len() > self.batch.size {
            let trimmed_logs = &logs[..self.batch.size];
            *logs = trimmed_logs.to_vec();
        }
    }
}

#[allow(dead_code)]
pub struct Batch {
    size: usize,
    order: enums::Order,
    sources: Vec<Source>,
    filters: Vec<Filter>,
    offset: Option<usize>,
}

#[allow(dead_code)]
impl Batch {
    pub fn new(
        size: usize,
        order: enums::Order,
        sources: Option<Vec<Source>>,
        filters: Option<Vec<Filter>>,
        offset: Option<usize>,
    ) -> Self {
        let s = sources.unwrap_or(Vec::new());
        let f = filters.unwrap_or(Vec::new());
        Self {
            size,
            order,
            sources: s,
            filters: f,
            offset,
        }
    }

    pub fn add_path(&mut self, p: Path) {
        let s = Source {
            path: p,
            size: None,
        };
        self.sources.push(s);
    }

    pub fn len_sources(&self) -> usize {
        self.sources.len()
    }
    pub fn len_filters(&self) -> usize {
        self.filters.len()
    }

    pub fn get_paths(&self) -> Vec<String> {
        let mut paths = vec![];
        for source in self.sources.iter() {
            paths.push(source.get_path());
        }
        return paths;
    }

    pub fn get_filters(&self) -> Vec<Filter> {
        self.filters.clone()
    }

    pub fn sort(&self, logs: &mut Vec<Log>) {
        match self.order {
            enums::Order::OrderByDate => logs.sort_by(|a, b| a.date_time.cmp(&b.date_time)),
            enums::Order::OrderByDateReverse => logs.sort_by(|a, b| b.date_time.cmp(&a.date_time)),
        }
    }
}

#[allow(dead_code)]
pub struct Source {
    path: Path,
    size: Option<usize>,
}

#[allow(dead_code)]
impl Source {
    pub fn new(path: String, name: String) -> Self {
        Self {
            path: Path { path, name },
            size: None,
        }
    }
    pub fn get_name(&self) -> String {
        self.path.name.clone()
    }
    pub fn get_path(&self) -> String {
        self.path.path.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Path {
    pub path: String,
    pub name: String,
}

impl Path {
    pub fn new(path: String, name: String) -> Self {
        Self { path, name }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DateFilter {
    pub date_format: String,
    #[serde(with = "option_datetime_utc")]
    pub date_start: Option<DateTime<Utc>>,
    #[serde(with = "option_datetime_utc")]
    pub date_finish: Option<DateTime<Utc>>,
    pub filter_type: DateFilterType,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum DateFilterType {
    Before,
    After, 
    Between,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RegexFilter {
    pub pattern: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchFilter {
    pub substr: String,
}
mod option_datetime_utc {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => serializer.serialize_some(&d.format(FORMAT).to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => {
                let dt = DateTime::parse_from_str(&s, FORMAT)
                    .map_err(serde::de::Error::custom)?
                    .with_timezone(&Utc);
                Ok(Some(dt))
            }
            None => Ok(None),
        }
    }
}
