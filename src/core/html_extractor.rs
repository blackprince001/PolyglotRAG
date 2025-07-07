use core::panic;
use html2text::from_read;
use std::{fs::File, io::Write, str::FromStr};
use url::Url;
use uuid::Uuid;

use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ParsedHTML {
    pub content: String,
}

impl ParsedHTML {
    pub fn from(parsed: String) -> Self {
        ParsedHTML { content: parsed }
    }

    pub fn save_to_txt(&self, id: Uuid) -> Result<String, std::io::Error> {
        let file_name = format!("parsed_html_{}.txt", id);
        let mut file = File::create(&file_name)?;

        write!(file, "{}", self.content)?;

        Ok(file_name.clone())
    }
}

pub async fn page_to_text(url: &str, padding: usize) -> ParsedHTML {
    if Url::parse(url).is_err() {
        panic!("something is wrong somewhere")
    }

    let body = reqwest::get(url).await;

    let mut text = String::from_str("");
    if body.is_ok() {
        text = Ok(body
            .unwrap()
            .text()
            .await
            .expect("Could not extract text from html"));
    }

    let content = text.unwrap();
    let html = content.as_bytes();

    ParsedHTML::from(from_read(html, padding).unwrap())
}
