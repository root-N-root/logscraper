use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::common::{structs::Batch, traits::LogTrait};

pub mod command;

pub fn start(batch: Batch) {
    // let (tx, mut rx): (
    //     UnboundedSender<&Vec<String>,
    //     UnboundedReceiver<&Vec<String>,
    // ) = mpsc::unbounded_channel();
}
