use std::fs::read_dir;

#[allow(dead_code)]
pub fn find_files(path: String) -> Vec<String> {
    let mut list = Vec::new();
    for entry in read_dir(path).unwrap() {
        let entry = entry.unwrap();
        list.push(entry.file_name().into_string().unwrap());
    }
    list
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use tempdir::TempDir;

    use crate::reader::find::find_files;

    #[test]
    #[should_panic]
    fn test_error() {
        let random_path = random_str::get_string(6, true, false, true, true);
        let res = find_files(random_path);
        assert_eq!(res.len(), 0)
    }

    #[test]
    fn test_empty_dir() {
        let random_path = random_str::get_string(6, true, false, true, true);
        let tmp_dir =
            TempDir::new(&random_path).expect("Не получилось создать временную директорию");
        let res = find_files(tmp_dir.path().to_str().unwrap().to_string());
        assert_eq!(res.len(), 0)
    }

    #[test]
    fn test_full_dir() {
        let test_files: [&str; 3] = ["test-1", "test-2", "test-3"];
        let random_path = random_str::get_string(6, true, false, true, true);
        let tmp_dir =
            TempDir::new(&random_path).expect("Не получилось создать временную директорию");
        for tf in test_files.iter() {
            let file_path = tmp_dir.path().join(tf);
            File::create(file_path).expect("Не удалось создать временный файл");
        }
        let res = find_files(tmp_dir.path().to_str().unwrap().to_string());
        assert_eq!(res.len(), test_files.len())
    }
}
