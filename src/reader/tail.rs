use std::{collections::HashMap, error::Error, fs, path::PathBuf};

use notify::{Event, EventKind, Watcher, recommended_watcher};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncSeekExt, BufReader},
    sync::mpsc::{self, UnboundedSender},
};

use crate::common::structs::{Memory, Stream};

#[derive(Debug)]
struct TrackedFile {
    reader: BufReader<File>,
    position: u64,
}

#[allow(dead_code)]
pub async fn tail(stream: Stream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let paths: Vec<PathBuf> = stream.batch.get_paths().iter().map(PathBuf::from).collect();

    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel::<PathBuf>();

    let mut watcher = recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_)) {
                for path in event.paths {
                    let _ = notify_tx.send(path);
                }
            }
        }
    })?;
    // Проверка использования notify_tx --- жизненный цикл с замыканием move

    let mut tracked_files: HashMap<PathBuf, TrackedFile> = HashMap::new();
    for path in &paths {
        let metadata = fs::metadata(path).unwrap();
        watcher.watch(path, notify::RecursiveMode::NonRecursive)?;

        let file = File::open(path).await?;
        // let metadata = file.metadata().await?;

        let mut reader = BufReader::new(file);
        let size = metadata.len();

        //GO to end
        reader.seek(std::io::SeekFrom::Start(size)).await?;
        tracked_files.insert(
            path.clone().to_path_buf(),
            TrackedFile {
                reader,
                position: size,
            },
        );
    }

    loop {
        tokio::select! {
            Some(changed_path) = notify_rx.recv() => {

                if let Some(tracked) = tracked_files.get_mut(&changed_path) {
                    read_new_lines(&mut tracked.reader, &mut tracked.position, &stream).await?;
                }
            }
        }
    }
}

pub async fn tail_stream(
    memory: Memory,
    tx: UnboundedSender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let paths: Vec<PathBuf> = memory.paths.iter().map(|p| PathBuf::from(&p.path)).collect();
    let filters = memory.filters;

    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel::<PathBuf>();

    let mut watcher = recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_)) {
                for path in event.paths {
                    let _ = notify_tx.send(path);
                }
            }
        }
    })?;

    let mut tracked_files: HashMap<PathBuf, TrackedFile> = HashMap::new();
    for path in &paths {
        let metadata = fs::metadata(path).unwrap();
        watcher.watch(path, notify::RecursiveMode::NonRecursive)?;

        let file = File::open(path).await?;
        let size = metadata.len();

        let mut reader = BufReader::new(file);
        reader.seek(std::io::SeekFrom::Start(size)).await?;
        tracked_files.insert(
            path.clone().to_path_buf(),
            TrackedFile {
                reader,
                position: size,
            },
        );
    }

    loop {
        tokio::select! {
            Some(changed_path) = notify_rx.recv() => {
                if let Some(tracked) = tracked_files.get_mut(&changed_path) {
                    read_new_lines_with_filters(&mut tracked.reader, &mut tracked.position, &tx, &filters).await?;
                }
            }
        }
    }
}

#[allow(dead_code)]
async fn read_new_lines(
    reader: &mut BufReader<File>,
    position: &mut u64,
    stream: &Stream,
) -> Result<(), std::io::Error> {
    let mut buf = String::new();
    loop {
        let bytes_read = reader.read_line(&mut buf).await?;

        if bytes_read == 0 {
            break;
        }

        if !buf.ends_with('\n') {
            break;
        }

        let line = buf.trim_end_matches('\n').to_string();
        *position += bytes_read as u64;

        if let Err(err) = stream.send(line).await {
            eprintln!("Error sending to stream: {:?}", err);
        }
        buf.clear();
    }
    Ok(())
}

async fn read_new_lines_with_filters(
    reader: &mut BufReader<File>,
    position: &mut u64,
    tx: &UnboundedSender<String>,
    filters: &[crate::common::enums::Filter],
) -> Result<(), std::io::Error> {
    let mut buf = String::new();
    loop {
        let bytes_read = reader.read_line(&mut buf).await?;

        if bytes_read == 0 {
            break;
        }

        if !buf.ends_with('\n') {
            break;
        }

        let line = buf.trim_end_matches('\n').to_string();
        
        // Apply filters - only send line if it passes all filters
        let should_send = filters.iter().all(|f| f.is_include(&line));
        
        if should_send {
            if let Err(_) = tx.send(line.clone()) {
                // Channel closed, stop reading
                break;
            }
        }
        
        *position += bytes_read as u64;
        buf.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        io::{self, Write},
        time::Duration,
    };
    use tempdir::TempDir;
    use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

    use crate::common::structs::{Batch, Source};

    use super::*;

    #[tokio::test]
    #[should_panic]
    async fn test_live_tail_error_path() {
        let random_path = random_str::get_string(6, true, false, true, true);

        let (tx, mut rx) = unbounded_channel::<String>();
        let stream = Stream::new(
            Batch::new(
                1,
                crate::common::enums::Order::OrderByDate,
                Some(vec![Source::new(random_path, "test".to_string())]),
                None,
                None,
            ),
            tx,
        );
        let _ = tokio::spawn(tail(stream)).await.unwrap();
    }

    #[tokio::test]
    async fn test_live_tail_success() {
        let random_path = random_str::get_string(6, true, false, true, true);
        let tmp_dir = TempDir::new(&random_path).expect("не удалось создать временную директорию");
        let file_path = tmp_dir.path().join("test.log");
        std::fs::write(&file_path, "").unwrap();

        let (tx, mut rx) = unbounded_channel::<String>();
        let stream = Stream::new(
            Batch::new(
                1,
                crate::common::enums::Order::OrderByDate,
                Some(vec![Source::new(
                    file_path.to_str().unwrap().to_string(),
                    "test".to_string(),
                )]),
                None,
                None,
            ),
            tx,
        );
        let handle = tokio::spawn(async move {
            let _ = tail(stream).await;
        });

        tokio::time::sleep(Duration::from_millis(200)).await;

        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&file_path)
            .unwrap();

        writeln!(f, "new test log").unwrap();
        drop(f);

        tokio::time::sleep(Duration::from_millis(200)).await;
        let msg = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(msg, "new test log");
        handle.abort();
    }
}
