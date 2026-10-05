//! Hiring outcomes reported by the company's ATS, as the candidate and the company see them.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::dto::HiringStage;

/// «Компания отметила: собеседование» — a stage the company's hiring system reported.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HiringOutcome {
    pub stage: HiringStage,
    /// Reason the company gave; visible to the candidate.
    pub reason: Option<String>,
    pub at: DateTime<Utc>,
}

/// Outcomes grouped by the invitation or application they belong to.
pub struct OutcomeIndex(Vec<(Uuid, HiringOutcome)>);

impl OutcomeIndex {
    pub fn from_rows(rows: Vec<db::integrations::OutcomeRow>) -> Self {
        Self(
            rows.into_iter()
                .filter_map(|r| {
                    let subject = r.invitation_id.or(r.application_id)?;
                    Some((
                        subject,
                        HiringOutcome {
                            stage: r.stage.into(),
                            reason: r.reason,
                            at: r.occurred_at,
                        },
                    ))
                })
                .collect(),
        )
    }

    pub async fn for_candidate(pool: &PgPool, candidate_id: Uuid) -> Result<Self, sqlx::Error> {
        Ok(Self::from_rows(
            db::integrations::candidate_outcomes(pool, candidate_id).await?,
        ))
    }

    /// In the order the stages happened.
    pub fn of(&self, subject: Uuid) -> Vec<HiringOutcome> {
        self.0
            .iter()
            .filter(|(s, _)| *s == subject)
            .map(|(_, o)| o.clone())
            .collect()
    }
}
