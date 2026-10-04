//! Grade-change policy (architecture §5.4, requirements T-5 / T-6).
//!
//! All timing is in unix seconds passed in as `now`; the crate never reads the clock. The
//! policy is a pure function of the candidate's assignment/attempt history and the current
//! time, returning which attempt kinds are allowed or why they are blocked.

use catalog::{Grade, Specialization};
use serde::{Deserialize, Serialize};

const DAY: i64 = 86_400;

/// Configurable intervals. Defaults follow architecture §5.4.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Minimum gap between grade changes after a category is assigned.
    pub change_cooldown_days: i64,
    /// Wait after a failed junior placement before retrying.
    pub retry_after_fail_days: i64,
    /// Wait after a failed upgrade attempt before the next attempt.
    pub retry_after_failed_upgrade_days: i64,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            change_cooldown_days: 90,
            retry_after_fail_days: 30,
            retry_after_failed_upgrade_days: 30,
        }
    }
}

impl PolicyConfig {
    fn cooldown(&self) -> i64 {
        self.change_cooldown_days * DAY
    }

    fn retry_fail(&self) -> i64 {
        self.retry_after_fail_days * DAY
    }

    fn retry_upgrade(&self) -> i64 {
        self.retry_after_failed_upgrade_days * DAY
    }
}

/// What a single session attempted and how it ended.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Attempt {
    pub specialization: Specialization,
    pub grade: Grade,
    pub kind: AttemptKind,
    pub passed: bool,
    /// Whether this attempt changed the candidate's active category.
    pub assigned: bool,
    pub at: i64,
}

/// The intent behind an attempt, which drives the next allowed action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum AttemptKind {
    /// First test on the claimed grade during onboarding.
    Placement,
    /// Immediate re-test one grade lower after failing placement (still placement, T-5).
    PlacementStepDown,
    /// Immediate upgrade test offered after a confident pass (one per placement, T-5).
    PlacementUpgrade,
    /// Post-assignment move up a grade (cooldown applies, T-6).
    Upgrade,
    /// Post-assignment move down a grade (cooldown applies, T-6).
    Downgrade,
    /// Retry after a 30-day block (failed junior placement or failed upgrade).
    Retry,
}

/// Where the candidate stands in the placement flow, derived from history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum PlacementStage {
    /// No category yet for this specialization.
    Unplaced,
    /// A category is assigned; changes are governed by the cooldown.
    Assigned { grade: Grade },
}

/// An attempt the candidate may start now.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AllowedAttempt {
    pub grade: Grade,
    pub kind: AttemptKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, thiserror::Error)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum PolicyError {
    /// A grade change is on cooldown; retry at `available_at` (unix seconds).
    #[error("grade change available at {available_at}")]
    ChangeCooldown { available_at: i64 },
    /// A failed attempt blocks retries until `available_at`.
    #[error("retry available at {available_at}")]
    RetryBlocked { available_at: i64 },
    /// The requested attempt is not valid from the current stage.
    #[error("attempt not allowed from the current state")]
    NotAllowed,
    /// Already at the grade bound (no grade above senior / below junior).
    #[error("no adjacent grade in that direction")]
    NoAdjacentGrade,
}

/// Grade policy over one candidate's history for a single specialization.
#[derive(Debug, Clone)]
pub struct GradePolicy {
    config: PolicyConfig,
}

impl Default for GradePolicy {
    fn default() -> Self {
        Self::new(PolicyConfig::default())
    }
}

impl GradePolicy {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    fn history<'a>(&self, spec: Specialization, attempts: &'a [Attempt]) -> Vec<&'a Attempt> {
        let mut h: Vec<&Attempt> = attempts
            .iter()
            .filter(|a| a.specialization == spec)
            .collect();
        h.sort_by_key(|a| a.at);
        h
    }

    /// Current active grade for the specialization (last assigning attempt), if any.
    pub fn current_grade(&self, spec: Specialization, attempts: &[Attempt]) -> Option<Grade> {
        self.history(spec, attempts)
            .iter()
            .rev()
            .find(|a| a.assigned)
            .map(|a| a.grade)
    }

    pub fn stage(&self, spec: Specialization, attempts: &[Attempt]) -> PlacementStage {
        match self.current_grade(spec, attempts) {
            Some(grade) => PlacementStage::Assigned { grade },
            None => PlacementStage::Unplaced,
        }
    }

    /// When the last assignment happened (start of the cooldown window).
    fn last_assignment_at(&self, spec: Specialization, attempts: &[Attempt]) -> Option<i64> {
        self.history(spec, attempts)
            .iter()
            .rev()
            .find(|a| a.assigned)
            .map(|a| a.at)
    }

    /// All attempt kinds the candidate may legitimately start right now, with reasons omitted.
    pub fn available(
        &self,
        spec: Specialization,
        attempts: &[Attempt],
        now: i64,
    ) -> Vec<AllowedAttempt> {
        let mut out = Vec::new();
        let kinds = match self.stage(spec, attempts) {
            PlacementStage::Unplaced => {
                vec![
                    AttemptKind::Placement,
                    AttemptKind::PlacementStepDown,
                    AttemptKind::Retry,
                ]
            }
            PlacementStage::Assigned { .. } => {
                vec![
                    AttemptKind::Upgrade,
                    AttemptKind::Downgrade,
                    AttemptKind::PlacementUpgrade,
                ]
            }
        };
        for kind in kinds {
            if let Ok(a) = self.check_kind(spec, attempts, kind, None, now) {
                out.push(a);
            }
        }
        out
    }

    /// Validates a specific requested attempt.
    pub fn check(
        &self,
        spec: Specialization,
        attempts: &[Attempt],
        kind: AttemptKind,
        grade: Grade,
        now: i64,
    ) -> Result<AllowedAttempt, PolicyError> {
        self.check_kind(spec, attempts, kind, Some(grade), now)
    }

    fn check_kind(
        &self,
        spec: Specialization,
        attempts: &[Attempt],
        kind: AttemptKind,
        grade: Option<Grade>,
        now: i64,
    ) -> Result<AllowedAttempt, PolicyError> {
        let history = self.history(spec, attempts);
        let stage = self.stage(spec, attempts);
        match kind {
            AttemptKind::Placement => self.check_placement(&history, grade, now),
            AttemptKind::PlacementStepDown => self.check_step_down(&history, grade, now),
            AttemptKind::PlacementUpgrade => self.check_placement_upgrade(stage, &history, now),
            AttemptKind::Upgrade => {
                self.check_change(spec, attempts, stage, Grade::next, grade, now)
            }
            AttemptKind::Downgrade => {
                self.check_change(spec, attempts, stage, Grade::prev, grade, now)
            }
            AttemptKind::Retry => self.check_retry(stage, &history, grade, now),
        }
    }

    fn check_placement(
        &self,
        history: &[&Attempt],
        grade: Option<Grade>,
        now: i64,
    ) -> Result<AllowedAttempt, PolicyError> {
        if history.iter().any(|a| a.assigned) {
            return Err(PolicyError::NotAllowed);
        }
        // After a failed attempt the only immediate option is one grade lower (step-down);
        // retaking the same or a higher grade waits out the retry window, otherwise the test
        // could be brute-forced by repeating it until the items happen to be easy.
        if let Some(last_fail) = history.iter().rev().find(|a| !a.passed && !a.assigned) {
            let available_at = last_fail.at + self.config.retry_fail();
            if now < available_at {
                return Err(PolicyError::RetryBlocked { available_at });
            }
        }
        Ok(AllowedAttempt {
            grade: grade.unwrap_or(Grade::Junior),
            kind: AttemptKind::Placement,
        })
    }

    fn check_step_down(
        &self,
        history: &[&Attempt],
        grade: Option<Grade>,
        now: i64,
    ) -> Result<AllowedAttempt, PolicyError> {
        // Allowed immediately after a failed placement/step-down that did not assign a category,
        // as long as there is a grade below the one just failed.
        let Some(last) = history.last() else {
            return Err(PolicyError::NotAllowed);
        };
        let is_placement_fail = matches!(
            last.kind,
            AttemptKind::Placement | AttemptKind::PlacementStepDown
        ) && !last.passed
            && !last.assigned;
        if !is_placement_fail || history.iter().any(|a| a.assigned) {
            return Err(PolicyError::NotAllowed);
        }
        let Some(lower) = last.grade.prev() else {
            // Failed junior → no lower grade; must wait out the retry window.
            return Err(PolicyError::RetryBlocked {
                available_at: last.at + self.config.retry_fail(),
            });
        };
        if let Some(g) = grade {
            if g != lower {
                return Err(PolicyError::NotAllowed);
            }
        }
        let _ = now;
        Ok(AllowedAttempt {
            grade: lower,
            kind: AttemptKind::PlacementStepDown,
        })
    }

    fn check_placement_upgrade(
        &self,
        stage: PlacementStage,
        history: &[&Attempt],
        now: i64,
    ) -> Result<AllowedAttempt, PolicyError> {
        let PlacementStage::Assigned { grade } = stage else {
            return Err(PolicyError::NotAllowed);
        };
        let Some(higher) = grade.next() else {
            return Err(PolicyError::NoAdjacentGrade);
        };
        // The immediate upgrade is offered once, right after the assigning attempt, and only if
        // that attempt was a confident placement pass. Any later attempt forecloses it.
        let Some(last) = history.last() else {
            return Err(PolicyError::NotAllowed);
        };
        let eligible = last.assigned
            && last.grade == grade
            && matches!(
                last.kind,
                AttemptKind::Placement | AttemptKind::PlacementStepDown
            );
        if !eligible {
            return Err(PolicyError::NotAllowed);
        }
        let _ = now;
        Ok(AllowedAttempt {
            grade: higher,
            kind: AttemptKind::PlacementUpgrade,
        })
    }

    fn check_change(
        &self,
        spec: Specialization,
        attempts: &[Attempt],
        stage: PlacementStage,
        step: fn(Grade) -> Option<Grade>,
        grade: Option<Grade>,
        now: i64,
    ) -> Result<AllowedAttempt, PolicyError> {
        let PlacementStage::Assigned { grade: current } = stage else {
            return Err(PolicyError::NotAllowed);
        };
        let Some(target) = step(current) else {
            return Err(PolicyError::NoAdjacentGrade);
        };
        if let Some(g) = grade {
            if g != target {
                return Err(PolicyError::NotAllowed);
            }
        }
        let kind = if target > current {
            AttemptKind::Upgrade
        } else {
            AttemptKind::Downgrade
        };
        // A failed upgrade never lowers the grade and blocks the next attempt for 30 days.
        if let Some(block) = self.failed_upgrade_block(spec, attempts, now) {
            return Err(PolicyError::RetryBlocked {
                available_at: block,
            });
        }
        // Grade changes are limited to once per cooldown from the last assignment.
        if let Some(assigned_at) = self.last_assignment_at(spec, attempts) {
            let available = assigned_at + self.config.cooldown();
            if now < available {
                return Err(PolicyError::ChangeCooldown {
                    available_at: available,
                });
            }
        }
        Ok(AllowedAttempt {
            grade: target,
            kind,
        })
    }

    fn check_retry(
        &self,
        stage: PlacementStage,
        history: &[&Attempt],
        grade: Option<Grade>,
        now: i64,
    ) -> Result<AllowedAttempt, PolicyError> {
        if !matches!(stage, PlacementStage::Unplaced) {
            return Err(PolicyError::NotAllowed);
        }
        let Some(block_until) = self.junior_fail_until(history) else {
            return Err(PolicyError::NotAllowed);
        };
        if now < block_until {
            return Err(PolicyError::RetryBlocked {
                available_at: block_until,
            });
        }
        Ok(AllowedAttempt {
            grade: grade.unwrap_or(Grade::Junior),
            kind: AttemptKind::Retry,
        })
    }

    /// If the last placement attempt failed at junior (no lower grade), returns when retry opens.
    fn junior_fail_until(&self, history: &[&Attempt]) -> Option<i64> {
        let last = history.last()?;
        let failed_junior = matches!(
            last.kind,
            AttemptKind::Placement | AttemptKind::PlacementStepDown | AttemptKind::Retry
        ) && !last.passed
            && !last.assigned
            && last.grade.prev().is_none();
        failed_junior.then(|| last.at + self.config.retry_fail())
    }

    /// If the most recent post-assignment upgrade failed, returns when the next attempt opens.
    fn failed_upgrade_block(
        &self,
        spec: Specialization,
        attempts: &[Attempt],
        now: i64,
    ) -> Option<i64> {
        let history = self.history(spec, attempts);
        let last_assigned_idx = history.iter().rposition(|a| a.assigned)?;
        let last = history.last()?;
        let failed_upgrade = history.len() - 1 > last_assigned_idx
            && matches!(
                last.kind,
                AttemptKind::Upgrade | AttemptKind::PlacementUpgrade
            )
            && !last.passed;
        if failed_upgrade {
            let available = last.at + self.config.retry_upgrade();
            (now < available).then_some(available)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog::Specialization::Backend;

    const T0: i64 = 1_700_000_000;

    fn attempt(grade: Grade, kind: AttemptKind, passed: bool, assigned: bool, at: i64) -> Attempt {
        Attempt {
            specialization: Backend,
            grade,
            kind,
            passed,
            assigned,
            at,
        }
    }

    fn policy() -> GradePolicy {
        GradePolicy::default()
    }

    #[test]
    fn failed_placement_cannot_be_retaken_immediately() {
        let p = policy();
        let day = 86_400;
        let history = [attempt(
            Grade::Middle,
            AttemptKind::Placement,
            false,
            false,
            T0,
        )];
        assert!(matches!(
            p.check(Backend, &history, AttemptKind::Placement, Grade::Middle, T0 + day),
            Err(PolicyError::RetryBlocked { available_at }) if available_at == T0 + 30 * day
        ));
        assert!(matches!(
            p.check(
                Backend,
                &history,
                AttemptKind::Placement,
                Grade::Senior,
                T0 + day
            ),
            Err(PolicyError::RetryBlocked { .. })
        ));
        let down = p
            .check(
                Backend,
                &history,
                AttemptKind::PlacementStepDown,
                Grade::Junior,
                T0 + day,
            )
            .unwrap();
        assert_eq!(down.grade, Grade::Junior);
        assert!(
            p.check(
                Backend,
                &history,
                AttemptKind::Placement,
                Grade::Middle,
                T0 + 31 * day
            )
            .is_ok()
        );
    }

    #[test]
    fn unplaced_can_take_placement() {
        let available = policy().available(Backend, &[], T0);
        assert!(available.iter().any(|a| a.kind == AttemptKind::Placement));
        assert!(
            policy()
                .check(Backend, &[], AttemptKind::Placement, Grade::Middle, T0)
                .is_ok()
        );
    }

    #[test]
    fn failed_placement_allows_immediate_step_down() {
        let history = [attempt(
            Grade::Middle,
            AttemptKind::Placement,
            false,
            false,
            T0,
        )];
        let step = policy()
            .check(
                Backend,
                &history,
                AttemptKind::PlacementStepDown,
                Grade::Junior,
                T0 + 60,
            )
            .unwrap();
        assert_eq!(step.grade, Grade::Junior);
        // Senior step-down after failing senior lands on middle.
        let history = [attempt(
            Grade::Senior,
            AttemptKind::Placement,
            false,
            false,
            T0,
        )];
        let step = policy()
            .check(
                Backend,
                &history,
                AttemptKind::PlacementStepDown,
                Grade::Middle,
                T0 + 60,
            )
            .unwrap();
        assert_eq!(step.grade, Grade::Middle);
    }

    #[test]
    fn failed_junior_blocks_for_30_days() {
        let history = [attempt(
            Grade::Junior,
            AttemptKind::Placement,
            false,
            false,
            T0,
        )];
        let err = policy()
            .check(
                Backend,
                &history,
                AttemptKind::PlacementStepDown,
                Grade::Junior,
                T0 + 60,
            )
            .unwrap_err();
        assert_eq!(
            err,
            PolicyError::RetryBlocked {
                available_at: T0 + 30 * DAY
            }
        );
        // Placement itself is blocked too until the window passes.
        assert!(matches!(
            policy().check(
                Backend,
                &history,
                AttemptKind::Placement,
                Grade::Junior,
                T0 + 60
            ),
            Err(PolicyError::RetryBlocked { .. })
        ));
        assert!(
            policy()
                .check(
                    Backend,
                    &history,
                    AttemptKind::Retry,
                    Grade::Junior,
                    T0 + 31 * DAY
                )
                .is_ok()
        );
        assert!(matches!(
            policy().check(Backend, &history, AttemptKind::Retry, Grade::Junior, T0 + 29 * DAY),
            Err(PolicyError::RetryBlocked { available_at }) if available_at == T0 + 30 * DAY
        ));
    }

    #[test]
    fn confident_pass_offers_one_upgrade() {
        let history = [attempt(
            Grade::Middle,
            AttemptKind::Placement,
            true,
            true,
            T0,
        )];
        let up = policy()
            .check(
                Backend,
                &history,
                AttemptKind::PlacementUpgrade,
                Grade::Senior,
                T0 + 60,
            )
            .unwrap();
        assert_eq!(up.grade, Grade::Senior);
        // Once another attempt happens, the immediate placement upgrade is gone.
        let history = [
            attempt(Grade::Middle, AttemptKind::Placement, true, true, T0),
            attempt(
                Grade::Senior,
                AttemptKind::PlacementUpgrade,
                false,
                false,
                T0 + 120,
            ),
        ];
        assert!(matches!(
            policy().check(
                Backend,
                &history,
                AttemptKind::PlacementUpgrade,
                Grade::Senior,
                T0 + 200
            ),
            Err(PolicyError::NotAllowed)
        ));
    }

    #[test]
    fn senior_pass_has_no_upgrade() {
        let history = [attempt(
            Grade::Senior,
            AttemptKind::Placement,
            true,
            true,
            T0,
        )];
        assert_eq!(
            policy().check(
                Backend,
                &history,
                AttemptKind::PlacementUpgrade,
                Grade::Senior,
                T0 + 60
            ),
            Err(PolicyError::NoAdjacentGrade)
        );
    }

    #[test]
    fn grade_change_respects_90_day_cooldown() {
        let history = [attempt(
            Grade::Middle,
            AttemptKind::Placement,
            true,
            true,
            T0,
        )];
        let err = policy()
            .check(
                Backend,
                &history,
                AttemptKind::Upgrade,
                Grade::Senior,
                T0 + 10 * DAY,
            )
            .unwrap_err();
        assert_eq!(
            err,
            PolicyError::ChangeCooldown {
                available_at: T0 + 90 * DAY
            }
        );
        assert!(
            policy()
                .check(
                    Backend,
                    &history,
                    AttemptKind::Upgrade,
                    Grade::Senior,
                    T0 + 90 * DAY
                )
                .is_ok()
        );
        assert!(
            policy()
                .check(
                    Backend,
                    &history,
                    AttemptKind::Downgrade,
                    Grade::Junior,
                    T0 + 91 * DAY
                )
                .is_ok()
        );
    }

    #[test]
    fn failed_upgrade_keeps_grade_and_blocks_30_days() {
        let history = [
            attempt(Grade::Middle, AttemptKind::Placement, true, true, T0),
            attempt(
                Grade::Senior,
                AttemptKind::Upgrade,
                false,
                false,
                T0 + 100 * DAY,
            ),
        ];
        let p = policy();
        assert_eq!(p.current_grade(Backend, &history), Some(Grade::Middle));
        let err = p
            .check(
                Backend,
                &history,
                AttemptKind::Upgrade,
                Grade::Senior,
                T0 + 110 * DAY,
            )
            .unwrap_err();
        assert_eq!(
            err,
            PolicyError::RetryBlocked {
                available_at: T0 + 130 * DAY
            }
        );
        assert!(
            p.check(
                Backend,
                &history,
                AttemptKind::Upgrade,
                Grade::Senior,
                T0 + 130 * DAY
            )
            .is_ok()
        );
    }

    #[test]
    fn successful_change_resets_cooldown() {
        let history = [
            attempt(Grade::Junior, AttemptKind::Placement, true, true, T0),
            attempt(
                Grade::Middle,
                AttemptKind::Upgrade,
                true,
                true,
                T0 + 100 * DAY,
            ),
        ];
        let p = policy();
        assert_eq!(p.current_grade(Backend, &history), Some(Grade::Middle));
        assert!(matches!(
            p.check(Backend, &history, AttemptKind::Upgrade, Grade::Senior, T0 + 150 * DAY),
            Err(PolicyError::ChangeCooldown { available_at }) if available_at == T0 + 190 * DAY
        ));
        assert!(
            p.check(
                Backend,
                &history,
                AttemptKind::Upgrade,
                Grade::Senior,
                T0 + 190 * DAY
            )
            .is_ok()
        );
    }

    #[test]
    fn configurable_intervals() {
        let p = GradePolicy::new(PolicyConfig {
            change_cooldown_days: 7,
            ..PolicyConfig::default()
        });
        let history = [attempt(
            Grade::Middle,
            AttemptKind::Placement,
            true,
            true,
            T0,
        )];
        assert!(
            p.check(
                Backend,
                &history,
                AttemptKind::Upgrade,
                Grade::Senior,
                T0 + 7 * DAY
            )
            .is_ok()
        );
    }

    #[test]
    fn other_specialization_history_is_ignored() {
        let history = [Attempt {
            specialization: catalog::Specialization::Ml,
            ..attempt(Grade::Middle, AttemptKind::Placement, true, true, T0)
        }];
        assert_eq!(policy().stage(Backend, &history), PlacementStage::Unplaced);
        assert!(
            policy()
                .check(Backend, &history, AttemptKind::Placement, Grade::Senior, T0)
                .is_ok()
        );
    }
}
