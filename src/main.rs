// use std::path::PathBuf;

use crate::service::scheduler::Scheduler;

mod db;
mod html;
mod pdf;
mod server;
mod service;
// mod video;
// use pdf::{PdfExtractOptions, extract_pdf_to_file};
#[tokio::main]
async fn main() -> () {
    let (scheduler, processor) = Scheduler::new();

    let _ = tokio::spawn(async move {
        processor.start_processing().await;
    });

    server::run(scheduler).await;
}
