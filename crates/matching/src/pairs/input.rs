use catalog::{Grade, Specialization, WorkFormat};
use serde::{Deserialize, Serialize};

use crate::fsp::Achievement;

/// One evaluation run: vacancies, candidates and optional expert labels. Structured fields are
/// optional overrides; whatever is missing is recognized from the texts by the same parsers the
/// API uses.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct PairsInput {
    /// Evaluation date `YYYY-MM-DD`: FSP result decay, "till now" in resumes. Default — today.
    #[serde(default)]
    pub now: Option<String>,
    /// Top-list length and the cutoff of Precision@k and nDCG@k, 1..=20. Default — 10.
    #[serde(default)]
    pub k: Option<usize>,
    pub vacancies: Vec<VacancyInput>,
    pub candidates: Vec<CandidateInput>,
    #[serde(default)]
    pub labels: Vec<LabelInput>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsVacancy))]
#[serde(deny_unknown_fields)]
pub struct VacancyInput {
    pub id: String,
    /// Vacancy text as published, title on the first line.
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub specialization: Option<Specialization>,
    /// Empty = recognized from the text; none there = any grade.
    #[serde(default)]
    pub grades: Vec<Grade>,
    /// Whether a candidate one grade off still qualifies (with a lower grade score).
    /// Default — yes: offline grades usually come from resumes, not from the test.
    #[serde(default)]
    pub allow_adjacent_grades: Option<bool>,
    /// Empty = recognized from the text. Slugs, aliases or free names.
    #[serde(default)]
    pub must_skills: Vec<String>,
    #[serde(default)]
    pub nice_skills: Vec<String>,
    /// Rubles per month.
    #[serde(default)]
    pub salary_min: Option<i64>,
    #[serde(default)]
    pub salary_max: Option<i64>,
    #[serde(default)]
    pub work_formats: Vec<WorkFormat>,
    #[serde(default)]
    pub city: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsCandidate))]
#[serde(deny_unknown_fields)]
pub struct CandidateInput {
    pub id: String,
    /// Plain resume text.
    #[serde(default)]
    pub resume_text: String,
    /// Missing specialization or grade is taken from the resume and marked as not confirmed
    /// by a test; without a recognizable specialization the candidate gets no category.
    #[serde(default)]
    pub specialization: Option<Specialization>,
    #[serde(default)]
    pub grade: Option<Grade>,
    /// Declared skills on top of the ones found in the resume.
    #[serde(default)]
    pub skills: Vec<String>,
    /// Rubles per month.
    #[serde(default)]
    pub salary_expectation: Option<i64>,
    #[serde(default)]
    pub work_formats: Vec<WorkFormat>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub relocation: Option<bool>,
    /// 0..=100: position of the test result inside the grade's band (≈ percentile among
    /// candidates of the same grade). Missing = neutral 50, stated in the explanation.
    #[serde(default)]
    pub test_result: Option<f32>,
    #[serde(default)]
    pub fsp_achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = PairsLabel))]
#[serde(deny_unknown_fields)]
pub struct LabelInput {
    pub vacancy_id: String,
    pub candidate_id: String,
    /// 0 — not relevant, 1 — partially, 2 — relevant, 3 — strong fit.
    #[cfg_attr(feature = "openapi", schema(minimum = 0, maximum = 3))]
    pub relevance: u8,
}

pub const MAX_VACANCIES: usize = 200;
pub const MAX_CANDIDATES: usize = 5_000;
pub const MAX_LABELS: usize = 100_000;
pub const MAX_TEXT_CHARS: usize = 20_000;
pub const MAX_ID_CHARS: usize = 100;
pub const MAX_LIST_ITEMS: usize = 100;
pub const MAX_SHORT_CHARS: usize = 200;
pub const DEFAULT_K: usize = 10;
pub const MAX_K: usize = 20;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PairsError {
    #[error("{list}: не больше {max}, передано {got}")]
    TooMany {
        list: &'static str,
        got: usize,
        max: usize,
    },
    #[error("{list}: нужен хотя бы один элемент")]
    Empty { list: &'static str },
    #[error("{path}: {message}")]
    Invalid {
        field: &'static str,
        path: String,
        message: String,
    },
}

impl PairsError {
    /// Top-level input field the error belongs to.
    pub fn field(&self) -> &'static str {
        match self {
            PairsError::TooMany { list, .. } | PairsError::Empty { list } => list,
            PairsError::Invalid { field, .. } => field,
        }
    }

    fn invalid(field: &'static str, path: String, message: impl Into<String>) -> Self {
        PairsError::Invalid {
            field,
            path,
            message: message.into(),
        }
    }
}

fn count(list: &'static str, len: usize, max: usize) -> Result<(), PairsError> {
    if len > max {
        return Err(PairsError::TooMany {
            list,
            got: len,
            max,
        });
    }
    Ok(())
}

fn check_id(field: &'static str, i: usize, id: &str) -> Result<(), PairsError> {
    if id.trim().is_empty() || id.chars().count() > MAX_ID_CHARS {
        return Err(PairsError::invalid(
            field,
            format!("{field}[{i}].id"),
            format!("пустой или длиннее {MAX_ID_CHARS} символов"),
        ));
    }
    Ok(())
}

fn check_text(field: &'static str, path: String, text: &str, max: usize) -> Result<(), PairsError> {
    // Byte length bounds the char count from above: skip the scan for short texts.
    if text.len() > max && text.chars().count() > max {
        return Err(PairsError::invalid(
            field,
            path,
            format!("длиннее {max} символов"),
        ));
    }
    Ok(())
}

fn check_list<T>(field: &'static str, path: String, list: &[T]) -> Result<(), PairsError> {
    if list.len() > MAX_LIST_ITEMS {
        return Err(PairsError::invalid(
            field,
            path,
            format!("больше {MAX_LIST_ITEMS} элементов"),
        ));
    }
    Ok(())
}

fn check_names(field: &'static str, path: String, list: &[String]) -> Result<(), PairsError> {
    check_list(field, path.clone(), list)?;
    for (j, s) in list.iter().enumerate() {
        check_text(field, format!("{path}[{j}]"), s, MAX_SHORT_CHARS)?;
    }
    Ok(())
}

fn check_money(field: &'static str, path: String, v: Option<i64>) -> Result<(), PairsError> {
    if v.is_some_and(|v| v < 0) {
        return Err(PairsError::invalid(
            field,
            path,
            "зарплата не может быть отрицательной",
        ));
    }
    Ok(())
}

impl PairsInput {
    /// Size and value checks; ids and labels are resolved separately.
    pub fn check(&self) -> Result<(), PairsError> {
        if self.vacancies.is_empty() {
            return Err(PairsError::Empty { list: "vacancies" });
        }
        if self.candidates.is_empty() {
            return Err(PairsError::Empty { list: "candidates" });
        }
        count("vacancies", self.vacancies.len(), MAX_VACANCIES)?;
        count("candidates", self.candidates.len(), MAX_CANDIDATES)?;
        count("labels", self.labels.len(), MAX_LABELS)?;
        if let Some(k) = self.k
            && !(1..=MAX_K).contains(&k)
        {
            return Err(PairsError::invalid(
                "k",
                "k".into(),
                format!("от 1 до {MAX_K}"),
            ));
        }
        const V: &str = "vacancies";
        for (i, v) in self.vacancies.iter().enumerate() {
            check_id(V, i, &v.id)?;
            check_text(V, format!("{V}[{i}].text"), &v.text, MAX_TEXT_CHARS)?;
            check_list(V, format!("{V}[{i}].grades"), &v.grades)?;
            check_names(V, format!("{V}[{i}].must_skills"), &v.must_skills)?;
            check_names(V, format!("{V}[{i}].nice_skills"), &v.nice_skills)?;
            check_list(V, format!("{V}[{i}].work_formats"), &v.work_formats)?;
            if let Some(city) = &v.city {
                check_text(V, format!("{V}[{i}].city"), city, MAX_SHORT_CHARS)?;
            }
            check_money(V, format!("{V}[{i}].salary_min"), v.salary_min)?;
            check_money(V, format!("{V}[{i}].salary_max"), v.salary_max)?;
            if let (Some(lo), Some(hi)) = (v.salary_min, v.salary_max)
                && lo > hi
            {
                return Err(PairsError::invalid(
                    V,
                    format!("{V}[{i}].salary_min"),
                    "«от» больше «до»",
                ));
            }
        }
        const C: &str = "candidates";
        for (i, c) in self.candidates.iter().enumerate() {
            check_id(C, i, &c.id)?;
            check_text(
                C,
                format!("{C}[{i}].resume_text"),
                &c.resume_text,
                MAX_TEXT_CHARS,
            )?;
            check_names(C, format!("{C}[{i}].skills"), &c.skills)?;
            check_list(C, format!("{C}[{i}].work_formats"), &c.work_formats)?;
            check_list(C, format!("{C}[{i}].fsp_achievements"), &c.fsp_achievements)?;
            if let Some(city) = &c.city {
                check_text(C, format!("{C}[{i}].city"), city, MAX_SHORT_CHARS)?;
            }
            check_money(
                C,
                format!("{C}[{i}].salary_expectation"),
                c.salary_expectation,
            )?;
            if c.test_result.is_some_and(|t| !(0.0..=100.0).contains(&t)) {
                return Err(PairsError::invalid(
                    C,
                    format!("{C}[{i}].test_result"),
                    "число от 0 до 100",
                ));
            }
        }
        for (i, l) in self.labels.iter().enumerate() {
            if l.relevance > 3 {
                return Err(PairsError::invalid(
                    "labels",
                    format!("labels[{i}].relevance"),
                    "от 0 до 3",
                ));
            }
        }
        Ok(())
    }
}
