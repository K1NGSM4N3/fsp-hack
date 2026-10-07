//! Sentence embeddings computed on our own hardware: candidate and company texts never leave the
//! infrastructure (152-ФЗ). [`wire`] is the HTTP contract and has no heavy dependencies, so the API
//! links it alone; `model` runs the encoder on the CPU with candle, `server` wraps it in a small
//! axum service.

pub mod wire;

#[cfg(feature = "model")]
pub mod batch;
#[cfg(feature = "model")]
pub mod model;
#[cfg(feature = "model")]
pub mod pooling;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "model")]
pub mod unigram;

#[cfg(feature = "model")]
pub use model::{Embedder, ModelError, ModelSpec, Pooling};
pub use wire::{EmbedRequest, EmbedResponse, TextKind};
