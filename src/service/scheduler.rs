use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::service::file_processor::FileProcessor;
use crate::service::inference::InferenceClient;
use crate::service::semantic_chunking::RTSplitter;

#[derive(Debug, Clone)]
pub struct FileEvent {
    pub id: Uuid,
    pub file_path: PathBuf,
    pub file_type: String,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub enum ProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ProcessingUpdate {
    pub file_id: Uuid,
    pub status: ProcessingStatus,
    pub progress: Option<f32>, // 0.0 to 1.0
    pub message: Option<String>,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug)]
pub struct FileRegistry {
    pub mappings: RwLock<HashMap<Uuid, FileEvent>>,
}

impl FileRegistry {
    pub fn new() -> Self {
        FileRegistry {
            mappings: RwLock::new(HashMap::new()),
        }
    }
}

pub struct Scheduler {
    file_sender: mpsc::UnboundedSender<FileEvent>,
    update_sender: mpsc::UnboundedSender<ProcessingUpdate>,
}

impl Scheduler {
    pub fn new() -> (Self, FileProcessor) {
        let (file_tx, file_rx) = mpsc::unbounded_channel::<FileEvent>();
        let (update_tx, update_rx) = mpsc::unbounded_channel::<ProcessingUpdate>();

        let inference_client = InferenceClient::from_env().expect("Failed to instantiate embedder");

        let splitter = RTSplitter::default();

        let scheduler = Scheduler {
            file_sender: file_tx,
            update_sender: update_tx.clone(),
        };

        let processor = FileProcessor {
            file_receiver: file_rx,
            update_receiver: update_rx,
            update_sender: update_tx,
            embedder: inference_client,
            splitter,
        };

        (scheduler, processor)
    }

    pub async fn schedule_file(
        &self,
        file_id: Uuid,
        file_path: String,
        file_type: String,
    ) -> Result<Uuid, String> {
        let event = FileEvent {
            id: file_id,
            file_path: file_path.into(),
            file_type,
            timestamp: std::time::SystemTime::now(),
        };

        self.file_sender
            .send(event)
            .map_err(|_| "Failed to schedule file - channel closed".to_string())?;

        let update = ProcessingUpdate {
            file_id,
            status: ProcessingStatus::Pending,
            progress: Some(0.0),
            message: Some("File scheduled for processing".to_string()),
            timestamp: std::time::SystemTime::now(),
        };

        let _ = self.update_sender.send(update);

        Ok(file_id)
    }
}
