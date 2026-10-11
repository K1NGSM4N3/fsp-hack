//! Item bank of parametric templates (README.md section 4.1).
//!
//! A template is an *item model*: given `(level, seed)` it deterministically renders a
//! concrete item and computes its answer. The level ("radicals") sets difficulty and has its
//! own prior IRT calibration; the seed only varies incidentals (numbers, names, option order).
//! The server stores just an [`ItemRef`] and regenerates the instance to check an answer.

mod backend;
mod devops;
mod frontend;
mod ml;
mod pool;
mod pools;
mod qa;
mod security;
mod shared;
mod util;

use std::sync::LazyLock;

use catalog::Specialization;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::irt::ItemParams;

pub(crate) type Rng = ChaCha8Rng;

/// Bumped whenever calibration or generators change so that old `ItemRef`s render differently.
pub const BANK_VERSION: u32 = 2;

/// Every single-choice item shows exactly this many options, so `c = 1 / CHOICE_OPTIONS`.
pub const CHOICE_OPTIONS: usize = 4;

/// Skill slugs shared with the `matching` crate.
pub const SKILLS: &[&str] = &[
    "python",
    "javascript",
    "typescript",
    "go",
    "java",
    "kotlin",
    "rust",
    "cpp",
    "csharp",
    "php",
    "sql",
    "bash",
    "postgresql",
    "mysql",
    "mongodb",
    "redis",
    "kafka",
    "rabbitmq",
    "clickhouse",
    "elasticsearch",
    "grpc",
    "rest",
    "graphql",
    "http",
    "django",
    "fastapi",
    "spring",
    "nodejs",
    "react",
    "vue",
    "angular",
    "nextjs",
    "html",
    "css",
    "redux",
    "docker",
    "kubernetes",
    "linux",
    "terraform",
    "ansible",
    "ci_cd",
    "prometheus",
    "nginx",
    "networking",
    "helm",
    "manual_testing",
    "test_design",
    "selenium",
    "playwright",
    "pytest",
    "postman",
    "load_testing",
    "api_testing",
    "pandas",
    "numpy",
    "scikit_learn",
    "pytorch",
    "statistics",
    "ml_metrics",
    "nlp",
    "computer_vision",
    "spark",
    "owasp",
    "pentest",
    "cryptography",
    "network_security",
    "appsec",
    "git",
    "algorithms",
    "data_structures",
    "system_design",
    "concurrency",
    "unit_testing",
];

/// Bit set over [`SKILLS`]; unknown slugs are ignored.
pub(crate) fn skill_mask<S: AsRef<str>>(skills: impl IntoIterator<Item = S>) -> u128 {
    skills.into_iter().fold(0, |mask, s| {
        match SKILLS.iter().position(|k| *k == s.as_ref()) {
            Some(i) => mask | (1u128 << i),
            None => mask,
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Topic {
    Algorithms,
    Databases,
    Web,
    Languages,
    Architecture,
    Tools,
    Javascript,
    Layout,
    Frameworks,
    Networking,
    Linux,
    Containers,
    Observability,
    TestDesign,
    QaProcess,
    TestAutomation,
    Statistics,
    MlMetrics,
    Modeling,
    DataTools,
    Appsec,
    Cryptography,
    NetworkSecurity,
}

impl Topic {
    pub const ALL: &'static [Topic] = &[
        Topic::Algorithms,
        Topic::Databases,
        Topic::Web,
        Topic::Languages,
        Topic::Architecture,
        Topic::Tools,
        Topic::Javascript,
        Topic::Layout,
        Topic::Frameworks,
        Topic::Networking,
        Topic::Linux,
        Topic::Containers,
        Topic::Observability,
        Topic::TestDesign,
        Topic::QaProcess,
        Topic::TestAutomation,
        Topic::Statistics,
        Topic::MlMetrics,
        Topic::Modeling,
        Topic::DataTools,
        Topic::Appsec,
        Topic::Cryptography,
        Topic::NetworkSecurity,
    ];

    pub const fn name_ru(self) -> &'static str {
        match self {
            Topic::Algorithms => "Алгоритмы и структуры данных",
            Topic::Databases => "Базы данных и SQL",
            Topic::Web => "HTTP и веб",
            Topic::Languages => "Языки программирования",
            Topic::Architecture => "Архитектура и надёжность",
            Topic::Tools => "Git и командная строка",
            Topic::Javascript => "JavaScript",
            Topic::Layout => "HTML и CSS",
            Topic::Frameworks => "React",
            Topic::Networking => "Сети",
            Topic::Linux => "Linux и автоматизация",
            Topic::Containers => "Контейнеры и оркестрация",
            Topic::Observability => "Мониторинг и SRE",
            Topic::TestDesign => "Тест-дизайн",
            Topic::QaProcess => "Процессы тестирования",
            Topic::TestAutomation => "Автоматизация и API",
            Topic::Statistics => "Статистика и вероятности",
            Topic::MlMetrics => "Метрики качества моделей",
            Topic::Modeling => "Обучение моделей",
            Topic::DataTools => "Инструменты анализа данных",
            Topic::Appsec => "Безопасность приложений",
            Topic::Cryptography => "Криптография",
            Topic::NetworkSecurity => "Сетевая безопасность",
        }
    }
}

/// Stable pointer to a concrete item: enough to regenerate the prompt and the answer.
///
/// The seed must never reach the client: generators are open source, so `(template, level,
/// seed)` is equivalent to the answer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ItemRef {
    pub template_id: String,
    pub level: u8,
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    SingleChoice,
    MultiChoice,
    Number,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CodeBlock {
    pub language: String,
    pub source: String,
}

/// What the candidate sees. Contains no answer key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ItemView {
    pub template_id: String,
    pub level: u8,
    pub title: String,
    pub topic: Topic,
    pub kind: ItemKind,
    /// Markdown (Russian).
    pub prompt: String,
    pub code: Option<CodeBlock>,
    /// Displayed options for choice kinds, empty otherwise.
    pub options: Vec<String>,
    pub input_hint: Option<String>,
    pub time_limit_secs: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Answer {
    /// Index into `ItemView::options`.
    Choice(usize),
    MultiChoice(Vec<usize>),
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
enum AnswerKey {
    Choice(usize),
    Multi(Vec<usize>),
    Number { value: f64, tolerance: f64 },
    Text(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemInstance {
    pub item: ItemRef,
    pub view: ItemView,
    key: AnswerKey,
}

impl ItemInstance {
    pub fn check(&self, answer: &Answer) -> bool {
        match (&self.key, answer) {
            (AnswerKey::Choice(k), Answer::Choice(a)) => k == a,
            (AnswerKey::Multi(k), Answer::MultiChoice(a)) => {
                let mut a = a.clone();
                a.sort_unstable();
                a.dedup();
                *k == a
            }
            (AnswerKey::Number { value, tolerance }, Answer::Number(x)) => {
                within(*x, *value, *tolerance)
            }
            (AnswerKey::Number { value, tolerance }, Answer::Text(s)) => {
                util::parse_number(s).is_some_and(|x| within(x, *value, *tolerance))
            }
            (AnswerKey::Text(accepted), Answer::Text(s)) => {
                let s = util::normalize_text(s);
                accepted.iter().any(|a| util::normalize_text(a) == s)
            }
            _ => false,
        }
    }

    /// Server-side only: for review screens and simulations.
    pub fn correct_answer(&self) -> Answer {
        match &self.key {
            AnswerKey::Choice(k) => Answer::Choice(*k),
            AnswerKey::Multi(k) => Answer::MultiChoice(k.clone()),
            AnswerKey::Number { value, .. } => Answer::Number(*value),
            AnswerKey::Text(accepted) => {
                Answer::Text(accepted.first().cloned().unwrap_or_default())
            }
        }
    }

    /// Human-readable correct answer, e.g. for showing results after the session.
    pub fn correct_answer_text(&self) -> String {
        match &self.key {
            AnswerKey::Choice(k) => self.view.options.get(*k).cloned().unwrap_or_default(),
            AnswerKey::Multi(k) => k
                .iter()
                .filter_map(|i| self.view.options.get(*i).map(String::as_str))
                .collect::<Vec<_>>()
                .join("; "),
            AnswerKey::Number { value, .. } => util::fmt_number(*value),
            AnswerKey::Text(accepted) => accepted.first().cloned().unwrap_or_default(),
        }
    }
}

fn within(x: f64, value: f64, tolerance: f64) -> bool {
    (x - value).abs() <= tolerance + 1e-9 * value.abs().max(1.0)
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ItemError {
    #[error("unknown item template `{0}`")]
    UnknownTemplate(String),
    #[error("template `{template}` has no level {level}")]
    UnknownLevel { template: String, level: u8 },
}

/// Difficulty level ("radicals") of a template with its prior calibration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level {
    pub level: u8,
    pub params: ItemParams,
    pub time_limit_secs: u16,
}

impl Level {
    pub(crate) const fn new(level: u8, a: f64, b: f64, c: f64, time_limit_secs: u16) -> Self {
        Self {
            level,
            params: ItemParams::new(a, b, c),
            time_limit_secs,
        }
    }
}

/// Guessing parameter for single choice among [`CHOICE_OPTIONS`].
pub(crate) const C_CHOICE: f64 = 1.0 / CHOICE_OPTIONS as f64;
/// Guessing parameter for "select all that apply" over five options: one of 31 non-empty subsets.
pub(crate) const C_MULTI5: f64 = 1.0 / 31.0;
pub(crate) const C_OPEN: f64 = 0.0;

pub(crate) type GenFn = fn(u8, &mut Rng) -> Draft;

pub(crate) enum Source {
    Computed(GenFn),
    /// One slice of entries per level, in level order.
    Pool(&'static [&'static [pool::Entry]]),
}

pub struct Template {
    pub id: &'static str,
    pub title: &'static str,
    pub topic: Topic,
    pub specializations: &'static [Specialization],
    pub skills: &'static [&'static str],
    pub levels: &'static [Level],
    pub(crate) source: Source,
}

impl Template {
    pub fn level(&self, level: u8) -> Option<&Level> {
        self.levels.iter().find(|l| l.level == level)
    }

    pub fn is_pool(&self) -> bool {
        matches!(self.source, Source::Pool(_))
    }

    pub fn applies_to(&self, spec: Specialization) -> bool {
        self.specializations.contains(&spec)
    }
}

impl std::fmt::Debug for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Template")
            .field("id", &self.id)
            .field("topic", &self.topic)
            .field("levels", &self.levels)
            .finish_non_exhaustive()
    }
}

/// One administrable (template, level) pair in the hot selection tables.
pub(crate) type Slot = u16;

/// Struct-of-arrays over all slots: only what CAT selection touches per step.
pub(crate) struct HotSlots {
    pub a: Vec<f32>,
    pub b: Vec<f32>,
    pub c: Vec<f32>,
    pub template: Vec<u16>,
    pub level: Vec<u8>,
}

pub struct Bank {
    templates: Vec<&'static Template>,
    /// Template ids sorted for binary search: a closed set of a few dozen keys.
    by_id: Vec<(&'static str, u16)>,
    pub(crate) hot: HotSlots,
    pub(crate) template_topic: Vec<Topic>,
    pub(crate) template_skills: Vec<u128>,
    spec_slots: Vec<Vec<Slot>>,
}

static BANK: LazyLock<Bank> = LazyLock::new(Bank::build);

pub fn bank() -> &'static Bank {
    &BANK
}

impl Bank {
    fn build() -> Self {
        let templates: Vec<&'static Template> = [
            shared::TEMPLATES,
            backend::TEMPLATES,
            frontend::TEMPLATES,
            devops::TEMPLATES,
            qa::TEMPLATES,
            ml::TEMPLATES,
            security::TEMPLATES,
            pools::TEMPLATES,
        ]
        .into_iter()
        .flatten()
        .collect();

        let mut hot = HotSlots {
            a: Vec::new(),
            b: Vec::new(),
            c: Vec::new(),
            template: Vec::new(),
            level: Vec::new(),
        };
        let mut spec_slots = vec![Vec::new(); Specialization::ALL.len()];
        for (ti, t) in templates.iter().enumerate() {
            for l in t.levels {
                let slot = hot.a.len() as Slot;
                hot.a.push(l.params.a as f32);
                hot.b.push(l.params.b as f32);
                hot.c.push(l.params.c as f32);
                hot.template.push(ti as u16);
                hot.level.push(l.level);
                for s in t.specializations {
                    spec_slots[*s as usize].push(slot);
                }
            }
        }

        let mut by_id: Vec<(&'static str, u16)> = templates
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id, i as u16))
            .collect();
        by_id.sort_unstable_by_key(|(id, _)| *id);

        Self {
            by_id,
            template_topic: templates.iter().map(|t| t.topic).collect(),
            template_skills: templates
                .iter()
                .map(|t| skill_mask(t.skills.iter()))
                .collect(),
            templates,
            hot,
            spec_slots,
        }
    }

    pub fn templates(&self) -> &[&'static Template] {
        &self.templates
    }

    pub fn template(&self, id: &str) -> Option<&'static Template> {
        self.template_index(id).map(|i| self.templates[i as usize])
    }

    pub fn for_specialization(
        &self,
        spec: Specialization,
    ) -> impl Iterator<Item = &'static Template> + '_ {
        self.templates
            .iter()
            .copied()
            .filter(move |t| t.applies_to(spec))
    }

    /// Calibration of a concrete item — what a simulator needs to draw responses.
    pub fn params(&self, item: &ItemRef) -> Option<ItemParams> {
        self.template(&item.template_id)?
            .level(item.level)
            .map(|l| l.params)
    }

    pub(crate) fn template_index(&self, id: &str) -> Option<u16> {
        self.by_id
            .binary_search_by(|(k, _)| (*k).cmp(id))
            .ok()
            .map(|pos| self.by_id[pos].1)
    }

    pub(crate) fn template_at(&self, index: u16) -> &'static Template {
        self.templates[index as usize]
    }

    pub(crate) fn slots(&self, spec: Specialization) -> &[Slot] {
        &self.spec_slots[spec as usize]
    }

    pub(crate) fn slot_params(&self, slot: Slot) -> ItemParams {
        let s = slot as usize;
        ItemParams::new(
            self.hot.a[s] as f64,
            self.hot.b[s] as f64,
            self.hot.c[s] as f64,
        )
    }
}

pub fn generate(item: &ItemRef) -> Result<ItemInstance, ItemError> {
    let template = bank()
        .template(&item.template_id)
        .ok_or_else(|| ItemError::UnknownTemplate(item.template_id.clone()))?;
    let level = template
        .level(item.level)
        .ok_or_else(|| ItemError::UnknownLevel {
            template: item.template_id.clone(),
            level: item.level,
        })?;
    let mut rng = Rng::seed_from_u64(item.seed);
    let draft = match &template.source {
        Source::Computed(generate) => generate(item.level, &mut rng),
        Source::Pool(levels) => {
            let index = template
                .levels
                .iter()
                .position(|l| l.level == item.level)
                .unwrap_or(0);
            pool::generate(levels.get(index).copied().unwrap_or(&[]), &mut rng)
        }
    };
    Ok(finalize(template, level, item, draft, &mut rng))
}

pub fn render(item: &ItemRef) -> Result<ItemView, ItemError> {
    generate(item).map(|i| i.view)
}

/// Regenerates the instance and checks the answer; unknown references are never correct.
pub fn check(item: &ItemRef, answer: &Answer) -> bool {
    generate(item).is_ok_and(|i| i.check(answer))
}

pub(crate) struct Draft {
    prompt: String,
    code: Option<CodeBlock>,
    hint: Option<String>,
    body: Body,
}

enum Body {
    Choice {
        correct: String,
        distractors: Vec<String>,
    },
    Multi {
        options: Vec<(String, bool)>,
    },
    Number {
        value: f64,
        tolerance: f64,
    },
    Text {
        accepted: Vec<String>,
    },
}

impl Draft {
    /// Distractors are taken in the given order after dropping duplicates of each other and
    /// of the correct option, so generators list the most plausible mistakes first.
    pub(crate) fn choice(
        prompt: impl Into<String>,
        correct: impl Into<String>,
        distractors: impl IntoIterator<Item = String>,
    ) -> Self {
        Self::with_body(
            prompt,
            Body::Choice {
                correct: correct.into(),
                distractors: distractors.into_iter().collect(),
            },
        )
    }

    pub(crate) fn multi(prompt: impl Into<String>, options: Vec<(String, bool)>) -> Self {
        Self::with_body(prompt, Body::Multi { options }).hint("Отметьте все подходящие варианты")
    }

    pub(crate) fn number(prompt: impl Into<String>, value: f64, tolerance: f64) -> Self {
        Self::with_body(prompt, Body::Number { value, tolerance }).hint("Число")
    }

    pub(crate) fn integer(prompt: impl Into<String>, value: i64) -> Self {
        Self::with_body(
            prompt,
            Body::Number {
                value: value as f64,
                tolerance: 0.0,
            },
        )
        .hint("Целое число")
    }

    pub(crate) fn text(prompt: impl Into<String>, accepted: Vec<String>) -> Self {
        Self::with_body(prompt, Body::Text { accepted })
    }

    fn with_body(prompt: impl Into<String>, body: Body) -> Self {
        Self {
            prompt: prompt.into(),
            code: None,
            hint: None,
            body,
        }
    }

    pub(crate) fn code(mut self, language: &str, source: impl Into<String>) -> Self {
        self.code = Some(CodeBlock {
            language: language.to_owned(),
            source: source.into(),
        });
        self
    }

    pub(crate) fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Prepends a paragraph (data tables, logs) before the question.
    pub(crate) fn context(mut self, context: impl AsRef<str>) -> Self {
        self.prompt = format!("{}\n\n{}", context.as_ref(), self.prompt);
        self
    }
}

fn finalize(
    template: &Template,
    level: &Level,
    item: &ItemRef,
    draft: Draft,
    rng: &mut Rng,
) -> ItemInstance {
    let (kind, options, key) = match draft.body {
        Body::Choice {
            correct,
            distractors,
        } => {
            let mut options = vec![correct];
            for d in distractors {
                if options.len() == CHOICE_OPTIONS {
                    break;
                }
                if !options.iter().any(|o| o.trim() == d.trim()) {
                    options.push(d);
                }
            }
            let mut order: Vec<usize> = (0..options.len()).collect();
            order.shuffle(rng);
            let key = order.iter().position(|&i| i == 0).unwrap_or(0);
            let mut slots: Vec<Option<String>> = options.into_iter().map(Some).collect();
            let shuffled = order.iter().filter_map(|&i| slots[i].take()).collect();
            (ItemKind::SingleChoice, shuffled, AnswerKey::Choice(key))
        }
        Body::Multi { mut options } => {
            options.shuffle(rng);
            let key = options
                .iter()
                .enumerate()
                .filter(|(_, (_, ok))| *ok)
                .map(|(i, _)| i)
                .collect();
            let texts = options.into_iter().map(|(text, _)| text).collect();
            (ItemKind::MultiChoice, texts, AnswerKey::Multi(key))
        }
        Body::Number { value, tolerance } => (
            ItemKind::Number,
            Vec::new(),
            AnswerKey::Number { value, tolerance },
        ),
        Body::Text { accepted } => (ItemKind::Text, Vec::new(), AnswerKey::Text(accepted)),
    };
    ItemInstance {
        item: item.clone(),
        view: ItemView {
            template_id: template.id.to_owned(),
            level: level.level,
            title: template.title.to_owned(),
            topic: template.topic,
            kind,
            prompt: draft.prompt,
            code: draft.code,
            options,
            input_hint: draft.hint,
            time_limit_secs: u32::from(level.time_limit_secs),
        },
        key,
    }
}

#[cfg(test)]
mod tests;
