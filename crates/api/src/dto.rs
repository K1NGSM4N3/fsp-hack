use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// API-facing mirror of a Postgres enum from the `db` crate, which stays free of OpenAPI derives.
macro_rules! mirror_enum {
    ($name:ident => $db:path { $($variant:ident),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }

        impl From<$db> for $name {
            fn from(v: $db) -> Self {
                match v { $(<$db>::$variant => $name::$variant),+ }
            }
        }

        impl From<$name> for $db {
            fn from(v: $name) -> Self {
                match v { $($name::$variant => <$db>::$variant),+ }
            }
        }
    };
}

mirror_enum!(InvitationStatus => db::types::InvitationStatus { Sent, Viewed, Accepted, Declined, Expired, Withdrawn });
mirror_enum!(AssessmentStatus => db::types::AssessmentStatus { InProgress, Passed, Failed, Expired });
mirror_enum!(SearchStatus => db::types::SearchStatus { Active, Open, Closed });
mirror_enum!(ApplicationStatus => db::types::ApplicationStatus { Sent, Viewed, Accepted, Rejected, Withdrawn });
mirror_enum!(TaskStatus => db::types::TaskAssignmentStatus { Assigned, Submitted, Reviewed, Skipped, Expired });
mirror_enum!(VacancyStatus => db::types::VacancyStatus { Draft, Published, Closed });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeclineReason {
    Salary,
    Stack,
    Format,
    Company,
    NotLooking,
    Other,
}

impl DeclineReason {
    pub const fn slug(self) -> &'static str {
        match self {
            DeclineReason::Salary => "salary",
            DeclineReason::Stack => "stack",
            DeclineReason::Format => "format",
            DeclineReason::Company => "company",
            DeclineReason::NotLooking => "not_looking",
            DeclineReason::Other => "other",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        Some(match s {
            "salary" => DeclineReason::Salary,
            "stack" => DeclineReason::Stack,
            "format" => DeclineReason::Format,
            "company" => DeclineReason::Company,
            "not_looking" => DeclineReason::NotLooking,
            "other" => DeclineReason::Other,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SalaryRange {
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
}

pub fn parse_slugs<T: std::str::FromStr>(values: &[String]) -> Vec<T> {
    values.iter().filter_map(|v| v.parse().ok()).collect()
}

pub fn slugs<T: std::fmt::Display>(values: &[T]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}
