use std::fs;

use crate::common::enums::Filter;
use crate::common::structs::{Memory, Path};

use crate::common::constants::MEMORY_FILE;

#[derive(Debug)]
pub enum MemoryError {
    FSError,
    SerdeError,
}

pub fn load() -> Result<Memory, MemoryError> {
    let json = fs::read_to_string(MEMORY_FILE).map_err(|_| MemoryError::FSError)?;
    let data: Memory = serde_json::from_str(&json).map_err(|_| MemoryError::SerdeError)?;
    Ok(data)
}

pub fn save(memory: &Memory) -> Result<(), MemoryError> {
    let json = serde_json::to_string(memory).map_err(|_| MemoryError::SerdeError)?;
    fs::write(MEMORY_FILE, json).map_err(|_| MemoryError::FSError)?;
    Ok(())
}

pub fn add_filter(filter: Filter) -> Result<(), MemoryError> {
    let mut mem = load()?;
    mem.filters.push(filter);
    Ok(())
}

pub fn add_path(path: Path) -> Result<(), MemoryError> {
    let mut mem = load()?;
    mem.paths.push(path);
    Ok(())
}
