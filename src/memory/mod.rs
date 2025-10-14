use std::fs;

use serde_json::Result;

use crate::common::constants::{FILTERS_FILE, PATHS_FILE};
use crate::common::structs::Path;

pub fn save_paths(paths: &Vec<Path>) -> Result<()> {
    let json = serde_json::to_string(paths)?;
    fs::write(PATHS_FILE, json).unwrap();
    Ok(())
}

pub fn load_paths() -> Result<Vec<Path>> {
    let json = fs::read_to_string(PATHS_FILE).unwrap();
    let data: Vec<Path> = serde_json::from_str(&json)?;
    Ok(data)
}

//TODO:: load and save filters
