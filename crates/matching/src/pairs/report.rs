use catalog::{Category, Grade, Specialization, WorkFormat};
use serde::Serialize;

use crate::scoring::ScoreBreakdown;

/// Where a value used for matching came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsOrigin))]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// Given in the input.
    Input,
    /// Recognized from the vacancy or resume text.
    Text,
    /// Neither: the field stays empty or a stated default is used.
    Missing,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsOriginCounts))]
pub struct OriginCounts {
    pub input: usize,
    pub text: usize,
    pub missing: usize,
}

impl OriginCounts {
    pub(crate) fn add(&mut self, o: Origin) {
        match o {
            Origin::Input => self.input += 1,
            Origin::Text => self.text += 1,
            Origin::Missing => self.missing += 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsCoverage))]
pub struct Coverage {
    pub vacancies: usize,
    pub vacancy_specialization: OriginCounts,
    pub vacancy_grades: OriginCounts,
    pub vacancy_skills: OriginCounts,
    /// Specialization guessed with confidence below 0.5 or not found, or no skills found.
    pub vacancies_low_confidence: Vec<String>,
    pub candidates: usize,
    pub candidate_specialization: OriginCounts,
    /// Among candidates with a category; `missing` = grade found nowhere, Middle assumed.
    pub candidate_grade: OriginCounts,
    /// No specialization anywhere: no category, so our ranking never shows them.
    pub candidates_without_category: Vec<String>,
    pub candidates_with_test: usize,
    pub candidates_with_fsp: usize,
    pub labels: usize,
    pub labeled_vacancies: usize,
}

/// Mean over vacancies and the half-width of its 95 % interval.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsMean))]
pub struct Mean {
    pub mean: f64,
    pub ci95: f64,
}

/// Quality of one ranking of one vacancy against its labels. Unlabeled candidates count as not
/// relevant; relevant means relevance ≥ 2.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsMetrics))]
pub struct Metrics {
    pub precision_at_5: f64,
    /// Precision at the run's `k`.
    pub precision_at_k: f64,
    pub ndcg_at_k: f64,
    pub average_precision: f64,
    pub reciprocal_rank: f64,
    /// Share of the shown top-k that carries a label.
    pub judged_at_k: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsMeanMetrics))]
pub struct MeanMetrics {
    pub precision_at_5: Mean,
    pub precision_at_k: Mean,
    pub ndcg_at_k: Mean,
    /// MAP.
    pub average_precision: Mean,
    /// MRR.
    pub reciprocal_rank: Mean,
    pub judged_at_k: Mean,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsMetricsSummary))]
pub struct MetricsSummary {
    /// Vacancies with at least one relevant label; the macro averages are over them.
    pub vacancies: usize,
    pub ours: MeanMetrics,
    pub baseline: MeanMetrics,
}

/// The need the vacancy was matched as.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsNeed))]
pub struct VacancySummary {
    pub specialization: Option<Specialization>,
    pub specialization_origin: Origin,
    pub specialization_confidence: Option<f32>,
    pub grades: Vec<Grade>,
    pub grades_origin: Origin,
    pub allow_adjacent_grades: bool,
    /// Display names.
    pub must_skills: Vec<String>,
    pub nice_skills: Vec<String>,
    pub skills_origin: Origin,
    pub salary_min: Option<i64>,
    pub salary_max: Option<i64>,
    pub work_formats: Vec<WorkFormat>,
    pub city: Option<String>,
    pub low_confidence: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsRanked))]
pub struct RankedPair {
    pub rank: usize,
    pub candidate_id: String,
    /// 0..100
    pub score: f32,
    pub relevance: Option<u8>,
    pub category: Category,
    /// Where the category came from and what was assumed, in Russian.
    pub notes: Vec<String>,
    pub breakdown: ScoreBreakdown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsBaselineRanked))]
pub struct BaselinePair {
    pub rank: usize,
    pub candidate_id: String,
    /// Vacancy skills (must + nice) found among the resume keywords.
    pub keywords: Vec<String>,
    pub experience_years: Option<f32>,
    pub relevance: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsVacancyReport))]
pub struct VacancyReport {
    pub id: String,
    /// First line of the text.
    pub title: String,
    pub need: VacancySummary,
    /// Candidates that passed the hard filters (specialization, grade, format, city).
    pub passed_filters: usize,
    pub labels: usize,
    pub relevant_labels: usize,
    /// Present when the vacancy has at least one relevant label.
    pub ours: Option<Metrics>,
    pub baseline: Option<Metrics>,
    pub top: Vec<RankedPair>,
    pub baseline_top: Vec<BaselinePair>,
    pub notes: Vec<String>,
}

/// How a resume was read.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsCandidateSummary))]
pub struct CandidateSummary {
    pub id: String,
    pub category: Option<Category>,
    pub specialization_origin: Origin,
    pub specialization_confidence: Option<f32>,
    pub grade_origin: Origin,
    pub test_result: Option<f32>,
    pub experience_years: Option<f32>,
    /// Canonical slugs from the input and the resume.
    pub skills: Vec<String>,
    pub has_fsp: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PairsReport {
    /// `YYYY-MM-DD`
    pub now: String,
    pub k: usize,
    /// Labels at or above this value count as relevant.
    pub relevant_from: u8,
    /// Neutral 0..1 test strength used for candidates without a test result.
    pub neutral_test_strength: f32,
    pub coverage: Coverage,
    /// Present when at least one vacancy has a relevant label.
    pub metrics: Option<MetricsSummary>,
    pub vacancies: Vec<VacancyReport>,
    pub candidates: Vec<CandidateSummary>,
}
