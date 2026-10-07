//! The embedding model for offline runs: loaded in-process from a snapshot directory
//! (`make model`), the same weights and code as the embedder service. Without it the runs fall back
//! to keyword closeness and say so.

use std::path::PathBuf;

use embedder::{Embedder, model};

pub struct Choice {
    /// Model name from `embedder::model::MODELS`, or `none`.
    pub name: String,
    /// Directory holding one snapshot directory per model name.
    pub dir: PathBuf,
}

impl Default for Choice {
    fn default() -> Self {
        Choice {
            name: model::MULTILINGUAL_E5_SMALL.name.to_owned(),
            dir: PathBuf::from("models"),
        }
    }
}

impl Choice {
    /// The loaded model, or `None` with the reason on stderr.
    pub fn load(&self) -> Option<Embedder> {
        if self.name == "none" {
            eprintln!("эмбеддинги выключены (--model none): близость описаний по ключевым словам");
            return None;
        }
        let Some(spec) = model::spec(&self.name) else {
            eprintln!(
                "неизвестная модель {}: близость описаний по ключевым словам",
                self.name
            );
            return None;
        };
        let dir = self.dir.join(&self.name);
        match Embedder::load(&dir, spec) {
            Ok(e) => {
                eprintln!("модель эмбеддингов: {} ({})", e.id(), dir.display());
                Some(e)
            }
            Err(e) => {
                eprintln!(
                    "модель не загружена ({e}) — `make model`; близость описаний по ключевым словам"
                );
                None
            }
        }
    }
}
