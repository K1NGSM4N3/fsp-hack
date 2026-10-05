//! Matching: skill dictionary, FSP score, candidate scoring with explanations, and parsers
//! for free-text needs and resumes, offline evaluation on external pairs. Pure logic: no IO,
//! no clock — time comes in as `now_unix`.

mod extract;
pub mod fsp;
mod lexicon;
pub mod metrics;
pub mod need_parse;
pub mod pairs;
pub mod profile;
pub mod resume;
mod ru;
pub mod scoring;
pub mod skills;
mod text;

pub use extract::SpecializationGuess;
pub use fsp::{
    Achievement, AchievementKind, Contribution, Discipline, EventLevel, FspScore, Place, Rank,
    fsp_score,
};
pub use lexicon::canonical_city;
pub use need_parse::{ParsedNeed, parse_need};
pub use profile::{
    CandidatePool, CandidateProfile, CandidateSkill, CandidateSnapshot, FspSummary, SkillSource,
};
pub use resume::{ParsedResume, parse_resume};
pub use scoring::{
    CategoryStats, FeatureKey, FeatureScore, FilterReason, Need, RankedCandidate, ScoreBreakdown,
    SkillMatch, StrengthRank, Weights, category_stats, explain_at, hard_filter, profile_strength,
    rank, rank_by_strength, rank_snapshots, score, test_strength, theta_for_strength,
};
pub use skills::{
    SkillDef, SkillGroup, SkillId, SkillView, extract_skills, lookup, normalize, search,
    similarity, typical_skills,
};
