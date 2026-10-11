//! Assessment engine: IRT, adaptive testing (CAT), parametric item templates, grade policy.
//!
//! Pure logic only — no IO, no clock, no ambient randomness. Time arrives as unix seconds
//! (`i64`) and randomness as explicit `seed: u64`, so every function is deterministic and the
//! `validate` crate can exercise the exact code that runs in production.

pub mod cat;
pub mod irt;
pub mod items;
pub mod policy;
pub mod proctoring;

pub use cat::{
    AdministeredItem, CatConfig, Decision, PosteriorSummary, RecordError, SessionState, TestPlan,
    TopicPlan, decision, next_item, plan_for, record, sample_items, start, start_with,
};
pub use irt::{ItemParams, Posterior};
pub use items::{
    Answer, BANK_VERSION, Bank, CodeBlock, ItemError, ItemKind, ItemRef, ItemView, Template, Topic,
    bank, check, generate, render,
};
pub use policy::{
    ABANDON_AFTER_SECS, AttemptKind, GradePolicy, Outcome, PlacementStage, PolicyError,
};
pub use proctoring::{EventCounts, IntegrityEvent, ProctoringStatus};
