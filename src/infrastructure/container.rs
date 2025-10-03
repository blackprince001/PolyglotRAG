use std::{path::PathBuf, sync::Arc};

use crate::{
    application::{
        ports::{DocumentExtractor, EmbeddingProvider, FileStorage, JobQueue},
        services::{DocumentProcessorService, EmbeddingService, SearchService},
        use_cases::{
            CancelJobUseCase, GetFileUseCase, GetJobStatusUseCase,
            ListFilesUseCase, ProcessDocumentUseCase, ProcessUrlDirectUseCase,
            ProcessYoutubeDirectUseCase, QueueProcessingJobUseCase, SearchContentUseCase,
            UploadFileUseCase, UploadWithProcessingUseCase,
        },
    },
    domain::repositories::{ChunkRepository, EmbeddingRepository, FileRepository, JobRepository},
    infrastructure::{
        database::{
            create_connection_pool, get_database_connection,
            repositories::{
                PostgresChunkRepository, PostgresEmbeddingRepository, PostgresFileRepository,
                PostgresJobRepository,
            },
            run_migrations,
        },
        external_services::{
            InferenceEmbeddingProvider, document_extractors::CompositeDocumentExtractor,
        },
        file_system::LocalFileStorage,
        messaging::{BackgroundProcessor, MpscJobQueue},
    },
    presentation::http::handlers::{
        ChunkHandler, ContentHandler, EmbeddingHandler, FileHandler, JobHandler, SearchHandler,
        SseHandler,
    },
};

pub struct AppContainer {
    // Repositories
    pub file_repository: Arc<dyn FileRepository>,
    pub chunk_repository: Arc<dyn ChunkRepository>,
    pub embedding_repository: Arc<dyn EmbeddingRepository>,
    pub job_repository: Arc<dyn JobRepository>,

    // External Services
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    pub file_storage: Arc<dyn FileStorage>,
    pub document_extractor: Arc<dyn DocumentExtractor>,

    // Job Queue and Background Processing
    pub job_queue: Arc<dyn JobQueue>,
    pub background_processor: Arc<BackgroundProcessor>,

    // Application Services
    pub document_processor: Arc<DocumentProcessorService>,
    pub embedding_service: Arc<EmbeddingService>,
    pub search_service: Arc<SearchService>,

    // Use Cases
    pub upload_file_use_case: Arc<UploadFileUseCase>,
    pub upload_with_processing_use_case: Arc<UploadWithProcessingUseCase>,
    pub list_files_use_case: Arc<ListFilesUseCase>,
    pub process_document_use_case: Arc<ProcessDocumentUseCase>,
    pub process_url_direct_use_case: Arc<ProcessUrlDirectUseCase>,
    pub process_youtube_direct_use_case: Arc<ProcessYoutubeDirectUseCase>,
    pub search_content_use_case: Arc<SearchContentUseCase>,
    pub queue_job_use_case: Arc<QueueProcessingJobUseCase>,
    pub get_job_status_use_case: Arc<GetJobStatusUseCase>,
    pub cancel_job_use_case: Arc<CancelJobUseCase>,

    // HTTP Handlers
    pub file_handler: Arc<FileHandler>,
    pub content_handler: Arc<ContentHandler>,
    pub search_handler: Arc<SearchHandler>,
    pub job_handler: Arc<JobHandler>,
    pub sse_handler: Arc<SseHandler>,
    pub chunk_handler: Arc<ChunkHandler>,
    pub embedding_handler: Arc<EmbeddingHandler>,
}

impl AppContainer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create database connection pool
        let db_pool = create_connection_pool()?;
        let mut conn = get_database_connection()
            .map_err(|e| format!("Failed to create database connection: {}", e))?;
        let _ = run_migrations(&mut conn)
            .map_err(|e| format!("Failed to run Database migrations: {}", e));

        // Create repositories
        let file_repository: Arc<dyn FileRepository> =
            Arc::new(PostgresFileRepository::new(db_pool.clone()));
        let chunk_repository: Arc<dyn ChunkRepository> =
            Arc::new(PostgresChunkRepository::new(db_pool.clone()));
        let embedding_repository: Arc<dyn EmbeddingRepository> =
            Arc::new(PostgresEmbeddingRepository::new(db_pool.clone()));
        let job_repository: Arc<dyn JobRepository> = Arc::new(PostgresJobRepository::new(db_pool));

        // Create external services
        let embedding_provider: Arc<dyn EmbeddingProvider> =
            Arc::new(InferenceEmbeddingProvider::from_env()?);

        let upload_dir =
            PathBuf::from(std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()));
        let file_storage: Arc<dyn FileStorage> = Arc::new(LocalFileStorage::new(upload_dir));

        // Create document extractor
        let document_extractor: Arc<dyn DocumentExtractor> = Arc::new(
            CompositeDocumentExtractor::new()
                .map_err(|e| format!("Failed to create document extractor: {}", e))?,
        );

        // Create application services
        let embedding_service = Arc::new(EmbeddingService::new(embedding_provider.clone()));
        let search_service = Arc::new(SearchService::new(
            embedding_provider.clone(),
            embedding_repository.clone(),
            chunk_repository.clone(),
        ));

        // Create document processor service
        let document_processor = Arc::new(DocumentProcessorService::new(
            document_extractor.clone(),
            embedding_provider.clone(),
            chunk_repository.clone(),
            embedding_repository.clone(),
            file_repository.clone(),
        ));

        // Create use cases
        let upload_file_use_case = Arc::new(UploadFileUseCase::new(
            file_repository.clone(),
            file_storage.clone(),
        ));

        let list_files_use_case = Arc::new(ListFilesUseCase::new(file_repository.clone()));

        let process_document_use_case = Arc::new(ProcessDocumentUseCase::new(
            file_repository.clone(),
            document_processor.clone(),
        ));

        let search_content_use_case = Arc::new(SearchContentUseCase::new(search_service.clone()));

        let get_file_use_case = Arc::new(GetFileUseCase::new(file_repository.clone()));

        // Create job queue and background processor
        let (job_queue, job_receiver) = MpscJobQueue::create_pair();
        let job_queue: Arc<dyn JobQueue> = Arc::new(job_queue);
        let job_receiver = Arc::new(job_receiver);

        let background_processor = Arc::new(
            BackgroundProcessor::new(
                job_receiver,
                job_repository.clone(),
                file_repository.clone(),
                document_processor.clone(),
                document_extractor.clone(),
                embedding_provider.clone(),
                file_storage.clone(),
                chunk_repository.clone(),
                embedding_repository.clone(),
            )
            .with_worker_count(3),
        );

        // Create async use cases
        let queue_job_use_case = Arc::new(QueueProcessingJobUseCase::new(
            job_repository.clone(),
            job_queue.clone(),
            file_repository.clone(),
        ));

        let upload_with_processing_use_case = Arc::new(UploadWithProcessingUseCase::new(
            upload_file_use_case.clone(),
            queue_job_use_case.clone(),
            file_repository.clone(),
        ));

        let get_job_status_use_case = Arc::new(GetJobStatusUseCase::new(job_repository.clone()));

        let cancel_job_use_case = Arc::new(CancelJobUseCase::new(
            job_repository.clone(),
            job_queue.clone(),
        ));

        let process_url_direct_use_case = Arc::new(ProcessUrlDirectUseCase::new(
            file_repository.clone(),
            queue_job_use_case.clone(),
        ));

        let process_youtube_direct_use_case = Arc::new(ProcessYoutubeDirectUseCase::new(
            file_repository.clone(),
            queue_job_use_case.clone(),
        ));

        // Create HTTP handlers
        let file_handler = Arc::new(FileHandler::new(
            upload_file_use_case.clone(),
            upload_with_processing_use_case.clone(),
            list_files_use_case.clone(),
            process_document_use_case.clone(),
            get_file_use_case.clone(),
            file_repository.clone(),
        ));

        let search_handler = Arc::new(SearchHandler::new(search_content_use_case.clone()));

        let job_handler = Arc::new(JobHandler::new(
            queue_job_use_case.clone(),
            get_job_status_use_case.clone(),
            cancel_job_use_case.clone(),
        ));

        let sse_handler = Arc::new(SseHandler::new(get_job_status_use_case.clone()));

        let content_handler = Arc::new(ContentHandler::new(
            process_url_direct_use_case.clone(),
            process_youtube_direct_use_case.clone(),
        ));

        let chunk_handler = Arc::new(ChunkHandler::new(chunk_repository.clone()));
        let embedding_handler = Arc::new(EmbeddingHandler::new(embedding_repository.clone()));

        Ok(Self {
            file_repository,
            chunk_repository,
            embedding_repository,
            job_repository,
            embedding_provider,
            file_storage,
            document_extractor,
            job_queue,
            background_processor,
            document_processor,
            embedding_service,
            search_service,
            upload_file_use_case,
            upload_with_processing_use_case,
            list_files_use_case,
            process_document_use_case,
            process_url_direct_use_case,
            process_youtube_direct_use_case,
            search_content_use_case,
            queue_job_use_case,
            get_job_status_use_case,
            cancel_job_use_case,
            file_handler,
            content_handler,
            search_handler,
            job_handler,
            sse_handler,
            chunk_handler,
            embedding_handler,
        })
    }
}
