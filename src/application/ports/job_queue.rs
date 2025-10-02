use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::ProcessingJob;

#[derive(Debug)]
pub enum JobQueueError {
    QueueFull,
    QueueEmpty,
    SerializationError(String),
    ConnectionError(String),
    InvalidJob(String),
}

impl std::fmt::Display for JobQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobQueueError::QueueFull => write!(f, "Job queue is full"),
            JobQueueError::QueueEmpty => write!(f, "Job queue is empty"),
            JobQueueError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            JobQueueError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            JobQueueError::InvalidJob(msg) => write!(f, "Invalid job: {}", msg),
        }
    }
}

impl std::error::Error for JobQueueError {}

#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Enqueue a job for processing
    async fn enqueue(&self, job: ProcessingJob) -> Result<(), JobQueueError>;
    
    /// Dequeue the next job for processing (blocking)
    async fn dequeue(&self) -> Result<ProcessingJob, JobQueueError>;
    
    /// Try to dequeue a job without blocking
    async fn try_dequeue(&self) -> Result<Option<ProcessingJob>, JobQueueError>;
    
    /// Get the current queue size
    async fn size(&self) -> Result<usize, JobQueueError>;
    
    /// Check if the queue is empty
    async fn is_empty(&self) -> Result<bool, JobQueueError>;
    
    /// Remove a specific job from the queue (for cancellation)
    async fn remove_job(&self, job_id: Uuid) -> Result<bool, JobQueueError>;
    
    /// Get queue health/statistics
    async fn health_check(&self) -> Result<QueueHealth, JobQueueError>;
}

#[derive(Debug, Clone)]
pub struct QueueHealth {
    pub queue_size: usize,
    pub total_enqueued: u64,
    pub total_dequeued: u64,
    pub is_healthy: bool,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}
