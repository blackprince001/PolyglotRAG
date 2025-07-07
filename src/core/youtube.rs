use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;

use yt_transcript_rs::api::YouTubeTranscriptApi;

use url::Url;

use crate::server::errors::AppError;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ParsedYoutubeVideo {
    pub title: String,
    pub author: String,
    pub channel_id: String,
    pub video_id: String,
    pub short_description: String,
    pub duration: u32,
    pub timestamped_content: Vec<String>,
    pub raw_content: Vec<String>,
}

impl ParsedYoutubeVideo {
    pub fn new() -> Self {
        ParsedYoutubeVideo::default()
    }

    pub fn save_to_json<P: AsRef<Path>>(&self, path: P, pretty: bool) -> Result<(), Error> {
        let data = match pretty {
            true => serde_json::to_string_pretty(self).unwrap(),
            false => serde_json::to_string(self).unwrap(),
        };

        let mut file = File::create(path)?;
        file.write_all(data.as_bytes())?;

        Ok(())
    }

    pub fn save_to_txt<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut file = File::create(path)?;

        for line in &self.raw_content {
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    pub fn video_duration(&self) -> u32 {
        self.duration
    }
}

pub async fn grab_video(youtube_video_link: &str) -> Result<ParsedYoutubeVideo, AppError> {
    let link = Url::parse(youtube_video_link).expect("Invalid URL");

    let video_dets = link
        .query_pairs()
        .next()
        .expect("Failed to extract youtube video id");

    let api = YouTubeTranscriptApi::new(None, None, None).expect("Failed to setup api");

    let (_, video_id) = video_dets;

    let languages = &["en"];

    let preserve_formatting = false;

    let details = api
        .fetch_video_details(&video_id)
        .await
        .expect("Failed to fetch video details");

    let mut content = vec![];
    let mut timestamped_content = vec![];

    match api
        .fetch_transcript(&video_id, languages, preserve_formatting)
        .await
    {
        Ok(transcript) => {
            if transcript.snippets.is_empty() {
                return Err(AppError::YoutubeExtractionError(
                    "Video has no transcripts yet".to_string(),
                ));
            }

            for (_, snippet) in transcript.snippets.iter().enumerate() {
                content.push(format!("{}", snippet.text));

                timestamped_content.push(format!(
                    "[{:.1}-{:.1}s] {}",
                    snippet.start,
                    snippet.start + snippet.duration,
                    snippet.text
                ))
            }

            let mut youtube = ParsedYoutubeVideo::new();

            youtube.title = details.title;
            youtube.author = details.author;
            youtube.channel_id = details.channel_id;
            youtube.video_id = video_id.to_string();
            youtube.short_description = details.short_description;
            youtube.duration = details.length_seconds;

            youtube.raw_content = content;
            youtube.timestamped_content = timestamped_content;

            Ok(youtube)
        }
        Err(e) => Err(AppError::YoutubeExtractionError(e.to_string())),
    }
}
