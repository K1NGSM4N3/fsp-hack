//! Signals against fake vacancies and dishonest employers (X-1): corporate domain, salary ranges
//! far from the category market, near-duplicate vacancy texts, and the company trust level that
//! sets the weekly invitation quota. Pure logic: counters and texts come in, `now` is a parameter.

pub mod domain;
mod level;
pub mod salary;
pub mod similarity;

use catalog::Category;
use serde::{Deserialize, Serialize};

pub use domain::{DomainCheck, check_domain, display_domain};
pub use level::{
    ACCEPTED_FOR_BONUS, BASE_QUOTA, MATURE_DAYS, MAX_QUOTA, MIN_QUOTA, TrustAssessment, TrustLevel,
    TrustReason, TrustReasonKind, TrustSignals, assess, quota_rule,
};
pub use salary::{MarketBand, salary_flags};
pub use similarity::{DUPLICATE_THRESHOLD, Signature};

use crate::ru::rub;

/// Why a vacancy or an invitation was flagged. Stored as JSON next to the item; ids are kept as
/// strings because this crate knows nothing about the database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TrustFlag {
    SalaryAboveMarket {
        category: Category,
        max: i64,
        p90: i64,
    },
    SalaryWideRange {
        min: i64,
        max: i64,
    },
    /// Near-copy of the same company's vacancy.
    DuplicateOwn {
        vacancy_id: String,
        title: String,
        similarity: f32,
    },
    /// Near-copy of another company's published vacancy: goes to moderation.
    CopyOfOther {
        vacancy_id: String,
        company: String,
        title: String,
        similarity: f32,
    },
}

fn ratio(a: i64, b: i64) -> String {
    format!("{:.1}", a as f64 / b.max(1) as f64).replace('.', ",")
}

fn percent(similarity: f32) -> u32 {
    (similarity * 100.0).round() as u32
}

impl TrustFlag {
    pub fn is_salary(&self) -> bool {
        matches!(
            self,
            TrustFlag::SalaryAboveMarket { .. } | TrustFlag::SalaryWideRange { .. }
        )
    }

    /// Soft warning for the employer while filling the form: what we noticed and that nothing is
    /// blocked.
    pub fn employer_text(&self) -> String {
        match self {
            TrustFlag::SalaryAboveMarket { category, max, p90 } => format!(
                "Верх вилки {} заметно выше рынка: 90 % кандидатов категории {} ожидают не больше {}. Публикации это не мешает — проверьте, нет ли опечатки.",
                rub(*max),
                category.name_ru(),
                rub(*p90)
            ),
            TrustFlag::SalaryWideRange { min, max } => format!(
                "Вилка {} – {} очень широкая: верхняя граница больше нижней в {} раза. Так обычно описывают сразу несколько грейдов — кандидаты доверяют такой вилке меньше.",
                rub(*min),
                rub(*max),
                ratio(*max, *min)
            ),
            TrustFlag::DuplicateOwn {
                title, similarity, ..
            } => format!(
                "Текст почти совпадает с вашей вакансией «{title}» ({} %). Возможно, удобнее обновить её.",
                percent(*similarity)
            ),
            TrustFlag::CopyOfOther { .. } => {
                "Текст почти совпадает с вакансией другой компании.".into()
            }
        }
    }

    /// Reason in the moderation list.
    pub fn moderator_text(&self) -> String {
        match self {
            TrustFlag::SalaryAboveMarket { category, max, p90 } => format!(
                "Верх вилки {} — в {} раза выше 90-го перцентиля ожиданий {} ({})",
                rub(*max),
                ratio(*max, *p90),
                category.name_ru(),
                rub(*p90)
            ),
            TrustFlag::SalaryWideRange { min, max } => format!(
                "Вилка {} – {}: верх больше низа в {} раза",
                rub(*min),
                rub(*max),
                ratio(*max, *min)
            ),
            TrustFlag::DuplicateOwn {
                title, similarity, ..
            } => format!(
                "Почти повторяет собственную вакансию «{title}» ({} %)",
                percent(*similarity)
            ),
            TrustFlag::CopyOfOther {
                company,
                title,
                similarity,
                ..
            } => format!(
                "Почти дословно совпадает с вакансией «{title}» компании «{company}» ({} %)",
                percent(*similarity)
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use catalog::{Grade, Specialization};

    use super::*;

    #[test]
    fn flags_round_trip_as_tagged_json() {
        let flag = TrustFlag::SalaryAboveMarket {
            category: Category::new(Specialization::Qa, Grade::Junior),
            max: 400_000,
            p90: 130_000,
        };
        let json = serde_json::to_value(&flag).unwrap();
        assert_eq!(json["kind"], "salary_above_market");
        assert_eq!(serde_json::from_value::<TrustFlag>(json).unwrap(), flag);
        assert_eq!(
            flag.moderator_text(),
            "Верх вилки 400\u{a0}000\u{a0}₽ — в 3,1 раза выше 90-го перцентиля ожиданий Тестирование (QA) · Junior (130\u{a0}000\u{a0}₽)"
        );
        assert!(flag.is_salary());
    }

    #[test]
    fn copy_flag_names_the_original() {
        let flag = TrustFlag::CopyOfOther {
            vacancy_id: "x".into(),
            company: "Северный Код".into(),
            title: "Go-разработчик".into(),
            similarity: 0.914,
        };
        assert_eq!(
            flag.moderator_text(),
            "Почти дословно совпадает с вакансией «Go-разработчик» компании «Северный Код» (91 %)"
        );
        assert!(!flag.is_salary());
    }
}
