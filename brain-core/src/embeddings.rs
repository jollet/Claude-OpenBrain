use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Serialize)]
struct EmbedRequestPayload {
    text: String,
}

#[derive(Debug, Deserialize)]
struct EmbedResponsePayload {
    embedding: Vec<f32>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct FallbackHealth {
    status: String,
    model_loaded: bool,
}

/// Result of an embedding operation.
#[derive(Debug, Clone, Serialize)]
pub struct EmbedResult {
    pub embedding: Vec<f32>,
    pub model: String,
    pub source: String,
}

/// Embedding client that tries LM Studio first, falls back to the local container.
pub struct EmbeddingClient {
    http: Client,
    primary_url: Option<String>,
    primary_model: String,
    fallback_url: Option<String>,
}

impl EmbeddingClient {
    pub fn new(
        primary_url: Option<String>,
        primary_model: String,
        fallback_url: Option<String>,
    ) -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            primary_url,
            primary_model,
            fallback_url,
        }
    }

    /// Embed text, trying primary (LM Studio) first, then fallback.
    pub async fn embed(&self, text: &str) -> Result<EmbedResult> {
        // Try primary (LM Studio / OpenAI-compatible endpoint)
        if let Some(ref url) = self.primary_url {
            match self.embed_via_lm_studio(url, text).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!("Primary embedding failed: {}, trying fallback", e);
                }
            }
        }

        // Try fallback (local sentence-transformers container)
        if let Some(ref url) = self.fallback_url {
            return self.embed_via_fallback(url, text).await;
        }

        anyhow::bail!("No embedding backend available")
    }

    /// Check which backend is available and return its name.
    pub async fn active_backend(&self) -> String {
        if let Some(ref url) = self.primary_url {
            if self.check_lm_studio(url).await {
                return format!("lm-studio ({})", self.primary_model);
            }
        }
        if let Some(ref url) = self.fallback_url {
            if self.check_fallback(url).await {
                return "fallback (all-MiniLM-L6-v2)".to_string();
            }
        }
        "none".to_string()
    }

    async fn embed_via_lm_studio(&self, base_url: &str, text: &str) -> Result<EmbedResult> {
        let payload = serde_json::json!({
            "model": self.primary_model,
            "input": [text]
        });
        let resp = self.http
            .post(format!("{}/v1/embeddings", base_url))
            .json(&payload)
            .send()
            .await?;
        let body: serde_json::Value = resp.json().await?;
        let embedding: Vec<f32> = body["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid LM Studio response"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();
        Ok(EmbedResult {
            embedding,
            model: self.primary_model.clone(),
            source: "lm-studio".to_string(),
        })
    }

    async fn embed_via_fallback(&self, base_url: &str, text: &str) -> Result<EmbedResult> {
        let resp = self.http
            .post(format!("{}/embed", base_url))
            .json(&EmbedRequestPayload { text: text.to_string() })
            .send()
            .await?;
        let body: EmbedResponsePayload = resp.json().await?;
        Ok(EmbedResult {
            embedding: body.embedding,
            model: body.model,
            source: "fallback".to_string(),
        })
    }

    async fn check_lm_studio(&self, base_url: &str) -> bool {
        self.http
            .get(format!("{}/v1/models", base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    async fn check_fallback(&self, base_url: &str) -> bool {
        self.http
            .get(format!("{}/health", base_url))
            .send()
            .await
            .and_then(|r| Ok(r.status().is_success()))
            .unwrap_or(false)
    }
}
