use super::serializers::AppState;
use crate::{
    server::routes::file_router::{list_files, process_file, upload_file},
    server::routes::html_router::parse_html_content,
    service::scheduler::Scheduler,
};

use axum::{
    Router,
    routing::{get, post},
    serve,
};

use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::fs::{self};

use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

pub async fn run(scheduler: Scheduler) {
    let upload_dir =
        PathBuf::from(std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()));

    fs::create_dir_all(&upload_dir)
        .await
        .expect("Failed to create upload directory");

    let state = Arc::new(AppState {
        upload_dir,
        scheduler,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Document Processing API" }))
        .route("/upload", post(upload_file))
        .route("/files", get(list_files))
        .route("/process/{id}", post(process_file))
        .route("/upload_html", post(parse_html_content))
        .with_state(state)
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(
            250 * 1024, // 25mb cap
        ))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    serve(listener, app).await.unwrap();
}
