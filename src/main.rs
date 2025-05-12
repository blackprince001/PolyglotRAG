use std::path::PathBuf;

mod db;
mod html;
mod pdf;
use pdf::{PdfExtractOptions, extract_pdf_to_file};

#[tokio::main]
async fn main() {
    let some_page = "https://blackprince.tech/blog/floating";
    let pad = 128_usize;

    let result = html::page_to_text(some_page, pad);

    println!("{}", result.await.content);

    let pdf_path =
        PathBuf::from("/Users/blackprince/Documents/Machine Learning Notes/Chapter - 8.pdf");

    let output_json = PathBuf::from("extracted_text.json");

    let options = PdfExtractOptions {
        password: String::new(),
        pretty_json: true,
    };

    let saved_text = extract_pdf_to_file(&pdf_path, &output_json, Some(options.clone()));

    println!(
        "Number of pages we have: {}",
        saved_text
            .expect("Could not extract file.")
            .number_of_pages()
    );
}
