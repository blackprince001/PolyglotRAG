use tokio::sync::mpsc;
use uuid::Uuid;

use crate::service::scheduler::{FileEvent, FileRegistry, ProcessingStatus, ProcessingUpdate};

pub struct FileProcessor {
    pub file_receiver: mpsc::UnboundedReceiver<FileEvent>,
    pub update_receiver: mpsc::UnboundedReceiver<ProcessingUpdate>,
    pub update_sender: mpsc::UnboundedSender<ProcessingUpdate>,
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

            tokio::spawn(async move {
                if let Err(e) = Self::process_file(file_id, file_event, update_sender).await {
                    eprintln!("Error processing file: {}", e);
                }
            });
        }

        println!("File processor stopped");
    }

    async fn process_file(
        file_id: Uuid,
        file_event: FileEvent,
        update_sender: mpsc::UnboundedSender<ProcessingUpdate>,
    ) -> Result<(), String> {
        let processing_result = Self::your_file_processing_logic(&file_event).await;

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

    async fn your_file_processing_logic(file_event: &FileEvent) -> Result<(), String> {
        match file_event.file_type.to_lowercase().as_str() {
            "html" => Self::process_html_file(file_event).await,
            "pdf" => Self::process_document_file(file_event).await,
            "transcript" => Self::process_video_file(file_event).await,
            _ => Self::process_unknown_file_type(file_event).await,
        }
    }

    async fn process_html_file(file_event: &FileEvent) -> Result<(), String> {
        println!("Processing image file: {:?}", file_event.file_path);
        // TODO: Actual image processing logic
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
