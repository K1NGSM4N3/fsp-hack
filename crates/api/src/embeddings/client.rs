//! HTTP client of the embedder service: `POST /embed` in batches of [`MAX_BATCH`], `GET /healthz`
//! for the model id. Texts are cut to [`MAX_TEXT_CHARS`] here, so a long text is embedded by its
//! beginning instead of being refused.

use std::time::Duration;

use embedder::wire::{EmbedRequest, EmbedResponse, MAX_BATCH, TextKind, clip};
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    #[error("embedder request: {0}")]
    Http(#[from] reqwest::Error),
    #[error("embedder answered {status}: {body}")]
    Status { status: u16, body: String },
    #[error("embedder returned {got} vectors for {sent} texts")]
    Count { sent: usize, got: usize },
    #[error("embedder switched models mid-request: {0} then {1}")]
    Model(String, String),
}

pub struct Embedded {
    /// `name@revision`.
    pub model: String,
    pub vectors: Vec<Vec<f32>>,
}

pub struct EmbedderClient {
    http: reqwest::Client,
    url: String,
}

#[derive(Deserialize)]
struct Health {
    model: String,
}

impl EmbedderClient {
    pub fn new(url: &str, timeout: Duration) -> Result<Self, reqwest::Error> {
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(timeout)
                .connect_timeout(Duration::from_secs(2))
                .build()?,
            url: url.trim_end_matches('/').to_owned(),
        })
    }

    async fn checked(response: reqwest::Response) -> Result<reqwest::Response, EmbedError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let body: String = response
            .text()
            .await
            .unwrap_or_default()
            .chars()
            .take(300)
            .collect();
        Err(EmbedError::Status {
            status: status.as_u16(),
            body,
        })
    }

    /// The model the service runs now.
    pub async fn model(&self) -> Result<String, EmbedError> {
        let response = self
            .http
            .get(format!("{}/healthz", self.url))
            .send()
            .await?;
        Ok(Self::checked(response).await?.json::<Health>().await?.model)
    }

    /// One vector per text, in order.
    pub async fn embed(&self, texts: &[String], kind: TextKind) -> Result<Embedded, EmbedError> {
        let mut model: Option<String> = None;
        let mut vectors = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(MAX_BATCH) {
            let request = EmbedRequest {
                texts: chunk.iter().map(|t| clip(t).to_owned()).collect(),
                kind,
            };
            let response = self
                .http
                .post(format!("{}/embed", self.url))
                .json(&request)
                .send()
                .await?;
            let body: EmbedResponse = Self::checked(response).await?.json().await?;
            if body.vectors.len() != chunk.len() {
                return Err(EmbedError::Count {
                    sent: chunk.len(),
                    got: body.vectors.len(),
                });
            }
            match &model {
                Some(m) if *m != body.model => {
                    return Err(EmbedError::Model(m.clone(), body.model));
                }
                Some(_) => {}
                None => model = Some(body.model),
            }
            vectors.extend(body.vectors);
        }
        Ok(Embedded {
            model: model.unwrap_or_default(),
            vectors,
        })
    }
}
