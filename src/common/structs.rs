use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use crate::common::enums::Filter;

use super::*;

#[derive(Serialize, Deserialize)]
pub struct Memory {
    pub paths: Vec<Path>,
    pub filters: Vec<Filter>,
}

#[derive(Clone)]
pub struct Log {
    pub date_time: DateTime<Utc>,
    pub data: String,
    pub source_name: String,
}

pub struct Stream {
    pub batch: Batch,
    tx: Option<UnboundedSender<String>>,
}

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

pub struct Batch {
    size: usize,
    order: enums::Order,
    sources: Vec<Source>, //TODO:: HashMap<Source.path.path: Source>
    filters: Vec<Filter>,
    offset: Option<usize>,
}

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
        return vec![];
    }

    pub fn sort(&self, logs: &mut Vec<Log>) {
        match self.order {
            enums::Order::OrderByDate => logs.sort_by(|a, b| a.date_time.cmp(&b.date_time)),
            enums::Order::OrderByDateReverse => logs.sort_by(|a, b| b.date_time.cmp(&a.date_time)),
        }
    }
}

pub struct Source {
    path: Path,
    size: Option<usize>,
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Path {
    path: String,
    name: String,
}

impl Path {
    pub fn new(path: String, name: String) -> Self {
        Self { path, name }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DateFilter {
    pub date_format: String,
    #[serde(with = "option_datetime_utc")]
    pub date_start: Option<DateTime<Utc>>,
    #[serde(with = "option_datetime_utc")]
    pub date_finish: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct RegexFilter {
    pub pattern: String,
}

#[derive(Serialize, Deserialize)]
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
