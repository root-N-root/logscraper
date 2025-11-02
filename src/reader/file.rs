use std::{
    error::Error,
    fs::File,
    io::{self, BufRead},
};

use crate::common::enums::Filter;
use crate::common::structs::Path;

pub async fn read_lines_from_start(
    path: String,
    limit: usize,
    offset: usize,
    filters: Option<Vec<Filter>>,
) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(path)?; // Чтобы вернуть ошибку даже при limit = 0

    // Ограничиваем начальную ёмкость вектора, чтобы избежать переполнения
    let initial_capacity = std::cmp::min(limit, 1000);
    let mut lines = Vec::with_capacity(initial_capacity);

    if limit < 1 {
        return Ok(lines);
    }

    let reader = io::BufReader::new(file);

    let mut counter: usize = 0;

    for line_result in reader.lines() {
        let line = line_result?;

        if let Some(filter_vec) = &filters {
            if !filter_vec.iter().all(|f| f.is_include(&line)) {
                continue;
            }
        }
        counter += 1;
        if counter <= offset {
            continue;
        }

        lines.push(line);

        if lines.len() >= limit {
            break;
        }
    }
    Ok(lines)
}

use crate::common::enums::Order;

pub async fn read_from_paths(
    paths: Vec<Path>,
    limit: usize,
    offset: usize,
    filters: Option<Vec<Filter>>,
    order: Order,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut all_lines = Vec::new();

    for path in paths {
        let mut file_lines = read_lines_from_start(
            path.path,
            usize::MAX, // Read all lines from this file
            0, // No offset at file level
            filters.clone(),
        ).await?;
        
        all_lines.append(&mut file_lines);
    }

    // Apply sorting based on order
    match order {
        Order::OrderByDate => {
            // For date ascending, we can sort the strings if they start with date
            // This is a simple approach - if logs start with date in sortable format like ISO 8601
            all_lines.sort();
        }
        Order::OrderByDateReverse => {
            // For date descending, sort then reverse
            all_lines.sort();
            all_lines.reverse();
        }
    }

    // Apply the overall limit and offset to the combined results
    let start = offset;
    let end = (offset + limit).min(all_lines.len());
    
    if start < all_lines.len() {
        Ok(all_lines[start..end].to_vec())
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use std::io::{self, Write};

    use crate::common::enums::Filter;
    use crate::common::structs::SearchFilter;
    use crate::reader::file::read_lines_from_start;
    use tempdir::TempDir;

    #[tokio::test]
    async fn read_error() {
        let random_path = random_str::get_string(6, true, false, true, true);
        let res = read_lines_from_start(random_path, 1, 1, None);
        assert!(res.await.is_err())
    }

    #[tokio::test]
    async fn read_empty() {
        let random_path = random_str::get_string(6, true, false, true, true);
        let tmp_dir =
            TempDir::new(&random_path).expect("Не получилось создать временную директорию");
        let file_path = tmp_dir.path().join("test.log");
        File::create(&file_path).expect("Не удалось создать временный файл");
        let res = read_lines_from_start(file_path.to_str().unwrap().to_string(), 0, 0, None)
            .await
            .expect("Не удалось прочитать временный файл");
        assert_eq!(res.len(), 0)
    }

    #[tokio::test]
    async fn read_full() {
        let random_path = random_str::get_string(6, true, false, true, true);
        let tmp_dir =
            TempDir::new(&random_path).expect("Не получилось создать временную директорию");
        let file_path = tmp_dir.path().join("test.log");
        let mut tmp_file = File::create(&file_path).expect("Не удалось создать временный файл");
        writeln!(tmp_file, "test-1").expect("Не удалось записать строку в файл");
        writeln!(tmp_file, "test-2").expect("Не удалось записать строку в файл");
        let file_path = file_path.to_str().unwrap().to_string();

        let res = read_lines_from_start(file_path, 3, 0, None)
            .await
            .expect("Не удалось прочитать временный файл");
        assert_eq!(res.len(), 2)
    }

    #[tokio::test]
    async fn read_full_with_filter() {
        let random_path = random_str::get_string(6, true, false, true, true);
        let tmp_dir =
            TempDir::new(&random_path).expect("Не получилось создать временную директорию");
        let file_path = tmp_dir.path().join("test.log");
        let mut tmp_file = File::create(&file_path).expect("Не удалось создать временный файл");
        writeln!(tmp_file, "test-1").expect("Не удалось записать строку в файл");
        writeln!(tmp_file, "tert-2").expect("Не удалось записать строку в файл");
        writeln!(tmp_file, "test-3").expect("Не удалось записать строку в файл");
        let file_path = file_path.to_str().unwrap().to_string();

        let f = Filter::Search(SearchFilter {
            substr: "test".to_string(),
        });

        let res = read_lines_from_start(file_path, 3, 0, Some(vec![f]))
            .await
            .expect("Не удалось прочитать временный файл");
        assert_eq!(res.len(), 2)
    }

    #[tokio::test]
    async fn read_full_with_filter_and_offset() {
        let random_path = random_str::get_string(6, true, false, true, true);
        let tmp_dir =
            TempDir::new(&random_path).expect("Не получилось создать временную директорию");
        let file_path = tmp_dir.path().join("test.log");
        let mut tmp_file = File::create(&file_path).expect("Не удалось создать временный файл");
        writeln!(tmp_file, "test-1").expect("Не удалось записать строку в файл");
        writeln!(tmp_file, "tert-2").expect("Не удалось записать строку в файл");
        writeln!(tmp_file, "test-3").expect("Не удалось записать строку в файл");
        let file_path = file_path.to_str().unwrap().to_string();

        let f = Filter::Search(SearchFilter {
            substr: "test".to_string(),
        });

        let res = read_lines_from_start(file_path, 3, 1, Some(vec![f]))
            .await
            .expect("Не удалось прочитать временный файл");
        assert_eq!(res.len(), 1)
    }
}
