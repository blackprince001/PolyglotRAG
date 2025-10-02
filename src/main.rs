use std::env;
mod application;
mod domain;
mod infrastructure;
mod presentation;

use infrastructure::container::AppContainer;
use presentation::http::server::HttpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv::dotenv().ok();

    let container = AppContainer::new().await?;

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let server = HttpServer::new(
        container.file_handler,
        container.search_handler,
        container.job_handler,
        container.sse_handler,
        container.background_processor,
        Some(port),
    );

    server.run().await?;

    Ok(())
}
