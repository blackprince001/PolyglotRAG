use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

use crate::application::ports::job_queue::{JobQueue, JobQueueError, QueueHealth};
use crate::domain::entities::processing_job::ProcessingJob;

pub struct MpscJobQueue {
    sender: mpsc::UnboundedSender<ProcessingJob>,
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<ProcessingJob>>>,
    pending_jobs: Arc<Mutex<HashMap<Uuid, ProcessingJob>>>, // For job removal
    stats: Arc<Mutex<QueueStats>>,
}

#[derive(Debug, Clone)]
struct QueueStats {
    total_enqueued: u64,
    total_dequeued: u64,
    last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

impl MpscJobQueue {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            pending_jobs: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(QueueStats {
                total_enqueued: 0,
                total_dequeued: 0,
                last_activity: None,
            })),
        }
    }

    pub fn create_pair() -> (Self, MpscJobQueueReceiver) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let pending_jobs = Arc::new(Mutex::new(HashMap::new()));
        let stats = Arc::new(Mutex::new(QueueStats {
            total_enqueued: 0,
            total_dequeued: 0,
            last_activity: None,
        }));

        let queue = Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            pending_jobs: pending_jobs.clone(),
            stats: stats.clone(),
        };

        // Create a separate receiver for the background processor
        let (bg_sender, bg_receiver) = mpsc::unbounded_channel();
        let bg_queue = MpscJobQueueReceiver {
            receiver: Arc::new(Mutex::new(bg_receiver)),
            pending_jobs: pending_jobs.clone(),
            stats: stats.clone(),
            sender: bg_sender,
        };

        // Forward messages from main queue to background queue
        let main_receiver = queue.receiver.clone();
        let bg_sender_clone = bg_queue.sender.clone();
        tokio::spawn(async move {
            loop {
                let job = {
                    let mut receiver = main_receiver.lock().await;
                    receiver.recv().await
                };

                match job {
                    Some(job) => {
                        if bg_sender_clone.send(job).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    None => break, // Channel closed
                }
            }
        });

        (queue, bg_queue)
    }
}

#[async_trait]
impl JobQueue for MpscJobQueue {
    async fn enqueue(&self, job: ProcessingJob) -> Result<(), JobQueueError> {
        // Add to pending jobs map
        {
            let mut pending = self.pending_jobs.lock().await;
            pending.insert(job.id(), job.clone());
        }

        // Send to channel
        self.sender
            .send(job)
            .map_err(|_| JobQueueError::ConnectionError("Channel closed".to_string()))?;

        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.total_enqueued += 1;
            stats.last_activity = Some(chrono::Utc::now());
        }

        Ok(())
    }

    async fn dequeue(&self) -> Result<ProcessingJob, JobQueueError> {
        let job = {
            let mut receiver = self.receiver.lock().await;
            receiver.recv().await
        };

        match job {
            Some(job) => {
                // Remove from pending jobs
                {
                    let mut pending = self.pending_jobs.lock().await;
                    pending.remove(&job.id());
                }

                // Update stats
                {
                    let mut stats = self.stats.lock().await;
                    stats.total_dequeued += 1;
                    stats.last_activity = Some(chrono::Utc::now());
                }

                Ok(job)
            }
            None => Err(JobQueueError::ConnectionError("Channel closed".to_string())),
        }
    }

    async fn try_dequeue(&self) -> Result<Option<ProcessingJob>, JobQueueError> {
        let job = {
            let mut receiver = self.receiver.lock().await;
            receiver.try_recv()
        };

        match job {
            Ok(job) => {
                // Remove from pending jobs
                {
                    let mut pending = self.pending_jobs.lock().await;
                    pending.remove(&job.id());
                }

                // Update stats
                {
                    let mut stats = self.stats.lock().await;
                    stats.total_dequeued += 1;
                    stats.last_activity = Some(chrono::Utc::now());
                }

                Ok(Some(job))
            }
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err(JobQueueError::ConnectionError("Channel closed".to_string()))
            }
        }
    }

    async fn size(&self) -> Result<usize, JobQueueError> {
        let pending = self.pending_jobs.lock().await;
        Ok(pending.len())
    }

    async fn is_empty(&self) -> Result<bool, JobQueueError> {
        let pending = self.pending_jobs.lock().await;
        Ok(pending.is_empty())
    }

    async fn remove_job(&self, job_id: Uuid) -> Result<bool, JobQueueError> {
        let mut pending = self.pending_jobs.lock().await;
        Ok(pending.remove(&job_id).is_some())
    }

    async fn health_check(&self) -> Result<QueueHealth, JobQueueError> {
        let pending = self.pending_jobs.lock().await;
        let stats = self.stats.lock().await;

        Ok(QueueHealth {
            queue_size: pending.len(),
            total_enqueued: stats.total_enqueued,
            total_dequeued: stats.total_dequeued,
            is_healthy: true, // MPSC is always healthy if not closed
            last_activity: stats.last_activity,
        })
    }
}

// Separate receiver for background processing
pub struct MpscJobQueueReceiver {
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<ProcessingJob>>>,
    pending_jobs: Arc<Mutex<HashMap<Uuid, ProcessingJob>>>,
    stats: Arc<Mutex<QueueStats>>,
    sender: mpsc::UnboundedSender<ProcessingJob>, // For forwarding
}

impl MpscJobQueueReceiver {
    pub async fn recv(&self) -> Option<ProcessingJob> {
        let mut receiver = self.receiver.lock().await;
        receiver.recv().await
    }

    pub async fn try_recv(&self) -> Result<Option<ProcessingJob>, mpsc::error::TryRecvError> {
        let mut receiver = self.receiver.lock().await;
        match receiver.try_recv() {
            Ok(job) => Ok(Some(job)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl Default for MpscJobQueue {
    fn default() -> Self {
        Self::new()
    }
}
