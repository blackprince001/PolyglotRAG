// use std::path::PathBuf;

mod db;
mod html;
mod pdf;
mod server;
mod service;
// mod video;
// use pdf::{PdfExtractOptions, extract_pdf_to_file};
#[tokio::main]
async fn main() -> () {
    let client = service::inference::InferenceClient::from_env()
        .expect("Failed to initialize inference client");

    let single_result = client
        .get_embedding("This is a sample text to embed.")
        .await
        .expect("Failed to get_embedding");
    println!("Single embedding shape: {:?}", single_result.embeddings);

    server::run().await;
}
