//! Closeness of texts by meaning: cosine similarity of sentence embeddings computed by the
//! embedder service (`crates/embedder`). This module only does arithmetic on vectors it is given:
//! which texts are embedded, how vectors are packed for the ranking loop, and how a cosine turns
//! into the 0..1 «Близость описаний» feature and into the paraphrase signal of anti-fake.

/// Sentence vectors of many items packed for scanning: one `i8` code per component and the inverse
/// norm of the codes per row (0 = the item has no vector). A 312-dimensional vector takes 316 bytes
/// instead of 1 248, and the cosine is an integer dot product.
#[derive(Debug, Clone, Default)]
pub struct Embeddings {
    dim: usize,
    codes: Vec<i8>,
    inv_norms: Vec<f32>,
}

/// One vector to compare against [`Embeddings`], quantized the same way.
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    codes: Box<[i8]>,
    inv_norm: f32,
}

/// Codes `round(x × 127 / max|x|)`: the largest component uses the whole `i8` range. Returns the
/// inverse norm of the codes, 0 for a zero or non-finite vector.
fn quantize(v: &[f32], out: &mut [i8]) -> f32 {
    let max = v.iter().fold(0f32, |m, x| m.max(x.abs()));
    if max == 0.0 || v.iter().any(|x| !x.is_finite()) {
        out.fill(0);
        return 0.0;
    }
    let k = 127.0 / max;
    let mut norm2 = 0i64;
    for (c, x) in out.iter_mut().zip(v) {
        let q = (x * k).round().clamp(-127.0, 127.0) as i8;
        norm2 += i64::from(q) * i64::from(q);
        *c = q;
    }
    if norm2 == 0 {
        0.0
    } else {
        1.0 / (norm2 as f32).sqrt()
    }
}

/// The `i8` codes of a vector and its largest absolute component: what [`Embeddings`] and
/// [`Query`] keep, in a form that can be stored and read back exactly ([`from_codes`]).
pub fn to_codes(v: &[f32]) -> Option<(Vec<i8>, f32)> {
    let mut codes = vec![0i8; v.len()];
    let max = v.iter().fold(0f32, |m, x| m.max(x.abs()));
    (quantize(v, &mut codes) > 0.0).then_some((codes, max))
}

/// A vector whose codes are `codes` again: quantizing it gives back the same codes.
pub fn from_codes(codes: &[i8], max: f32) -> Vec<f32> {
    codes.iter().map(|&c| f32::from(c) * max / 127.0).collect()
}

fn dot(a: &[i8], b: &[i8]) -> i32 {
    a.iter()
        .zip(b)
        .map(|(&x, &y)| i32::from(x) * i32::from(y))
        .sum()
}

impl Query {
    pub fn new(v: &[f32]) -> Option<Query> {
        let mut codes = vec![0i8; v.len()].into_boxed_slice();
        let inv_norm = quantize(v, &mut codes);
        (inv_norm > 0.0).then_some(Query { codes, inv_norm })
    }

    pub fn dim(&self) -> usize {
        self.codes.len()
    }

    pub fn codes(&self) -> &[i8] {
        &self.codes
    }

    pub fn cosine(&self, other: &Query) -> Option<f32> {
        (self.dim() == other.dim())
            .then(|| dot(&self.codes, &other.codes) as f32 * self.inv_norm * other.inv_norm)
    }
}

impl Embeddings {
    pub fn with_capacity(dim: usize, rows: usize) -> Self {
        Embeddings {
            dim,
            codes: Vec::with_capacity(dim * rows),
            inv_norms: Vec::with_capacity(rows),
        }
    }

    /// Appends a row; a missing vector or one of another dimension is stored as absent.
    pub fn push(&mut self, v: Option<&[f32]>) {
        let start = self.codes.len();
        self.codes.resize(start + self.dim, 0);
        let inv = match v {
            Some(v) if v.len() == self.dim => quantize(v, &mut self.codes[start..]),
            _ => 0.0,
        };
        self.inv_norms.push(inv);
    }

    pub fn len(&self) -> usize {
        self.inv_norms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inv_norms.is_empty()
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn has(&self, row: usize) -> bool {
        self.inv_norms.get(row).is_some_and(|&n| n > 0.0)
    }

    /// Cosine between the row and the query; `None` without a vector or with another dimension.
    pub fn cosine(&self, row: usize, q: &Query) -> Option<f32> {
        let inv = *self.inv_norms.get(row)?;
        if inv == 0.0 || q.dim() != self.dim {
            return None;
        }
        let codes = self.codes.get(row * self.dim..(row + 1) * self.dim)?;
        Some(dot(codes, &q.codes) as f32 * inv * q.inv_norm)
    }

    /// Bytes held, for the pool's memory accounting.
    pub fn heap_bytes(&self) -> usize {
        self.codes.capacity() + 4 * self.inv_norms.capacity()
    }
}

/// How cosines of one model map to the 0..1 feature and where a paraphrase starts. Raw cosines are
/// not comparable across models: two unrelated vacancies of different specializations score ~0.78
/// with rubert-tiny2 and ~0.88 with multilingual-e5.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calibration {
    /// Cosine of a typical unrelated profile of the same specialization: closeness 0.
    pub floor: f32,
    /// Cosine reached only by the closest profiles: closeness 1.
    pub ceil: f32,
    /// From this cosine two vacancy texts are the same vacancy retold.
    pub paraphrase: f32,
}

impl Calibration {
    /// Linear between `floor` and `ceil`, clamped to 0..1.
    pub fn closeness(&self, cosine: f32) -> f32 {
        if !cosine.is_finite() || self.ceil <= self.floor {
            return 0.0;
        }
        ((cosine - self.floor) / (self.ceil - self.floor)).clamp(0.0, 1.0)
    }
}

/// Fitted by `crates/validate` on the training needs (docs/validation/matching.md): `floor` and
/// `ceil` are the 10th and 99th percentiles of need × same-specialization profile cosines;
/// `paraphrase` is the threshold with no false alarm on the paraphrase set, midway between the
/// closest different vacancy and the next retelling (docs/validation/paraphrase.md,
/// embeddings.md). multilingual-e5-small is the model in use; rubert-tiny2 is kept comparable.
const CALIBRATIONS: [(&str, Calibration); 2] = [
    (
        "rubert-tiny2",
        Calibration {
            floor: 0.689,
            ceil: 0.823,
            paraphrase: 0.918,
        },
    ),
    (
        "multilingual-e5-small",
        Calibration {
            floor: 0.88,
            ceil: 0.918,
            paraphrase: 0.953,
        },
    ),
];

/// Calibration of a model id `name@revision`; `None` for a model nobody calibrated, whose vectors
/// are then not used.
pub fn calibration(model: &str) -> Option<Calibration> {
    let name = model.split('@').next().unwrap_or(model);
    CALIBRATIONS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| *c)
}

/// From this closeness the explanation says the experience is close to the task by meaning.
pub const CLOSE_BY_MEANING: f32 = 0.6;

/// The text-closeness feature of one candidate for one need: by meaning when both have vectors of
/// one model, by keywords (the database's `ts_rank_cd`, normalized) otherwise.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TextCloseness {
    /// 0..1
    pub value: f32,
    pub by_meaning: bool,
}

impl TextCloseness {
    pub fn keywords(value: f32) -> Self {
        TextCloseness {
            value,
            by_meaning: false,
        }
    }

    pub fn meaning(value: f32) -> Self {
        TextCloseness {
            value,
            by_meaning: true,
        }
    }
}

/// Characters of a profile or vacancy text sent to the model; the encoder reads 512 tokens, and
/// 4 000 characters of Russian are about 1 000 of them.
pub const MAX_TEXT_CHARS: usize = 4_000;

fn push_part(out: &mut String, part: &str) {
    let part = part.trim();
    if part.is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(part);
}

fn clipped(mut s: String) -> String {
    if let Some((at, _)) = s.char_indices().nth(MAX_TEXT_CHARS) {
        s.truncate(at);
    }
    s
}

/// What a candidate's profile vector is computed from: roles, «о себе», then each job's position
/// and description. Company names are left out: they say nothing about the work and are hidden
/// from employers unless the candidate shows them.
pub fn candidate_text<'a>(
    roles: &[String],
    about: &str,
    experience: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> String {
    let mut out = String::new();
    push_part(&mut out, &roles.join(", "));
    push_part(&mut out, about);
    for (position, description) in experience {
        push_part(&mut out, position);
        push_part(&mut out, description);
    }
    clipped(out)
}

/// What a need's or a vacancy's vector is computed from: the title and the description.
pub fn posting_text(title: &str, description: &str) -> String {
    let mut out = String::new();
    push_part(&mut out, title);
    push_part(&mut out, description);
    clipped(out)
}

/// 64-bit FNV-1a of the exact text (the models are case-sensitive): a stored vector is reused while
/// the hash of the text it came from is unchanged.
pub fn text_hash(text: &str) -> i64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    }) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(v: &[f32]) -> Vec<f32> {
        let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        v.iter().map(|x| x / n).collect()
    }

    fn exact(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }

    /// Deterministic pseudo-random unit vectors spread like sentence embeddings.
    fn vectors(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
        let mut s = seed;
        let mut next = || {
            s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            ((s >> 33) as f32 / (1u64 << 31) as f32) - 0.5
        };
        (0..n)
            .map(|_| unit(&(0..dim).map(|_| next()).collect::<Vec<_>>()))
            .collect()
    }

    #[test]
    fn quantized_cosine_stays_within_a_hundredth_of_the_exact_one() {
        let vs = vectors(40, 312, 7);
        let mut table = Embeddings::with_capacity(312, vs.len());
        for v in &vs {
            table.push(Some(v));
        }
        let mut worst = 0f32;
        for (i, a) in vs.iter().enumerate() {
            let q = Query::new(a).unwrap();
            for (j, b) in vs.iter().enumerate() {
                let got = table.cosine(j, &q).unwrap();
                worst = worst.max((got - exact(a, b)).abs());
                if i == j {
                    assert!((got - 1.0).abs() < 1e-5, "self-cosine {got}");
                }
            }
        }
        assert!(worst < 0.01, "worst error {worst}");
    }

    #[test]
    fn related_vectors_keep_their_order_after_quantization() {
        let base = vectors(1, 312, 1).remove(0);
        let noise = vectors(3, 312, 2);
        let mix = |t: f32, n: &[f32]| -> Vec<f32> {
            unit(
                &base
                    .iter()
                    .zip(n)
                    .map(|(b, x)| t * b + (1.0 - t) * x)
                    .collect::<Vec<_>>(),
            )
        };
        let close = mix(0.8, &noise[0]);
        let far = mix(0.4, &noise[1]);
        let q = Query::new(&base).unwrap();
        let (qc, qf) = (Query::new(&close).unwrap(), Query::new(&far).unwrap());
        assert!(q.cosine(&qc).unwrap() > q.cosine(&qf).unwrap());
        assert!((q.cosine(&qc).unwrap() - exact(&base, &close)).abs() < 0.01);
    }

    #[test]
    fn stored_codes_quantize_back_to_themselves() {
        for v in vectors(20, 384, 11) {
            let (codes, max) = to_codes(&v).unwrap();
            let back = from_codes(&codes, max);
            assert_eq!(to_codes(&back).unwrap().0, codes);
            assert_eq!(Query::new(&back), Query::new(&v));
        }
        assert_eq!(to_codes(&[0.0, 0.0]), None);
    }

    #[test]
    fn missing_zero_and_mismatched_vectors_are_absent() {
        let mut table = Embeddings::with_capacity(3, 4);
        table.push(Some(&[0.6, 0.8, 0.0]));
        table.push(None);
        table.push(Some(&[0.0, 0.0, 0.0]));
        table.push(Some(&[1.0, 0.0]));
        assert_eq!(table.len(), 4);
        assert!(table.has(0) && !table.has(1) && !table.has(2) && !table.has(3));
        let q = Query::new(&[0.6, 0.8, 0.0]).unwrap();
        assert!((table.cosine(0, &q).unwrap() - 1.0).abs() < 1e-5);
        assert_eq!(table.cosine(1, &q), None);
        assert_eq!(table.cosine(2, &q), None);
        assert_eq!(table.cosine(3, &q), None);
        assert_eq!(table.cosine(9, &q), None);
        assert_eq!(Query::new(&[0.0, 0.0]), None);
        assert_eq!(Query::new(&[f32::NAN, 1.0]), None);
        assert_eq!(q.cosine(&Query::new(&[1.0, 0.0]).unwrap()), None);
        assert_eq!(table.heap_bytes(), 12 + 16);
    }

    #[test]
    fn calibration_is_linear_between_floor_and_ceil() {
        let c = Calibration {
            floor: 0.5,
            ceil: 0.9,
            paraphrase: 0.95,
        };
        assert_eq!(c.closeness(0.3), 0.0);
        assert_eq!(c.closeness(0.5), 0.0);
        assert!((c.closeness(0.7) - 0.5).abs() < 1e-6);
        assert_eq!(c.closeness(0.9), 1.0);
        assert_eq!(c.closeness(0.99), 1.0);
        assert_eq!(c.closeness(f32::NAN), 0.0);
        let broken = Calibration { ceil: 0.5, ..c };
        assert_eq!(broken.closeness(0.7), 0.0);
    }

    #[test]
    fn calibration_is_looked_up_by_model_name_without_revision() {
        let c = calibration("rubert-tiny2@e8ed3b0").unwrap();
        assert_eq!(Some(c), calibration("rubert-tiny2"));
        assert!(c.floor < c.ceil && c.ceil <= c.paraphrase && c.paraphrase < 1.0);
        let e5 = calibration("multilingual-e5-small@614241f").unwrap();
        assert!(e5.floor < e5.ceil && e5.ceil <= e5.paraphrase && e5.paraphrase < 1.0);
        assert_eq!(calibration("bge-m3@abc"), None);
    }

    #[test]
    fn texts_join_non_empty_parts_and_leave_out_companies() {
        let text = candidate_text(
            &["Бэкенд-разработчик".into(), "Тимлид".into()],
            "  Пишу на Go.  ",
            [
                ("Go-разработчик", "Платёжный шлюз, Kafka."),
                ("", "  "),
                ("Стажёр", ""),
            ],
        );
        assert_eq!(
            text,
            "Бэкенд-разработчик, Тимлид\nПишу на Go.\nGo-разработчик\nПлатёжный шлюз, Kafka.\nСтажёр"
        );
        assert_eq!(candidate_text(&[], "", []), "");
        assert_eq!(
            posting_text("Go-разработчик", "Платежи"),
            "Go-разработчик\nПлатежи"
        );
        let long = posting_text("Заголовок", &"я".repeat(MAX_TEXT_CHARS * 2));
        assert_eq!(long.chars().count(), MAX_TEXT_CHARS);
    }

    #[test]
    fn hash_follows_the_exact_text() {
        assert_eq!(text_hash("Пишу на Go"), text_hash("Пишу на Go"));
        assert_ne!(text_hash("Пишу на Go"), text_hash("пишу на go"));
        assert_ne!(text_hash(""), text_hash(" "));
        assert_eq!(text_hash(""), 0xcbf2_9ce4_8422_2325u64 as i64);
    }
}
