// use std::path::PathBuf;

mod db;
mod html;
mod pdf;
mod server;
mod video;
// use pdf::{PdfExtractOptions, extract_pdf_to_file};
#[tokio::main]
async fn main() -> () {
    let parsed = video::youtube::grab_video("https://www.youtube.com/watch?v=lil_bBN8QQ0")
        .await
        .expect("Something with youtube");

    for line in parsed.raw_content {
        println!("line: {}", line);
    }

    server::run().await;
}
