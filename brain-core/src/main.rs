mod api;
mod db;
mod embeddings;
mod models;

use anyhow::Result;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "brain_core=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = std::env::var("BRAIN_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("BRAIN_PORT").unwrap_or_else(|_| "3000".to_string());
    let db_path = std::env::var("BRAIN_DB_PATH").unwrap_or_else(|_| "/data/brain.db".to_string());

    // Embedding config
    let embedding_url = std::env::var("EMBEDDING_URL").ok();
    let embedding_model = std::env::var("EMBEDDING_MODEL")
        .unwrap_or_else(|_| "nomic-embed-text-v1.5".to_string());
    let fallback_url = std::env::var("EMBEDDING_FALLBACK_URL").ok();

    info!("Starting brain-core v{}", env!("CARGO_PKG_VERSION"));

    let database = db::Database::open(&db_path)?;
    info!("Database ready at {}", db_path);

    let embed_client = embeddings::EmbeddingClient::new(
        embedding_url,
        embedding_model,
        fallback_url,
    );
    let backend = embed_client.active_backend().await;
    info!("Embedding backend: {}", backend);

    let state = Arc::new(api::AppState {
        db: Arc::new(database),
        embeddings: Arc::new(embed_client),
    });

    let app = api::router(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    info!("Listening on {}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
