//! `POST /embed` request and response, shared by the service and its clients.

use serde::{Deserialize, Serialize};

/// Texts per request; a client with more splits them.
pub const MAX_BATCH: usize = 64;
/// Characters per text. Longer texts are refused rather than cut: the caller decides what to drop.
pub const MAX_TEXT_CHARS: usize = 8_000;

/// Which side of a search the text is on. Asymmetric models (multilingual-e5) are trained with a
/// prefix per side; symmetric comparisons (vacancy against vacancy) use `Query` on both.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextKind {
    Query,
    #[default]
    Passage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub texts: Vec<String>,
    #[serde(default)]
    pub kind: TextKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    /// `name@revision` of the weights; stored next to every vector so a model change re-embeds.
    pub model: String,
    pub dim: usize,
    /// L2-normalized, one per input text, in input order.
    pub vectors: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RequestError {
    #[error("texts is empty")]
    Empty,
    #[error("at most {MAX_BATCH} texts per request, got {0}")]
    TooMany(usize),
    #[error("text {index} has {chars} characters, at most {MAX_TEXT_CHARS}")]
    TooLong { index: usize, chars: usize },
}

impl EmbedRequest {
    pub fn validate(&self) -> Result<(), RequestError> {
        if self.texts.is_empty() {
            return Err(RequestError::Empty);
        }
        if self.texts.len() > MAX_BATCH {
            return Err(RequestError::TooMany(self.texts.len()));
        }
        for (index, t) in self.texts.iter().enumerate() {
            let chars = t.chars().count();
            if chars > MAX_TEXT_CHARS {
                return Err(RequestError::TooLong { index, chars });
            }
        }
        Ok(())
    }
}

/// Cuts a text to [`MAX_TEXT_CHARS`] on a character boundary, for clients that would rather embed
/// the beginning of a long text than nothing.
pub fn clip(text: &str) -> &str {
    match text.char_indices().nth(MAX_TEXT_CHARS) {
        Some((at, _)) => &text[..at],
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(texts: &[&str]) -> EmbedRequest {
        EmbedRequest {
            texts: texts.iter().map(|t| (*t).to_owned()).collect(),
            kind: TextKind::Passage,
        }
    }

    #[test]
    fn requests_are_bounded() {
        assert_eq!(req(&[]).validate(), Err(RequestError::Empty));
        assert_eq!(req(&["a"; 64]).validate(), Ok(()));
        assert_eq!(req(&["a"; 65]).validate(), Err(RequestError::TooMany(65)));
        let long = "я".repeat(MAX_TEXT_CHARS + 1);
        assert_eq!(
            req(&["ok", &long]).validate(),
            Err(RequestError::TooLong {
                index: 1,
                chars: MAX_TEXT_CHARS + 1
            })
        );
        assert_eq!(req(&[&"я".repeat(MAX_TEXT_CHARS)]).validate(), Ok(()));
    }

    #[test]
    fn clip_keeps_whole_characters() {
        let long = "ё".repeat(MAX_TEXT_CHARS + 10);
        let clipped = clip(&long);
        assert_eq!(clipped.chars().count(), MAX_TEXT_CHARS);
        assert_eq!(clip("короткий"), "короткий");
    }

    #[test]
    fn kind_defaults_to_passage_and_is_snake_case() {
        let r: EmbedRequest = serde_json::from_str(r#"{"texts": ["x"]}"#).unwrap();
        assert_eq!(r.kind, TextKind::Passage);
        let r: EmbedRequest = serde_json::from_str(r#"{"texts": ["x"], "kind": "query"}"#).unwrap();
        assert_eq!(r.kind, TextKind::Query);
    }
}
