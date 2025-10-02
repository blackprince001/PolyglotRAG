use std::sync::Arc;

use crate::application::ports::document_extractor::DocumentExtractor;
use crate::application::ports::document_extractor::ExtractionOptions;
use crate::application::ports::embedding_provider::BatchEmbeddingRequest;
use crate::application::ports::embedding_provider::EmbeddingProvider;
use crate::application::ports::file_storage::FileStorage;
use crate::application::services::DocumentProcessorService;
use crate::domain::entities::processing_job::{JobResult, JobType, ProcessingJob};
use crate::domain::repositories::{
    ChunkRepository, EmbeddingRepository, FileRepository, JobRepository,
};
use crate::infrastructure::external_services::semantic_chunking::{
    RTSplitter, RecursiveTextSplitter,
};
use crate::infrastructure::messaging::MpscJobQueueReceiver;

pub struct BackgroundProcessor {
    job_receiver: Arc<MpscJobQueueReceiver>,
    job_repository: Arc<dyn JobRepository>,
    file_repository: Arc<dyn FileRepository>,
    document_processor: Arc<DocumentProcessorService>,
    document_extractor: Arc<dyn DocumentExtractor>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    file_storage: Arc<dyn FileStorage>,
    chunk_repository: Arc<dyn ChunkRepository>,
    embedding_repository: Arc<dyn EmbeddingRepository>,
    text_splitter: RTSplitter,
    worker_count: usize,
}

impl BackgroundProcessor {
    pub fn new(
        job_receiver: Arc<MpscJobQueueReceiver>,
        job_repository: Arc<dyn JobRepository>,
        file_repository: Arc<dyn FileRepository>,
        document_processor: Arc<DocumentProcessorService>,
        document_extractor: Arc<dyn DocumentExtractor>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        file_storage: Arc<dyn FileStorage>,
        chunk_repository: Arc<dyn ChunkRepository>,
        embedding_repository: Arc<dyn EmbeddingRepository>,
    ) -> Self {
        Self {
            job_receiver,
            job_repository,
            file_repository,
            document_processor,
            document_extractor,
            embedding_provider,
            file_storage,
            chunk_repository,
            embedding_repository,
            text_splitter: RTSplitter::default(),
            worker_count: 3, // Default worker count
        }
    }

    pub fn with_worker_count(mut self, count: usize) -> Self {
        self.worker_count = count.max(1); // At least 1 worker
        self
    }

    pub async fn start(&self) {
        println!(
            "Starting background processor with {} workers",
            self.worker_count
        );

        // Spawn multiple worker tasks
        let mut handles = Vec::new();

        for worker_id in 0..self.worker_count {
            let processor = self.clone_for_worker();
            let handle = tokio::spawn(async move {
                processor.worker_loop(worker_id).await;
            });
            handles.push(handle);
        }

        // Wait for all workers to complete (they shouldn't unless there's an error)
        for (i, handle) in handles.into_iter().enumerate() {
            if let Err(e) = handle.await {
                eprintln!("Worker {} panicked: {}", i, e);
            }
        }

        println!("Background processor stopped");
    }

    async fn worker_loop(&self, worker_id: usize) {
        println!("Worker {} started", worker_id);

        loop {
            match self.job_receiver.recv().await {
                Some(v) => {
                    println!("Worker {} processing job: {}", worker_id, v.id());
                    self.process_job(v).await;
                }
                None => {
                    println!("Worker {} received None, closing channel", worker_id);
                    break;
                }
            }
        }

        println!("Worker {} stopped", worker_id);
    }

    async fn process_job(&self, mut job: ProcessingJob) {
        let job_id = job.id();
        let start_time = std::time::Instant::now();

        // Update job status to processing
        if let Err(e) = job.start_processing() {
            eprintln!("Failed to start job {}: {}", job_id, e);
            return;
        }

        if let Err(e) = self.job_repository.update(&job).await {
            eprintln!("Failed to update job {} status: {}", job_id, e);
            return;
        }

        // Process based on job type
        let result = match job.job_type().clone() {
            JobType::FileProcessing => self.process_file_job(&mut job).await,
            JobType::UrlExtraction { url } => self.process_url_extraction_job(&mut job, &url).await,
            JobType::YoutubeExtraction { url } => {
                self.process_youtube_extraction_job(&mut job, &url).await
            }
        };

        // Update job with result
        match result {
            Ok(job_result) => {
                if let Err(e) = job.complete_processing(job_result) {
                    eprintln!("Failed to complete job {}: {}", job_id, e);
                } else {
                    let duration = start_time.elapsed();
                    println!("Job {} completed in {:.2}s", job_id, duration.as_secs_f64());
                }
            }
            Err(error) => {
                if let Err(e) = job.fail_processing(error.clone()) {
                    eprintln!("Failed to fail job {}: {}", job_id, e);
                } else {
                    println!("Job {} failed: {}", job_id, error);
                }
            }
        }

        // Save final job state
        if let Err(e) = self.job_repository.update(&job).await {
            eprintln!("Failed to save final job {} state: {}", job_id, e);
        }
    }

    async fn process_file_job(&self, job: &mut ProcessingJob) -> Result<JobResult, String> {
        // Update progress
        let _ = job.update_progress(0.1, Some("Loading file...".to_string()));
        let _ = self.job_repository.update(job).await;

        // Add a small delay to ensure file save transaction is visible to this connection
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Get the file - this should exist if upload was successful
        let file = self
            .file_repository
            .find_by_id(job.file_id())
            .await
            .map_err(|e| format!("Failed to find file: {}", e))?
            .ok_or_else(|| format!("File not found in database: {}", job.file_id()))?;

        // Update progress
        let _ = job.update_progress(0.2, Some("Processing document...".to_string()));
        let _ = self.job_repository.update(job).await;

        // Process the document
        let (chunks_created, embeddings_created) = self
            .document_processor
            .process_file(&file, ExtractionOptions::default())
            .await
            .map_err(|e| format!("Document processing failed: {}", e))?;

        Ok(JobResult {
            chunks_created,
            embeddings_created,
            processing_time_ms: 0,    // Will be calculated by the job
            extracted_text_length: 0, // Could be calculated if needed
        })
    }

    async fn process_url_extraction_job(
        &self,
        job: &mut ProcessingJob,
        url: &str,
    ) -> Result<JobResult, String> {
        // Update progress
        let _ = job.update_progress(0.1, Some("Extracting content from URL...".to_string()));
        let _ = self.job_repository.update(job).await;

        // Extract content from URL
        let extracted_content = self
            .document_extractor
            .extract_text_from_bytes(
                url.as_bytes(),
                "text/html",
                ExtractionOptions {
                    extract_metadata: true,
                    max_pages: None,
                },
            )
            .await
            .map_err(|e| format!("URL extraction failed: {}", e))?;

        // Update progress
        let _ = job.update_progress(0.3, Some("Creating chunks...".to_string()));
        let _ = self.job_repository.update(job).await;

        // Create chunks from extracted text
        let chunks = self.create_chunks_from_text(job.file_id(), &extracted_content.text)?;

        // Save chunks
        self.chunk_repository
            .save_batch(&chunks)
            .await
            .map_err(|e| format!("Failed to save chunks: {}", e))?;

        // Update progress
        let _ = job.update_progress(0.6, Some("Generating embeddings...".to_string()));
        let _ = self.job_repository.update(job).await;

        // Generate embeddings
        let embeddings = self.generate_embeddings_for_chunks(&chunks).await?;

        // Save embeddings
        self.embedding_repository
            .save_batch(&embeddings)
            .await
            .map_err(|e| format!("Failed to save embeddings: {}", e))?;

        Ok(JobResult {
            chunks_created: chunks.len() as i32,
            embeddings_created: embeddings.len() as i32,
            processing_time_ms: 0,
            extracted_text_length: extracted_content.text.len(),
        })
    }

    async fn process_youtube_extraction_job(
        &self,
        job: &mut ProcessingJob,
        url: &str,
    ) -> Result<JobResult, String> {
        // Update progress
        let _ = job.update_progress(0.1, Some("Fetching YouTube transcript...".to_string()));
        let _ = self.job_repository.update(job).await;

        // Extract YouTube transcript
        let extracted_content = self
            .document_extractor
            .extract_text_from_bytes(
                url.as_bytes(),
                "text/youtube-url",
                ExtractionOptions {
                    extract_metadata: true,
                    max_pages: None,
                },
            )
            .await
            .map_err(|e| format!("YouTube extraction failed: {}", e))?;

        // Update progress
        let _ = job.update_progress(0.3, Some("Creating chunks...".to_string()));
        let _ = self.job_repository.update(job).await;

        // Create chunks from transcript
        let chunks = self.create_chunks_from_text(job.file_id(), &extracted_content.text)?;

        // Save chunks
        self.chunk_repository
            .save_batch(&chunks)
            .await
            .map_err(|e| format!("Failed to save chunks: {}", e))?;

        // Update progress
        let _ = job.update_progress(0.6, Some("Generating embeddings...".to_string()));
        let _ = self.job_repository.update(job).await;

        // Generate embeddings
        let embeddings = self.generate_embeddings_for_chunks(&chunks).await?;

        // Save embeddings
        self.embedding_repository
            .save_batch(&embeddings)
            .await
            .map_err(|e| format!("Failed to save embeddings: {}", e))?;

        Ok(JobResult {
            chunks_created: chunks.len() as i32,
            embeddings_created: embeddings.len() as i32,
            processing_time_ms: 0,
            extracted_text_length: extracted_content.text.len(),
        })
    }

    fn create_chunks_from_text(
        &self,
        file_id: uuid::Uuid,
        text: &str,
    ) -> Result<Vec<crate::domain::entities::ContentChunk>, String> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Use RTSplitter with a reasonable chunk size (characters, not words)
        let max_chunk_size = 2000; // characters - good balance for embeddings
        let chunk_texts = self.text_splitter.split_text(text, max_chunk_size);

        let mut chunks = Vec::new();
        for (index, chunk_text) in chunk_texts.into_iter().enumerate() {
            if chunk_text.trim().len() < 10 {
                continue; // Skip very small chunks
            }

            let word_count = chunk_text.split_whitespace().count() as i32;

            let chunk = crate::domain::entities::ContentChunk::new(
                file_id,
                chunk_text,
                index as i32,
                Some(word_count),
                None, // page_number - not applicable for text extraction
                None, // section_path - could be enhanced later
            );

            chunks.push(chunk);
        }

        Ok(chunks)
    }

    async fn generate_embeddings_for_chunks(
        &self,
        chunks: &[crate::domain::entities::ContentChunk],
    ) -> Result<Vec<crate::domain::entities::Embedding>, String> {
        let mut embeddings = Vec::new();
        let (model_name, model_version) = self.embedding_provider.model_info();

        const BATCH_SIZE: usize = 10;

        for chunk_batch in chunks.chunks(BATCH_SIZE) {
            let texts: Vec<String> = chunk_batch
                .iter()
                .map(|chunk| chunk.chunk_text().to_string())
                .collect();

            let batch_request = BatchEmbeddingRequest {
                texts,
                model_name: Some(model_name.clone()),
                model_version: model_version.clone(),
            };

            let batch_response = self
                .embedding_provider
                .generate_embeddings(batch_request)
                .await
                .map_err(|e| format!("Embedding generation failed: {}", e))?;

            for (chunk, embedding_vector) in
                chunk_batch.iter().zip(batch_response.embeddings.iter())
            {
                let embedding = crate::domain::entities::Embedding::new(
                    chunk.id(),
                    batch_response.model_name.clone(),
                    batch_response.model_version.clone(),
                    None,
                    embedding_vector.clone(),
                );

                embeddings.push(embedding);
            }
        }

        Ok(embeddings)
    }

    fn clone_for_worker(&self) -> Self {
        Self {
            job_receiver: self.job_receiver.clone(),
            job_repository: self.job_repository.clone(),
            file_repository: self.file_repository.clone(),
            document_processor: self.document_processor.clone(),
            document_extractor: self.document_extractor.clone(),
            embedding_provider: self.embedding_provider.clone(),
            file_storage: self.file_storage.clone(),
            chunk_repository: self.chunk_repository.clone(),
            embedding_repository: self.embedding_repository.clone(),
            text_splitter: self.text_splitter.clone(),
            worker_count: self.worker_count,
        }
    }
}
