use std::fs;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    db::{
        get_database_connection,
        models::{self},
    },
    server::errors::AppError,
    service::{
        inference::InferenceClient,
        scheduler::{FileEvent, ProcessingStatus, ProcessingUpdate},
        semantic_chunking::{RTSplitter, RecursiveTextSplitter},
    },
};

pub struct FileProcessor {
    pub file_receiver: mpsc::UnboundedReceiver<FileEvent>,
    pub update_receiver: mpsc::UnboundedReceiver<ProcessingUpdate>,
    pub update_sender: mpsc::UnboundedSender<ProcessingUpdate>,
    pub embedder: InferenceClient,
    pub splitter: RTSplitter,
}

impl FileProcessor {
    pub async fn start_processing(mut self) {
        println!("File processor started, waiting for events...");

        while let Some(file_event) = self.file_receiver.recv().await {
            println!("Received file event: {:?}", file_event);
            let file_id = file_event.id;

            let update = ProcessingUpdate {
                file_id,
                status: ProcessingStatus::Processing,
                progress: Some(0.1),
                message: Some("Started processing file".to_string()),
                timestamp: std::time::SystemTime::now(),
            };
            let _ = self.update_sender.send(update);

            let update_sender = self.update_sender.clone();
            let embedder = self.embedder.clone();
            let splitter = self.splitter.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::process_file_with_deps(
                    embedder,
                    splitter,
                    file_id,
                    file_event,
                    update_sender,
                )
                .await
                {
                    eprintln!("Error processing file: {}", e);
                }
            });
        }

        println!("File processor stopped");
    }

    async fn process_file_with_deps(
        embedder: InferenceClient,
        splitter: RTSplitter,
        file_id: Uuid,
        file_event: FileEvent,
        update_sender: mpsc::UnboundedSender<ProcessingUpdate>,
    ) -> Result<(), String> {
        let processing_result = Self::extraction_processing_with_deps(
            &update_sender,
            &embedder,
            &splitter,
            &file_event,
        )
        .await;

        let final_status = match processing_result {
            Ok(_) => ProcessingStatus::Completed,
            Err(e) => ProcessingStatus::Failed(e.to_string()),
        };

        let final_update = ProcessingUpdate {
            file_id,
            status: final_status,
            progress: Some(1.0),
            message: Some("Processing completed".to_string()),
            timestamp: std::time::SystemTime::now(),
        };

        update_sender
            .send(final_update)
            .map_err(|e| format!("Failed to send final update: {}", e))?;

        println!("Finished processing file: {:?}", file_event.file_path);
        Ok(())
    }

    async fn extraction_processing_with_deps(
        update_sender: &mpsc::UnboundedSender<ProcessingUpdate>,
        embedder: &InferenceClient,
        splitter: &RTSplitter,
        file_event: &FileEvent,
    ) -> Result<(), String> {
        match file_event.file_type.to_lowercase().as_str() {
            "text/plain" => Self::process_html_file(update_sender, embedder, splitter, file_event).await,
            // TODO: Implement other file types
            "pdf" => Self::process_document_file(file_event).await,
            "transcript" => Self::process_video_file(file_event).await,
            _ => Self::process_unknown_file_type(file_event).await,
        }
    }

    async fn process_html_file(
        update_sender: &mpsc::UnboundedSender<ProcessingUpdate>,
        embedder: &InferenceClient,
        splitter: &RTSplitter,
        file_event: &FileEvent,
    ) -> Result<(), String> {
        let content = fs::read_to_string(file_event.file_path.clone())
            .expect("Failed to read file from file path");

        let chunks = splitter.split_text(content.as_str(), 200);

        let embeddings_response = embedder
            .get_embeddings(&chunks)
            .await
            .expect("Failed to grab embedding for content chunk.");

        let mut conn = get_database_connection()
            .map_err(|e| AppError::DatabaseError(format!("Could not connect to database: {}", e)))
            .unwrap();

        for (index, chunk) in chunks.clone().iter().enumerate() {
            let new_content_chunk = models::NewContentChunk {
                file_id: Some(file_event.id),
                chunk_text: chunk.clone(),
                chunk_index: index as i32,
                token_count: Some(chunk.split_whitespace().count() as i32),
                page_number: None,
                section_path: None,
            };

            let content_chunk = models::ContentChunk::create(&mut conn, new_content_chunk)
                .map_err(|e| {
                    AppError::DatabaseError(format!("Content chunk could not be created: {}", e))
                })
                .unwrap();

            let new_embeddings_for_chunk = models::NewEmbedding {
                embedding: Some(embeddings_response.embeddings[index].clone()),
                content_chunk_id: content_chunk.id,
                model_name: "normal embedding model".to_string(),
                model_version: Some("v1".to_string()),
                generation_parameters: Some(serde_json::json!({
                    "chunk_size": 200,
                    "chunk_overlap": 50,
                })),
            };

            models::Embedding::create(&mut conn, new_embeddings_for_chunk)
                .map_err(|e| {
                    AppError::DatabaseError(format!("Embedding could not be created: {}", e))
                })
                .unwrap();

            let update = ProcessingUpdate {
                file_id: file_event.id,
                status: ProcessingStatus::Processing,
                progress: Some((index + 1) as f32 / chunks.len() as f32),
                message: Some(format!("Processed chunk {}/{}", index + 1, chunks.len())),
                timestamp: std::time::SystemTime::now(),
            };
            let _ = update_sender.send(update);
        }

        Ok(())
    }

    async fn process_document_file(file_event: &FileEvent) -> Result<(), String> {
        println!("Processing document file: {:?}", file_event.file_path);
        // TODO: Actual document processing logic
        Ok(())
    }

    async fn process_video_file(file_event: &FileEvent) -> Result<(), String> {
        println!("Processing video file: {:?}", file_event.file_path);
        // TODO: Actual video processing logic
        Ok(())
    }

    async fn process_unknown_file_type(file_event: &FileEvent) -> Result<(), String> {
        let msg = format!(
            "Unsupported file type '{}' for file: {:?}",
            file_event.file_type, file_event.file_path
        );
        Err(msg)
    }
}
