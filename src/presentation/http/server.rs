use axum::Router;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::infrastructure::messaging::BackgroundProcessor;
use crate::presentation::http::{
    handlers::{
        ChunkHandler, ContentHandler, EmbeddingHandler, FileHandler, JobHandler, SearchHandler,
        SseHandler,
    },
    routes::{
        chunk_routes, content_processing_routes, embedding_routes, file_routes, health_routes,
        job_routes, search_routes,
    },
};

pub struct HttpServer {
    file_handler: Arc<FileHandler>,
    content_handler: Arc<ContentHandler>,
    search_handler: Arc<SearchHandler>,
    job_handler: Arc<JobHandler>,
    sse_handler: Arc<SseHandler>,
    chunk_handler: Arc<ChunkHandler>,
    embedding_handler: Arc<EmbeddingHandler>,
    background_processor: Arc<BackgroundProcessor>,
    port: u16,
}

impl HttpServer {
    pub fn new(
        file_handler: Arc<FileHandler>,
        content_handler: Arc<ContentHandler>,
        search_handler: Arc<SearchHandler>,
        job_handler: Arc<JobHandler>,
        sse_handler: Arc<SseHandler>,
        chunk_handler: Arc<ChunkHandler>,
        embedding_handler: Arc<EmbeddingHandler>,
        background_processor: Arc<BackgroundProcessor>,
        port: Option<u16>,
    ) -> Self {
        Self {
            file_handler,
            content_handler,
            search_handler,
            job_handler,
            sse_handler,
            chunk_handler,
            embedding_handler,
            background_processor,
            port: port.unwrap_or(3000),
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // Start background processor
        let background_processor = self.background_processor.clone();
        tokio::spawn(async move {
            background_processor.start().await;
        });

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .merge(health_routes())
            .merge(file_routes(self.file_handler.clone()))
            .merge(content_processing_routes(self.content_handler))
            .merge(search_routes(self.search_handler))
            .merge(job_routes(self.job_handler, self.sse_handler))
            .merge(chunk_routes(self.chunk_handler.clone()))
            .merge(embedding_routes(self.embedding_handler.clone()))
            .layer(cors)
            .layer(RequestBodyLimitLayer::new(250 * 1024 * 1024)) // 250MB cap
            .layer(
                TraceLayer::new_for_http()
                    .on_request(
                        |request: &axum::http::Request<axum::body::Body>, _span: &tracing::Span| {
                            tracing::info!(
                                "Received request: {} {}",
                                request.method(),
                                request.uri()
                            );
                        },
                    )
                    .on_response(
                        |response: &axum::http::Response<axum::body::Body>,
                         latency: std::time::Duration,
                         _span: &tracing::Span| {
                            tracing::info!(
                                "Response: {} (took {} ms)",
                                response.status(),
                                latency.as_millis()
                            );
                        },
                    )
                    .on_failure(
                        |error: ServerErrorsFailureClass,
                         latency: std::time::Duration,
                         _span: &tracing::Span| {
                            tracing::error!(
                                "Request failed: {:?} (took {} ms)",
                                error,
                                latency.as_millis()
                            );
                        },
                    ),
            );

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));

        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
