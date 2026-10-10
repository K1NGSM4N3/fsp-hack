//! Sentence embeddings in the API (docs/architecture.md section 6.4). The embedder service computes them; a
//! background [`worker`] embeds what the queue holds, and requests only read stored vectors, never
//! the model. With `EMBEDDINGS=off` nothing here is used; with the embedder down the stored vectors
//! keep working and new texts fall back to keywords until the queue drains.

pub mod client;
pub mod worker;

pub use client::{EmbedError, Embedded, EmbedderClient};

use std::sync::Arc;

use matching::semantic::{self, Calibration, Query};
use serde_json::Value;
use uuid::Uuid;

use crate::{error::ApiResult, state::AppState};

/// A need's vector ready for the pool: quantized like the profiles, with its model's calibration.
#[derive(Debug, Clone, PartialEq)]
pub struct NeedVector {
    pub model: String,
    pub query: Query,
    pub calibration: Calibration,
}

impl NeedVector {
    /// `None` for a model nobody calibrated or an empty vector: such a need ranks by keywords.
    pub fn new(model: String, embedding: &[f32]) -> Option<Self> {
        let calibration = semantic::calibration(&model)?;
        Some(Self {
            query: Query::new(embedding)?,
            model,
            calibration,
        })
    }
}

/// The stored vector of a need, when embeddings are on (remembered per generation by
/// [`crate::cache::CandidateCache::need_vector`]).
pub async fn need_vector(state: &AppState, need_id: Uuid) -> ApiResult<Option<Arc<NeedVector>>> {
    if state.embedder.is_none() {
        return Ok(None);
    }
    state.candidates.need_vector(&state.pool, need_id).await
}

/// Positions and descriptions of the jobs in `candidates.experience`
/// (`[{company, position, start, end, description}]`); malformed entries are skipped.
pub fn experience_parts(experience: &Value) -> Vec<(String, String)> {
    let field = |e: &Value, key: &str| {
        e.get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    experience
        .as_array()
        .map(|jobs| {
            jobs.iter()
                .map(|e| (field(e, "position"), field(e, "description")))
                .collect()
        })
        .unwrap_or_default()
}

/// The profile text a candidate's vector is computed from (`matching::semantic::candidate_text`).
pub fn profile_text(roles: &[String], about: &str, experience: &Value) -> String {
    let parts = experience_parts(experience);
    semantic::candidate_text(
        roles,
        about,
        parts.iter().map(|(p, d)| (p.as_str(), d.as_str())),
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn profile_text_reads_jobs_and_leaves_out_companies() {
        let experience = json!([
            {"company": "Банк «Северная Гавань»", "position": "Go-разработчик", "from": "2021-03", "description": "Платёжный шлюз на Go."},
            {"company": "Фриланс", "position": "Стажёр"},
            "сломанная запись",
        ]);
        let text = profile_text(&["Бэкенд-разработчик".into()], "Пишу на Go.", &experience);
        assert_eq!(
            text,
            "Бэкенд-разработчик\nПишу на Go.\nGo-разработчик\nПлатёжный шлюз на Go.\nСтажёр"
        );
        assert!(!text.contains("Гавань"));
        assert_eq!(profile_text(&[], "", &json!({})), "");
    }

    #[test]
    fn a_need_vector_needs_a_calibrated_model_and_a_real_vector() {
        let v = [0.6f32, 0.8, 0.0];
        let n = NeedVector::new("multilingual-e5-small@614241f".into(), &v).unwrap();
        assert_eq!(
            Some(n.calibration),
            semantic::calibration("multilingual-e5-small")
        );
        assert!(NeedVector::new("unknown-model@1".into(), &v).is_none());
        assert!(NeedVector::new("multilingual-e5-small@614241f".into(), &[0.0, 0.0]).is_none());
    }
}
