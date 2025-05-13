// use std::path::PathBuf;

mod db;
mod html;
mod pdf;
mod server;
// use pdf::{PdfExtractOptions, extract_pdf_to_file};

#[tokio::main]
async fn main() -> () {
    server::run().await;
}
