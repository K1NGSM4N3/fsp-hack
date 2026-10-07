//! A Unigram (SentencePiece) tokenizer model that fits a 250 000-piece vocabulary into ~20 MB.
//!
//! `tokenizers` keeps the Unigram vocabulary in a byte trie with a hash map in every node, which
//! for multilingual-e5's vocabulary takes ~390 MB of resident memory — more than the encoder
//! itself. This model keeps one hash map from piece to id and finds the pieces starting at a
//! position by looking up its prefixes, up to the longest piece. The best-path search is the one of
//! `tokenizers` (`Unigram::encode_optimized`, after SentencePiece): same scores, the same unknown
//! penalty, unknown characters fused, so the ids are the same. Normalizer, pre-tokenizer, post-
//! processor and added tokens are the library's, read from the same tokenizer.json.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use rustc_hash::FxHashMap;
use serde::Deserialize;
use tokenizers::{
    AddedToken, DecoderWrapper, Model, NormalizerWrapper, PostProcessorWrapper,
    PreTokenizerWrapper, Token, TokenizerImpl, models::unigram::UnigramTrainer,
};

/// SentencePiece's penalty below the lowest piece score for a character no piece covers.
const UNK_PENALTY: f64 = 10.0;

pub type CompactTokenizer = TokenizerImpl<
    CompactUnigram,
    NormalizerWrapper,
    PreTokenizerWrapper,
    PostProcessorWrapper,
    DecoderWrapper,
>;

#[derive(Debug, thiserror::Error)]
pub enum UnigramError {
    #[error("tokenizer.json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("tokenizer.json: the model is {0}, not Unigram")]
    NotUnigram(String),
    #[error("tokenizer.json: unk_id {0} is outside the vocabulary")]
    Unk(usize),
    #[error("tokenizer.json: byte fallback is not supported")]
    ByteFallback,
    #[error("tokenizer: {0}")]
    Library(String),
}

/// Pieces, their scores and the unknown id, as `tokenizers` stores a Unigram model.
#[derive(Deserialize)]
struct UnigramJson {
    #[serde(rename = "type")]
    kind: String,
    unk_id: Option<usize>,
    vocab: Vec<(String, f64)>,
    #[serde(default)]
    byte_fallback: bool,
}

pub struct CompactUnigram {
    ids: FxHashMap<Box<str>, u32>,
    scores: Box<[f64]>,
    min_score: f64,
    unk_id: u32,
    /// Bytes of the longest piece: no piece starting at a position ends further away.
    longest: usize,
}

impl CompactUnigram {
    fn from_json(model: UnigramJson) -> Result<Self, UnigramError> {
        if model.kind != "Unigram" {
            return Err(UnigramError::NotUnigram(model.kind));
        }
        if model.byte_fallback {
            return Err(UnigramError::ByteFallback);
        }
        let unk = model.unk_id.unwrap_or(0);
        if unk >= model.vocab.len() {
            return Err(UnigramError::Unk(unk));
        }
        let mut ids: FxHashMap<Box<str>, u32> =
            FxHashMap::with_capacity_and_hasher(model.vocab.len(), Default::default());
        let mut scores = Vec::with_capacity(model.vocab.len());
        let (mut min_score, mut longest) = (f64::INFINITY, 0);
        for (id, (piece, score)) in model.vocab.into_iter().enumerate() {
            min_score = min_score.min(score);
            longest = longest.max(piece.len());
            scores.push(score);
            ids.insert(piece.into_boxed_str(), id as u32);
        }
        Ok(CompactUnigram {
            ids,
            scores: scores.into_boxed_slice(),
            min_score,
            unk_id: unk as u32,
            longest,
        })
    }

    fn score(&self, id: u32) -> f64 {
        self.scores
            .get(id as usize)
            .copied()
            .unwrap_or(self.min_score)
    }

    /// Best segmentation of one pre-token: `(id, start, end)` byte spans, unknown runs fused.
    fn segment(&self, s: &str) -> Vec<(u32, usize, usize)> {
        #[derive(Clone, Copy)]
        struct Best {
            id: u32,
            score: f64,
            from: Option<usize>,
        }
        let unk_score = self.min_score - UNK_PENALTY;
        let mut best = vec![
            Best {
                id: 0,
                score: 0.0,
                from: None,
            };
            s.len() + 1
        ];
        let boundaries: Vec<usize> = s
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(s.len()))
            .collect();
        for (k, &at) in boundaries.iter().enumerate().take(boundaries.len() - 1) {
            let here = best[at].score;
            let first_char_end = boundaries[k + 1];
            let mut single = false;
            for &end in boundaries[k + 1..]
                .iter()
                .take_while(|&&end| end - at <= self.longest)
            {
                let Some(&id) = self.ids.get(&s[at..end]) else {
                    continue;
                };
                let candidate = self.score(id) + here;
                let target = &mut best[end];
                if target.from.is_none() || candidate > target.score {
                    *target = Best {
                        id,
                        score: candidate,
                        from: Some(at),
                    };
                }
                single |= end == first_char_end;
            }
            if !single {
                let candidate = unk_score + here;
                let target = &mut best[first_char_end];
                if target.from.is_none() || candidate > target.score {
                    *target = Best {
                        id: self.unk_id,
                        score: candidate,
                        from: Some(at),
                    };
                }
            }
        }
        let mut out: Vec<(u32, usize, usize)> = Vec::new();
        let mut end = s.len();
        while end > 0 {
            let node = best[end];
            let from = node.from.unwrap_or(0);
            match out.last_mut() {
                Some(last) if node.id == self.unk_id && last.0 == self.unk_id => last.1 = from,
                _ => out.push((node.id, from, end)),
            }
            end = from;
        }
        out.reverse();
        // As the library does: a fused run of unknown characters that is itself a piece keeps that
        // piece's id.
        for t in &mut out {
            if t.0 == self.unk_id
                && let Some(&id) = self.ids.get(&s[t.1..t.2])
            {
                t.0 = id;
            }
        }
        out
    }
}

impl Model for CompactUnigram {
    type Trainer = UnigramTrainer;

    fn tokenize(&self, sequence: &str) -> tokenizers::Result<Vec<Token>> {
        Ok(self
            .segment(sequence)
            .into_iter()
            .map(|(id, start, end)| Token::new(id, sequence[start..end].to_owned(), (start, end)))
            .collect())
    }

    fn token_to_id(&self, token: &str) -> Option<u32> {
        self.ids.get(token).copied()
    }

    /// A scan: only decoding needs it, and embeddings are never decoded.
    fn id_to_token(&self, id: u32) -> Option<String> {
        self.ids
            .iter()
            .find(|(_, v)| **v == id)
            .map(|(k, _)| k.to_string())
    }

    fn get_vocab(&self) -> HashMap<String, u32> {
        self.ids.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    fn get_vocab_size(&self) -> usize {
        self.scores.len()
    }

    fn save(&self, _folder: &Path, _prefix: Option<&str>) -> tokenizers::Result<Vec<PathBuf>> {
        Ok(Vec::new())
    }

    fn get_trainer(&self) -> UnigramTrainer {
        UnigramTrainer::default()
    }
}

#[derive(Deserialize)]
struct AddedJson {
    content: String,
    #[serde(default)]
    single_word: bool,
    #[serde(default)]
    lstrip: bool,
    #[serde(default)]
    rstrip: bool,
    #[serde(default = "yes")]
    normalized: bool,
    #[serde(default)]
    special: bool,
}

fn yes() -> bool {
    true
}

#[derive(Deserialize)]
struct TokenizerJson {
    #[serde(default)]
    added_tokens: Vec<AddedJson>,
    normalizer: Option<NormalizerWrapper>,
    pre_tokenizer: Option<PreTokenizerWrapper>,
    post_processor: Option<PostProcessorWrapper>,
    decoder: Option<DecoderWrapper>,
    model: UnigramJson,
}

/// Whether a tokenizer.json holds a Unigram model, without building anything.
pub fn is_unigram(json: &[u8]) -> bool {
    #[derive(Deserialize)]
    struct Kind {
        model: KindModel,
    }
    #[derive(Deserialize)]
    struct KindModel {
        #[serde(rename = "type")]
        kind: String,
    }
    serde_json::from_slice::<Kind>(json).is_ok_and(|k| k.model.kind == "Unigram")
}

pub fn load(json: &[u8]) -> Result<CompactTokenizer, UnigramError> {
    let parsed: TokenizerJson = serde_json::from_slice(json)?;
    let library = |e: tokenizers::Error| UnigramError::Library(e.to_string());
    let mut tokenizer = TokenizerImpl::new(CompactUnigram::from_json(parsed.model)?);
    tokenizer
        .with_normalizer(parsed.normalizer)
        .map_err(library)?;
    tokenizer.with_pre_tokenizer(parsed.pre_tokenizer);
    tokenizer.with_post_processor(parsed.post_processor);
    tokenizer.with_decoder(parsed.decoder);
    let (special, plain): (Vec<AddedJson>, Vec<AddedJson>) =
        parsed.added_tokens.into_iter().partition(|t| t.special);
    let added = |t: AddedJson| AddedToken {
        content: t.content,
        single_word: t.single_word,
        lstrip: t.lstrip,
        rstrip: t.rstrip,
        normalized: t.normalized,
        special: t.special,
    };
    tokenizer
        .add_special_tokens(special.into_iter().map(added))
        .map_err(library)?;
    tokenizer
        .add_tokens(plain.into_iter().map(added))
        .map_err(library)?;
    Ok(tokenizer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(vocab: &[(&str, f64)]) -> CompactUnigram {
        CompactUnigram::from_json(UnigramJson {
            kind: "Unigram".into(),
            unk_id: Some(0),
            vocab: vocab.iter().map(|(p, s)| ((*p).to_owned(), *s)).collect(),
            byte_fallback: false,
        })
        .unwrap()
    }

    fn pieces(m: &CompactUnigram, s: &str) -> Vec<String> {
        m.segment(s)
            .into_iter()
            .map(|(id, a, b)| format!("{id}:{}", &s[a..b]))
            .collect()
    }

    #[test]
    fn best_path_prefers_the_higher_total_score() {
        let m = model(&[
            ("<unk>", 0.0),
            ("▁а", -2.0),
            ("б", -2.0),
            ("▁аб", -5.0),
            ("▁абв", -3.0),
            ("в", -1.0),
        ]);
        assert_eq!(pieces(&m, "▁абв"), ["4:▁абв"]);
        assert_eq!(pieces(&m, "▁аб"), ["1:▁а", "2:б"]);
    }

    #[test]
    fn uncovered_characters_become_one_fused_unknown() {
        let m = model(&[("<unk>", 0.0), ("▁", -1.0), ("а", -1.0), ("б", -2.0)]);
        assert_eq!(pieces(&m, "▁а漢字б"), ["1:▁", "2:а", "0:漢字", "3:б"]);
        assert_eq!(pieces(&m, "漢"), ["0:漢"]);
        assert!(pieces(&m, "").is_empty());
    }

    #[test]
    fn json_errors_are_reported() {
        let json = |kind: &str, unk: usize| {
            serde_json::from_value::<UnigramJson>(serde_json::json!({
                "type": kind, "unk_id": unk, "vocab": [["<unk>", 0.0]]
            }))
            .unwrap()
        };
        assert!(matches!(
            CompactUnigram::from_json(json("BPE", 0)),
            Err(UnigramError::NotUnigram(_))
        ));
        assert!(matches!(
            CompactUnigram::from_json(json("Unigram", 3)),
            Err(UnigramError::Unk(3))
        ));
        assert!(is_unigram(br#"{"model": {"type": "Unigram"}}"#));
        assert!(!is_unigram(br#"{"model": {"type": "WordPiece"}}"#));
    }

    #[test]
    #[ignore = "needs the weights: make model MODEL=multilingual-e5-small"]
    fn ids_equal_the_library_on_the_e5_vocabulary() {
        let path = std::env::var_os("EMBEDDER_MODELS")
            .map_or_else(
                || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models"),
                PathBuf::from,
            )
            .join("multilingual-e5-small/tokenizer.json");
        let raw = std::fs::read(&path).unwrap();
        let library = tokenizers::Tokenizer::from_bytes(&raw).unwrap();
        let ours = load(&raw).unwrap();
        let mut texts: Vec<String> = [
            "Go-разработчик в команду платежей: идемпотентные API, outbox, шардирование PostgreSQL.",
            "query: Опыт разработки сервисов на Go и Kafka",
            "passage: Писал консьюмеры потока событий, настраивал репликацию базы данных.",
            "Ёжик в тумане; «кавычки» — тире… №5, 300 000 ₽, 3.5 %, e-mail: hr@corp.ru, https://x.example/a?b=c",
            "Senior SRE: Kubernetes, Linux internals, CI/CD.   Двойные   пробелы\tи\nпереводы строк",
            "漢字とかなの混じった文, Ελληνικά, العربية, עברית, हिन्दी, 한국어 😀🚀 ✓ ∑ ≤ ≥",
            "<s> <mask> </s> <pad> <unk> специальные токены в тексте",
            "",
            " ",
            "a",
            "ПРОПИСНЫЕ БУКВЫ И CamelCaseIdentifiers snake_case_names x86_64 ARMv8.2-A",
        ]
        .map(str::to_owned)
        .to_vec();
        let alphabet: Vec<char> =
            "абвгдеёжзийклмнопрстуфхцчшщъыьэюяАБВЖЯabcxyzXYZ0123456789 .,;:!?-—«»()[]{}/\\@#$%^&*+=_~`'\"\t\n€₽№…漢字αβγ😀\u{0301}\u{00a0}"
                .chars()
                .collect();
        let mut state = 0x2545_f491_4f6c_dd1du64;
        for _ in 0..3000 {
            let mut t = String::new();
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let len = (state % 60) as usize;
            for _ in 0..len {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                t.push(alphabet[(state % alphabet.len() as u64) as usize]);
            }
            texts.push(t);
        }
        for t in &texts {
            let want = library.encode(t.as_str(), true).unwrap();
            let got = ours.encode(t.as_str(), true).unwrap();
            assert_eq!(got.get_ids(), want.get_ids(), "{t:?}");
        }
    }
}
