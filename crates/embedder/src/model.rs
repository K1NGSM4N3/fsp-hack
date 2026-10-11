//! A BERT-family sentence encoder on the CPU: tokenizer.json, config.json and model.safetensors
//! from a Hugging Face snapshot directory, pooled and normalized as the model card says.
//!
//! The token embedding matrix is most of a multilingual model (250 037 × 384 floats, 82 % of
//! multilingual-e5-small) and a Russian or English text touches a few thousand of its rows. It is
//! therefore not copied into memory: rows are read from the memory-mapped weights file per pass,
//! and the pages the kernel keeps are clean file cache it can drop under memory pressure. Only the
//! encoder layers, position and segment embeddings are loaded.

use std::path::Path;

use candle_core::{DType, Device, Module, Tensor, safetensors::MmapedSafetensors};
use candle_nn::{Embedding, LayerNorm, VarBuilder, embedding, layer_norm};
use candle_transformers::models::bert::{BertEncoder, Config};
use tokenizers::{Encoding, Tokenizer, TruncationParams};

use crate::{
    batch, pooling,
    unigram::{self, CompactTokenizer},
    wire::TextKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pooling {
    /// State of `[CLS]` (rubert-tiny2: the model card and its sentence-transformers config).
    Cls,
    /// Masked mean of the token states (multilingual-e5).
    Mean,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSpec {
    pub name: &'static str,
    /// Hugging Face repository the weights come from.
    pub repo: &'static str,
    pub pooling: Pooling,
    pub query_prefix: &'static str,
    pub passage_prefix: &'static str,
    /// Longer inputs are truncated; both models were trained on at most 512 tokens.
    pub max_tokens: usize,
}

pub const RUBERT_TINY2: ModelSpec = ModelSpec {
    name: "rubert-tiny2",
    repo: "cointegrated/rubert-tiny2",
    pooling: Pooling::Cls,
    query_prefix: "",
    passage_prefix: "",
    max_tokens: 512,
};

pub const MULTILINGUAL_E5_SMALL: ModelSpec = ModelSpec {
    name: "multilingual-e5-small",
    repo: "intfloat/multilingual-e5-small",
    pooling: Pooling::Mean,
    query_prefix: "query: ",
    passage_prefix: "passage: ",
    max_tokens: 512,
};

pub const MODELS: [ModelSpec; 2] = [RUBERT_TINY2, MULTILINGUAL_E5_SMALL];

pub fn spec(name: &str) -> Option<ModelSpec> {
    MODELS.iter().copied().find(|m| m.name == name)
}

impl ModelSpec {
    pub fn prefix(&self, kind: TextKind) -> &'static str {
        match kind {
            TextKind::Query => self.query_prefix,
            TextKind::Passage => self.passage_prefix,
        }
    }
}

/// Texts and padded tokens per forward pass. They bound a pass's activations: its attention scores
/// take heads × `PASS_TOKENS` × longest × 4 bytes, 25 MB for 12 heads at 512 tokens.
const PASS_TEXTS: usize = 32;
const PASS_TOKENS: usize = 1024;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("unknown model {0}; known: rubert-tiny2, multilingual-e5-small")]
    Unknown(String),
    #[error("{path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Config {
        path: String,
        source: serde_json::Error,
    },
    #[error("tokenizer: {0}")]
    Tokenizer(#[source] tokenizers::Error),
    #[error(transparent)]
    Unigram(#[from] unigram::UnigramError),
    #[error("{0}: unexpected shape or type of the token embeddings")]
    Weights(String),
    #[error("inference: {0}")]
    Candle(#[from] candle_core::Error),
}

pub struct Embedder {
    words: WordRows,
    positions: Embedding,
    /// Segment embedding of the single-sentence input (segment 0), added to every token.
    segment: Tensor,
    norm: LayerNorm,
    encoder: BertEncoder,
    tokenizer: Tokens,
    spec: ModelSpec,
    id: String,
    dim: usize,
    pad_id: u32,
}

/// WordPiece vocabularies (rubert-tiny2) go through the library as is; Unigram ones (multilingual-e5)
/// through [`unigram`], which holds the same vocabulary in a twentieth of the memory.
enum Tokens {
    Library(Box<Tokenizer>),
    Compact(Box<CompactTokenizer>),
}

impl Tokens {
    fn load(path: &Path, max_tokens: usize) -> Result<Self, ModelError> {
        let raw = std::fs::read(path).map_err(|source| ModelError::Read {
            path: path.display().to_string(),
            source,
        })?;
        let truncation = Some(TruncationParams {
            max_length: max_tokens,
            ..TruncationParams::default()
        });
        if unigram::is_unigram(&raw) {
            let mut t = unigram::load(&raw)?;
            t.with_padding(None);
            t.with_truncation(truncation)
                .map_err(ModelError::Tokenizer)?;
            return Ok(Tokens::Compact(Box::new(t)));
        }
        let mut t = Tokenizer::from_bytes(&raw).map_err(ModelError::Tokenizer)?;
        t.with_padding(None);
        t.with_truncation(truncation)
            .map_err(ModelError::Tokenizer)?;
        Ok(Tokens::Library(Box::new(t)))
    }

    fn encode(&self, inputs: Vec<String>) -> Result<Vec<Encoding>, ModelError> {
        match self {
            Tokens::Library(t) => t.encode_batch(inputs, true),
            Tokens::Compact(t) => t.encode_batch(inputs, true),
        }
        .map_err(ModelError::Tokenizer)
    }
}

/// Rows of the token embedding matrix, read from the mapped weights file on demand.
struct WordRows {
    file: MmapedSafetensors,
    name: String,
    rows: usize,
    dim: usize,
}

impl WordRows {
    fn gather(&self, ids: &[u32]) -> Result<Vec<f32>, ModelError> {
        let view = self.file.get(&self.name)?;
        let data = view.data();
        let row_bytes = self.dim * 4;
        let mut out = Vec::with_capacity(ids.len() * self.dim);
        for &id in ids {
            let id = id as usize;
            let row = data
                .get(id * row_bytes..(id + 1) * row_bytes)
                .filter(|_| id < self.rows)
                .ok_or_else(|| ModelError::Weights(format!("{} row {id}", self.name)))?;
            out.extend(
                row.chunks_exact(4)
                    .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])),
            );
        }
        Ok(out)
    }
}

/// `(batch, seq)` mask of ones and zeros → additive `(batch, 1, 1, seq)` mask for the attention
/// scores: 0 where a token is kept, the lowest float where it is padding.
fn attention_bias(mask: &Tensor) -> candle_core::Result<Tensor> {
    let mask = mask.to_dtype(DType::F32)?.unsqueeze(1)?.unsqueeze(1)?;
    (mask.ones_like()? - &mask)?.affine(f64::from(f32::MIN), 0.0)
}

impl Embedder {
    /// Loads a snapshot directory. A `REVISION` file next to the weights (written by the image
    /// build) becomes part of [`Embedder::id`].
    pub fn load(dir: &Path, spec: ModelSpec) -> Result<Self, ModelError> {
        let config_path = dir.join("config.json");
        let raw = std::fs::read(&config_path).map_err(|source| ModelError::Read {
            path: config_path.display().to_string(),
            source,
        })?;
        let config: Config = serde_json::from_slice(&raw).map_err(|source| ModelError::Config {
            path: config_path.display().to_string(),
            source,
        })?;
        let tokenizer = Tokens::load(
            &dir.join("tokenizer.json"),
            spec.max_tokens.min(config.max_position_embeddings),
        )?;
        let weights = dir.join("model.safetensors");
        if !weights.is_file() {
            return Err(ModelError::Read {
                path: weights.display().to_string(),
                source: std::io::ErrorKind::NotFound.into(),
            });
        }
        // SAFETY: the weights file is read-only in the image (or a read-only mount) and nothing
        // writes or truncates it while the service runs; the mapping is only ever read.
        let file = unsafe { MmapedSafetensors::new(&weights)? };
        let prefix = match file.get("bert.embeddings.word_embeddings.weight") {
            Ok(_) => "bert.",
            Err(_) => "",
        };
        let name = format!("{prefix}embeddings.word_embeddings.weight");
        let view = file.get(&name)?;
        let dim = config.hidden_size;
        let rows = match view.shape() {
            [rows, d] if *d == dim && view.data().len() == rows * d * 4 => *rows,
            _ => return Err(ModelError::Weights(name)),
        };
        drop(view);
        // SAFETY: as above; this second mapping only lives while the layers are copied out.
        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(&[&weights], DType::F32, &Device::Cpu)? };
        let vb = if prefix.is_empty() { vb } else { vb.pp("bert") };
        let emb = vb.pp("embeddings");
        let positions = embedding(
            config.max_position_embeddings,
            dim,
            emb.pp("position_embeddings"),
        )?;
        let segments = embedding(config.type_vocab_size, dim, emb.pp("token_type_embeddings"))?;
        let segment = segments.embeddings().get(0)?;
        let norm = layer_norm(dim, config.layer_norm_eps, emb.pp("LayerNorm"))?;
        let encoder = BertEncoder::load(vb.pp("encoder"), &config)?;
        let revision = std::fs::read_to_string(dir.join("REVISION"))
            .ok()
            .map(|r| r.trim().chars().take(7).collect::<String>())
            .filter(|r| !r.is_empty());
        Ok(Self {
            words: WordRows {
                file,
                name,
                rows,
                dim,
            },
            positions,
            segment,
            norm,
            encoder,
            tokenizer,
            spec,
            id: match revision {
                Some(r) => format!("{}@{r}", spec.name),
                None => spec.name.to_owned(),
            },
            dim,
            pad_id: u32::try_from(config.pad_token_id).unwrap_or(0),
        })
    }

    /// `name@revision`.
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn spec(&self) -> ModelSpec {
        self.spec
    }

    /// Token count of each text after the prefix and truncation, as the model will see it.
    pub fn token_counts(&self, texts: &[&str], kind: TextKind) -> Result<Vec<usize>, ModelError> {
        Ok(self
            .encode(texts, kind)?
            .iter()
            .map(|e| e.get_ids().len())
            .collect())
    }

    fn encode(&self, texts: &[&str], kind: TextKind) -> Result<Vec<Encoding>, ModelError> {
        let prefix = self.spec.prefix(kind);
        self.tokenizer
            .encode(texts.iter().map(|t| format!("{prefix}{t}")).collect())
    }

    /// Unit vectors in input order. Texts are grouped into passes of similar length, so a text's
    /// vector does not depend on what else is in the request.
    pub fn embed(&self, texts: &[&str], kind: TextKind) -> Result<Vec<Vec<f32>>, ModelError> {
        let encodings = self.encode(texts, kind)?;
        let lengths: Vec<usize> = encodings.iter().map(|e| e.get_ids().len()).collect();
        let mut out = vec![Vec::new(); texts.len()];
        for pass in batch::plan(&lengths, PASS_TEXTS, PASS_TOKENS) {
            let longest = pass.iter().map(|&i| lengths[i]).max().unwrap_or(0);
            let mut ids: Vec<u32> = Vec::with_capacity(pass.len() * longest);
            let mut mask: Vec<u32> = Vec::with_capacity(pass.len() * longest);
            for &i in &pass {
                let e = encodings[i].get_ids();
                ids.extend_from_slice(e);
                ids.extend(std::iter::repeat_n(self.pad_id, longest - e.len()));
                mask.extend(std::iter::repeat_n(1u32, e.len()));
                mask.extend(std::iter::repeat_n(0u32, longest - e.len()));
            }
            let shape = (pass.len(), longest);
            let words = Tensor::from_vec(
                self.words.gather(&ids)?,
                (pass.len(), longest, self.dim),
                &Device::Cpu,
            )?;
            let mask = Tensor::from_vec(mask, shape, &Device::Cpu)?;
            let positions =
                self.positions
                    .forward(&Tensor::arange(0u32, longest as u32, &Device::Cpu)?)?;
            let x = words
                .broadcast_add(&self.segment)?
                .broadcast_add(&positions)?;
            let hidden = self
                .encoder
                .forward(&self.norm.forward(&x)?, &attention_bias(&mask)?)?;
            let pooled = match self.spec.pooling {
                Pooling::Cls => pooling::cls(&hidden)?,
                Pooling::Mean => pooling::mean(&hidden, &mask)?,
            };
            let rows: Vec<Vec<f32>> = pooling::l2_normalize(&pooled)?.to_vec2()?;
            for (&i, row) in pass.iter().zip(rows) {
                out[i] = row;
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn model_dir(spec: ModelSpec) -> PathBuf {
        std::env::var_os("EMBEDDER_MODELS")
            .map_or_else(
                || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models"),
                PathBuf::from,
            )
            .join(spec.name)
    }

    struct Golden {
        text: &'static str,
        kind: TextKind,
        ids: &'static [u32],
        head: [f32; 12],
    }

    /// Token ids and the first components of the unit vector from the reference implementation
    /// (transformers 4.57, torch 2.8, CPU), following each model card: `[CLS]` for rubert-tiny2,
    /// masked mean with the `query: ` / `passage: ` prefixes for multilingual-e5.
    fn check_golden(spec: ModelSpec, cases: &[Golden]) {
        let e = Embedder::load(&model_dir(spec), spec).unwrap();
        for g in cases {
            let ids: Vec<u32> = e.encode(&[g.text], g.kind).unwrap()[0].get_ids().to_vec();
            assert_eq!(ids, g.ids, "{}", g.text);
            let v = &e.embed(&[g.text], g.kind).unwrap()[0];
            let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-5, "{norm}");
            for (i, (got, want)) in v.iter().zip(g.head).enumerate() {
                assert!(
                    (got - want).abs() < 1e-5,
                    "{} [{i}]: {got} != {want}",
                    g.text
                );
            }
        }
        let texts: Vec<&str> = cases.iter().map(|g| g.text).collect();
        let one_by_one: Vec<Vec<f32>> = texts
            .iter()
            .map(|t| e.embed(&[t], TextKind::Query).unwrap().remove(0))
            .collect();
        let together = e.embed(&texts, TextKind::Query).unwrap();
        for (a, b) in one_by_one.iter().zip(&together) {
            let diff = a
                .iter()
                .zip(b)
                .map(|(x, y)| (x - y).abs())
                .fold(0f32, f32::max);
            assert!(diff < 1e-5, "padding changed a vector by {diff}");
        }
    }

    const PAYMENTS: &str =
        "Go-разработчик в команду платежей: идемпотентные API, outbox, шардирование PostgreSQL.";
    const EVENTS: &str = "Писал консьюмеры потока событий и настраивал репликацию базы данных.";

    #[test]
    #[ignore = "needs the weights: make model"]
    fn rubert_tiny2_matches_the_reference() {
        check_golden(
            RUBERT_TINY2,
            &[
                Golden {
                    text: "привет мир",
                    kind: TextKind::Query,
                    ids: &[2, 32840, 9311, 3],
                    head: [
                        0.019348, -0.020398, -0.008455, -0.068583, -0.001672, 0.038617, -0.012737,
                        -0.031413, -0.073095, -0.045753, 0.019569, 0.072097,
                    ],
                },
                Golden {
                    text: PAYMENTS,
                    kind: TextKind::Query,
                    ids: &[
                        2, 3144, 17, 53834, 314, 7592, 38144, 30, 55080, 32396, 4406, 1308, 14374,
                        16, 1151, 10931, 16, 29134, 2253, 14273, 3196, 8176, 26501, 18, 3,
                    ],
                    head: [
                        -0.032695, -0.01013, -0.020953, -0.157738, 0.000583, 0.013273, -0.044127,
                        -0.032828, -0.043192, -0.024635, 0.03764, 0.063084,
                    ],
                },
                Golden {
                    text: EVENTS,
                    kind: TextKind::Passage,
                    ids: &[
                        2, 62902, 10310, 8506, 57242, 34506, 18914, 320, 42627, 34493, 65969, 4005,
                        26127, 10938, 18, 3,
                    ],
                    head: [
                        -0.014988, -0.038885, 0.044551, -0.07199, -0.024687, -0.027779, -0.075963,
                        0.019482, -0.020603, -0.001583, 0.04565, 0.036734,
                    ],
                },
            ],
        );
    }

    #[test]
    #[ignore = "needs the weights: make model MODEL=multilingual-e5-small"]
    fn multilingual_e5_small_matches_the_reference() {
        check_golden(
            MULTILINGUAL_E5_SMALL,
            &[
                Golden {
                    text: "привет мир",
                    kind: TextKind::Query,
                    ids: &[0, 41, 1294, 12, 146038, 11373, 2],
                    head: [
                        0.053546, 0.052436, -0.00933, -0.066602, 0.06358, -0.015372, 0.030129,
                        0.035596, 0.005456, 0.019467, 0.06917, 0.012803,
                    ],
                },
                Golden {
                    text: PAYMENTS,
                    kind: TextKind::Query,
                    ids: &[
                        0, 41, 1294, 12, 2016, 9, 160587, 49, 132782, 48895, 2519, 12, 35, 30462,
                        46817, 17134, 1874, 61687, 4, 1810, 11728, 4, 28435, 526, 36274, 2795,
                        7134, 158897, 5, 2,
                    ],
                    head: [
                        0.019597, -0.013212, -0.04549, -0.013757, 0.052467, -0.057171, 0.043295,
                        0.043447, 0.076184, -0.006268, 0.081178, 0.004781,
                    ],
                },
                Golden {
                    text: EVENTS,
                    kind: TextKind::Passage,
                    ids: &[
                        0, 46692, 12, 167653, 547, 4406, 4401, 743, 81330, 57661, 59, 128297, 35,
                        29, 7133, 89, 7846, 126565, 43112, 22959, 115906, 36129, 5, 2,
                    ],
                    head: [
                        0.019064, -0.01189, -0.013017, -0.08171, 0.073196, -0.039359, 0.00252,
                        0.015555, 0.037058, 0.047985, 0.040048, -0.022829,
                    ],
                },
            ],
        );
    }

    #[test]
    fn specs_follow_their_model_cards() {
        let rubert = spec("rubert-tiny2").unwrap();
        assert_eq!(rubert.pooling, Pooling::Cls);
        assert_eq!(rubert.prefix(TextKind::Query), "");
        let e5 = spec("multilingual-e5-small").unwrap();
        assert_eq!(e5.pooling, Pooling::Mean);
        assert_eq!(e5.prefix(TextKind::Query), "query: ");
        assert_eq!(e5.prefix(TextKind::Passage), "passage: ");
        assert!(spec("bge-m3").is_none());
    }
}
