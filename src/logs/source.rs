use crate::common::Filter;

use super::*;

// pub struct Pack ?

pub struct Source {
    paths: Vec<Path>,
    filters: Vec<Box<dyn Filter>>,
}

impl Source {
    pub fn new(paths: Vec<Path>, filters: Vec<Box<dyn Filter>>) -> Source {
        Self { paths, filters }
    }

    // pub fn filter(&self, log: &Log) -> bool {
    //     self.filters.iter().all(|f| f.is_include(log))
    // }
}

pub struct Path {
    path: String,
    name: String,
}
