//! Shared reference data used by every other crate: directions, specializations,
//! grades, categories, work formats and industries.
//!
//! Slugs are stable identifiers stored in the database and sent over the API —
//! never rename them, only add new ones.

use serde::{Deserialize, Serialize};

macro_rules! slug_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $($variant:ident => $slug:literal, $ru:literal;)+ }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant,)+ }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant,)+];

            pub const fn slug(self) -> &'static str {
                match self { $($name::$variant => $slug,)+ }
            }

            pub const fn name_ru(self) -> &'static str {
                match self { $($name::$variant => $ru,)+ }
            }

            pub fn from_slug(slug: &str) -> Option<Self> {
                match slug { $($slug => Some($name::$variant),)+ _ => None }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.slug())
            }
        }

        impl std::str::FromStr for $name {
            type Err = UnknownSlug;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::from_slug(s).ok_or_else(|| UnknownSlug(s.to_owned()))
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownSlug(pub String);

impl std::fmt::Display for UnknownSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown slug `{}`", self.0)
    }
}

impl std::error::Error for UnknownSlug {}

slug_enum! {
    /// Top-level IT direction ("отрасль" in the survey).
    Direction {
        Development => "development", "Разработка";
        Infrastructure => "infrastructure", "Инфраструктура";
        Quality => "quality", "Качество";
        DataAi => "data_ai", "Данные и ИИ";
        Security => "security", "Безопасность";
    }
}

slug_enum! {
    Specialization {
        Backend => "backend", "Бэкенд-разработка";
        Frontend => "frontend", "Фронтенд-разработка";
        Devops => "devops", "DevOps / SRE";
        Qa => "qa", "Тестирование (QA)";
        Ml => "ml", "ML / Data Science";
        Security => "security", "Информационная безопасность";
    }
}

impl Specialization {
    pub const fn direction(self) -> Direction {
        match self {
            Specialization::Backend | Specialization::Frontend => Direction::Development,
            Specialization::Devops => Direction::Infrastructure,
            Specialization::Qa => Direction::Quality,
            Specialization::Ml => Direction::DataAi,
            Specialization::Security => Direction::Security,
        }
    }
}

impl Direction {
    pub fn specializations(self) -> impl Iterator<Item = Specialization> {
        Specialization::ALL
            .iter()
            .copied()
            .filter(move |s| s.direction() == self)
    }
}

slug_enum! {
    /// Grades are ordered: `Junior < Middle < Senior`.
    Grade {
        Junior => "junior", "Junior";
        Middle => "middle", "Middle";
        Senior => "senior", "Senior";
    }
}

impl Grade {
    /// Lower bound of the grade on the IRT ability scale (θ) of a specialization.
    /// A candidate passes a grade test when the ability estimate is at or above it.
    /// See architecture.md §5.3.
    pub const fn theta_cut(self) -> f64 {
        match self {
            Grade::Junior => -1.0,
            Grade::Middle => 0.0,
            Grade::Senior => 1.0,
        }
    }

    pub const fn next(self) -> Option<Grade> {
        match self {
            Grade::Junior => Some(Grade::Middle),
            Grade::Middle => Some(Grade::Senior),
            Grade::Senior => None,
        }
    }

    pub const fn prev(self) -> Option<Grade> {
        match self {
            Grade::Junior => None,
            Grade::Middle => Some(Grade::Junior),
            Grade::Senior => Some(Grade::Middle),
        }
    }

    /// Grade whose θ range contains `theta`, or `None` below the junior cut.
    pub fn for_theta(theta: f64) -> Option<Grade> {
        Grade::ALL
            .iter()
            .rev()
            .copied()
            .find(|g| theta >= g.theta_cut())
    }

    /// Signed distance in grade steps (`Senior - Junior == 2`).
    pub const fn steps_from(self, other: Grade) -> i8 {
        self as i8 - other as i8
    }
}

slug_enum! {
    WorkFormat {
        Office => "office", "Офис";
        Hybrid => "hybrid", "Гибрид";
        Remote => "remote", "Удалённо";
    }
}

slug_enum! {
    /// Business domain of past experience — a soft matching signal, never a category.
    Industry {
        Fintech => "fintech", "Финтех и банки";
        Ecommerce => "ecommerce", "E-commerce и ритейл";
        Gamedev => "gamedev", "Геймдев";
        Govtech => "govtech", "Госсектор";
        Telecom => "telecom", "Телеком";
        Edtech => "edtech", "Образование";
        Healthtech => "healthtech", "Медицина";
        Media => "media", "Медиа и контент";
        Industry40 => "industry", "Промышленность";
        Logistics => "logistics", "Логистика и транспорт";
        Outsourcing => "outsourcing", "Заказная разработка";
    }
}

/// A category is the unit employers browse: specialization × grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Category {
    pub specialization: Specialization,
    pub grade: Grade,
}

impl Category {
    pub const fn new(specialization: Specialization, grade: Grade) -> Self {
        Self { specialization, grade }
    }

    /// Stable key, e.g. `backend:middle`.
    pub fn key(self) -> String {
        format!("{}:{}", self.specialization.slug(), self.grade.slug())
    }

    pub fn parse(key: &str) -> Option<Self> {
        let (spec, grade) = key.split_once(':')?;
        Some(Self::new(
            Specialization::from_slug(spec)?,
            Grade::from_slug(grade)?,
        ))
    }

    pub fn name_ru(self) -> String {
        format!("{} · {}", self.specialization.name_ru(), self.grade.name_ru())
    }

    pub fn all() -> impl Iterator<Item = Category> {
        Specialization::ALL.iter().flat_map(|&s| {
            Grade::ALL.iter().map(move |&g| Category::new(s, g))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_round_trip() {
        for s in Specialization::ALL {
            assert_eq!(Specialization::from_slug(s.slug()), Some(*s));
        }
        for c in Category::all() {
            assert_eq!(Category::parse(&c.key()), Some(c));
        }
        assert_eq!(Category::all().count(), 18);
    }

    #[test]
    fn grade_for_theta_uses_cuts() {
        assert_eq!(Grade::for_theta(-1.5), None);
        assert_eq!(Grade::for_theta(-1.0), Some(Grade::Junior));
        assert_eq!(Grade::for_theta(0.3), Some(Grade::Middle));
        assert_eq!(Grade::for_theta(2.0), Some(Grade::Senior));
        assert_eq!(Grade::Senior.steps_from(Grade::Junior), 2);
    }

    #[test]
    fn every_direction_has_specializations() {
        for d in Direction::ALL {
            assert!(d.specializations().next().is_some(), "{d}");
        }
    }

    #[test]
    fn serde_uses_slugs() {
        let json = serde_json::to_string(&Category::new(Specialization::Ml, Grade::Senior)).unwrap();
        assert_eq!(json, r#"{"specialization":"ml","grade":"senior"}"#);
    }
}
