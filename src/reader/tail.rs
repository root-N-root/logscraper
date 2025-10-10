use std::{fs, io::Error, os, path};

use linemux::MuxedLines;
use tokio::sync::mpsc::UnboundedSender;

use crate::common::structs::Batch;

pub async fn read_tail(batch: Batch, tx: UnboundedSender<Vec<String>>) {
    let mut lines = MuxedLines::new().unwrap();

    for path in batch.get_paths().iter() {
        let _ = fs::metadata(&path).unwrap();
        lines.add_file(&path).await.unwrap();
    }
    // tx.send(vec!["TEST".to_string()]).unwrap();

    //TODO::: command for stop
    while let Ok(Some(line)) = lines.next_line().await {
        tx.send(vec!["TEST".to_string()]).unwrap();
        let log_txt = &line.line().to_string();
        // TODO:: match Command -> close tx and exit, update filters
        //
        //LINE has source --- add limit by source (after pack in Vec with timer)
        if batch.get_filters().iter().all(|f| f.is_include(&log_txt)) {
            continue;
        }
        tx.send(vec![log_txt.clone()]).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::time::Duration;
    use std::{fmt::Debug, fs::File};

    use tempdir::TempDir;
    use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

    use crate::common::structs::Source;

    use super::*;

    #[tokio::test]
    #[should_panic]
    async fn test_tail_error_wrong_path() {
        let random_path = random_str::get_string(6, true, false, true, true);

        let batch = Batch::new(
            1,
            crate::common::enums::Order::OrderByDate,
            Some(vec![Source::new(random_path, "test".to_string())]),
            None,
        );
        let (tx, mut rx): (UnboundedSender<Vec<String>>, UnboundedReceiver<Vec<String>>) =
            unbounded_channel();

        let _ = tokio::spawn(read_tail(batch, tx)).await.unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn test_read_one_file() {
        let random_path = random_str::get_string(6, true, false, true, true);

        let tmp_dir =
            TempDir::new(&random_path).expect("Не получилось создать временную директорию");
        let file_path = tmp_dir.path().join("test.log");
        let mut tmp_file = File::create(&file_path).expect("Не удалось создать временный файл");

        let file_path = file_path.to_str().unwrap().to_string();
        let batch = Batch::new(
            1,
            crate::common::enums::Order::OrderByDate,
            Some(vec![Source::new(file_path, "test".to_string())]),
            None,
        );
        let (tx, mut rx): (UnboundedSender<Vec<String>>, UnboundedReceiver<Vec<String>>) =
            unbounded_channel();

        let _ = tokio::spawn(async move {
            read_tail(batch, tx).await;
        });

        writeln!(tmp_file, "test-1").expect("Не удалось записать строку в файл");
        //TODO:: set timeout and send closeCh
        // tokio::time::sleep(Duration::from_secs(12)).await;

        tokio::select! {
            logs = rx.recv() => {
                assert_eq!(logs.unwrap().len(), 1);
            }
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    assert_eq!(false, true);
            }
        }

        let logs = rx.try_recv().unwrap();
        assert_eq!(logs[0], "TEST");
    }
}
