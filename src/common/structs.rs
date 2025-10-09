use super::*;

pub struct Batch {
    size: usize,
    order: enums::Order,
    sources: Vec<Source>,
    filters: Vec<Box<dyn traits::Filter>>,
}

impl Batch {
    pub fn new(
        size: usize,
        order: enums::Order,
        sources: Option<Vec<Source>>,
        filters: Option<Vec<Box<dyn traits::Filter>>>,
    ) -> Batch {
        let s = sources.unwrap_or(Vec::new());
        let f = filters.unwrap_or(Vec::new());
        Self {
            size: size,
            order: order,
            sources: s,
            filters: f,
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

    pub fn sort(&self, logs: &mut Vec<Box<dyn traits::LogTrait>>) {
        match self.order {
            enums::Order::OrderByDate => logs.sort_by(|a, b| a.get_date().cmp(&b.get_date())),
            enums::Order::OrderByDateReverse => {
                logs.sort_by(|a, b| b.get_date().cmp(&a.get_date()))
            }
        }
    }
}

pub struct Source {
    path: Path,
    size: Option<usize>,
}

impl Source {
    fn get_name(&self) -> String {
        self.path.name.clone()
    }
}

pub struct Path {
    path: String,
    name: String,
}

impl Path {
    pub fn new(path: String, name: String) -> Self {
        Self { path, name }
    }
}
