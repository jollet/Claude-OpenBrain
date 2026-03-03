use axum::{
    Router,
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, delete},
};
use std::sync::Arc;

use crate::db::Database;
use crate::embeddings::EmbeddingClient;
use crate::models::*;

pub struct AppState {
    pub db: Arc<Database>,
    pub embeddings: Arc<EmbeddingClient>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/thoughts", post(create_thought).get(list_thoughts))
        .route("/api/thoughts/{id}", get(get_thought).delete(delete_thought))
        .route("/api/search", post(search_thoughts))
        .route("/api/stats", get(stats_handler))
        .with_state(state)
}

async fn health_handler(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let backend = state.embeddings.active_backend().await;
    Json(HealthResponse {
        status: if state.db.is_healthy() { "ok".into() } else { "degraded".into() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        db_healthy: state.db.is_healthy(),
        embedding_backend: backend,
    })
}

async fn stats_handler(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    state.db.get_stats()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_thought(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateThought>,
) -> Result<(StatusCode, Json<Thought>), StatusCode> {
    if body.content.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Save to DB first to get an ID
    let mut thought = state.db.insert_thought(&body.content, &body.tags)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    // Try to get embeddings in the background / inline
    if let Ok(embed_res) = state.embeddings.embed(&body.content).await {
        if state.db.set_embedding(thought.id, &embed_res.embedding).is_ok() {
            thought.has_embedding = true;
        }
    }
    
    Ok((StatusCode::CREATED, Json(thought)))
}

#[derive(serde::Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: i64,
}

async fn search_thoughts(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SearchRequest>,
) -> Result<Json<Vec<Thought>>, StatusCode> {
    // Generate embedding for query
    let embed_res = state.embeddings.embed(&body.query)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        
    // Search local Vector DB
    let thoughts = state.db.search(&embed_res.embedding, body.limit)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    Ok(Json(thoughts))
}

async fn list_thoughts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Thought>>, StatusCode> {
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);
    let thoughts = state.db.list_thoughts(limit, offset)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(thoughts))
}

async fn get_thought(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Thought>, StatusCode> {
    state.db.get_thought(id)
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn delete_thought(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> StatusCode {
    match state.db.delete_thought(id) {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
