//! Grade-change policy (architecture §5.4, requirements T-5 / T-6).
//!
//! All timing is in unix seconds passed in as `now`; the crate never reads the clock. The
//! policy is a pure function of the candidate's assignment/attempt history and the current
//! time, returning which attempt kinds are allowed or why they are blocked.

use catalog::{Grade, Specialization};
use serde::{Deserialize, Serialize};

const DAY: i64 = 86_400;

/// A session left without an answer this long after the current item's deadline is abandoned:
/// it closes as `expired` and counts as an unsuccessful attempt at its grade.
pub const ABANDON_AFTER_SECS: i64 = 30 * 60;

/// Configurable intervals. Defaults follow architecture §5.4.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Minimum gap between grade changes after a category is assigned.
    pub change_cooldown_days: i64,
    /// Wait after an unsuccessful attempt before the same grade can be taken again.
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

/// How a session ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Passed,
    Failed,
    /// Left unfinished until it expired. Counts exactly like a failure: otherwise walking away
    /// from a test that goes badly would dodge every retry window.
    Abandoned,
    /// A moderator annulled the result. The attempt never assigns, and `at` is the moment of the
    /// decision rather than of the test: it blocks its grade like a failure counted from the
    /// decision, so a slow review cannot hand back an immediate retake, and the category falls
    /// back to the previous assignment with that assignment's own cooldown.
    Annulled,
    /// Voided like [`Outcome::Annulled`], with a retest ordered under the camera: the same attempt
    /// may be taken again right away while nothing else has happened in the specialization since
    /// the decision ([`GradePolicy::ordered_retest`]).
    RetestOrdered,
}

/// What a single session attempted and how it ended.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Attempt {
    pub specialization: Specialization,
    pub grade: Grade,
    pub kind: AttemptKind,
    pub outcome: Outcome,
    /// Whether this attempt changed the candidate's active category.
    pub assigned: bool,
    /// When it ended: the last answer, the moment an abandoned session expired, or the
    /// moderator's decision that voided the result.
    pub at: i64,
}

impl Attempt {
    fn unsuccessful(&self) -> bool {
        self.outcome != Outcome::Passed
    }
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
    /// Upgrade test offered after a confident pass (one per placement, T-5).
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

impl PolicyError {
    /// When the attempt opens, for the two time-based blocks.
    pub const fn available_at(self) -> Option<i64> {
        match self {
            PolicyError::ChangeCooldown { available_at }
            | PolicyError::RetryBlocked { available_at } => Some(available_at),
            PolicyError::NotAllowed | PolicyError::NoAdjacentGrade => None,
        }
    }
}

/// The latest of the windows an attempt has to wait out. Every rule that applies contributes
/// one, so the date shown is the one when the attempt really opens, not the first rule hit.
#[derive(Debug, Clone, Copy, Default)]
struct Wait(Option<PolicyError>);

impl Wait {
    fn retry(&mut self, until: i64) {
        self.push(PolicyError::RetryBlocked {
            available_at: until,
        });
    }

    fn cooldown(&mut self, until: i64) {
        self.push(PolicyError::ChangeCooldown {
            available_at: until,
        });
    }

    fn push(&mut self, e: PolicyError) {
        let later = match (self.0.and_then(PolicyError::available_at), e.available_at()) {
            (Some(current), Some(new)) => new > current,
            _ => true,
        };
        if later {
            self.0 = Some(e);
        }
    }

    fn check(self, allowed: AllowedAttempt, now: i64) -> Result<AllowedAttempt, PolicyError> {
        match self.0 {
            Some(e) if e.available_at().is_some_and(|at| now < at) => Err(e),
            _ => Ok(allowed),
        }
    }
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

    /// The retest a moderator ordered: the voided attempt once more, same grade and kind, open at
    /// once and outside every window while it is still the latest attempt in the specialization.
    /// Taking any other test first gives it up, and the voided attempt then counts like an
    /// annulled one.
    pub fn ordered_retest(
        &self,
        spec: Specialization,
        attempts: &[Attempt],
    ) -> Option<AllowedAttempt> {
        let last = *self.history(spec, attempts).last()?;
        (last.outcome == Outcome::RetestOrdered).then_some(AllowedAttempt {
            grade: last.grade,
            kind: last.kind,
        })
    }

    fn check_kind(
        &self,
        spec: Specialization,
        attempts: &[Attempt],
        kind: AttemptKind,
        grade: Option<Grade>,
        now: i64,
    ) -> Result<AllowedAttempt, PolicyError> {
        if let Some(retest) = self.ordered_retest(spec, attempts)
            && retest.kind == kind
            && grade.is_none_or(|g| g == retest.grade)
        {
            return Ok(retest);
        }
        let history = self.history(spec, attempts);
        let stage = self.stage(spec, attempts);
        let mut wait = Wait::default();
        let allowed = match kind {
            AttemptKind::Placement => self.check_placement(&history, grade, &mut wait)?,
            AttemptKind::PlacementStepDown => self.check_step_down(&history, grade)?,
            AttemptKind::PlacementUpgrade => {
                self.check_placement_upgrade(stage, &history, grade)?
            }
            AttemptKind::Upgrade => {
                self.check_change(spec, attempts, stage, Grade::next, grade, &mut wait)?
            }
            AttemptKind::Downgrade => {
                self.check_change(spec, attempts, stage, Grade::prev, grade, &mut wait)?
            }
            AttemptKind::Retry => self.check_retry(stage, &history, grade, &mut wait)?,
        };
        // Whatever the kind, a grade the candidate failed or abandoned waits out the retry
        // window: an immediate up-test after a confident pass one grade lower would otherwise
        // retake that very grade the same day.
        for a in history
            .iter()
            .filter(|a| a.grade == allowed.grade && a.unsuccessful())
        {
            wait.retry(a.at + self.config.retry_fail());
        }
        wait.check(allowed, now)
    }

    fn check_placement(
        &self,
        history: &[&Attempt],
        grade: Option<Grade>,
        wait: &mut Wait,
    ) -> Result<AllowedAttempt, PolicyError> {
        if history.iter().any(|a| a.assigned) {
            return Err(PolicyError::NotAllowed);
        }
        // After a failed attempt the only immediate option is one grade lower (step-down);
        // retaking the same or a higher grade waits out the retry window, otherwise the test
        // could be brute-forced by repeating it until the items happen to be easy.
        if let Some(last_fail) = history.iter().rev().find(|a| a.unsuccessful()) {
            wait.retry(last_fail.at + self.config.retry_fail());
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
    ) -> Result<AllowedAttempt, PolicyError> {
        // Allowed immediately after a failed placement/step-down that did not assign a category,
        // as long as there is a grade below the one just failed.
        let Some(last) = history.last() else {
            return Err(PolicyError::NotAllowed);
        };
        let is_placement_fail = matches!(
            last.kind,
            AttemptKind::Placement | AttemptKind::PlacementStepDown
        ) && last.unsuccessful()
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
        if grade.is_some_and(|g| g != lower) {
            return Err(PolicyError::NotAllowed);
        }
        Ok(AllowedAttempt {
            grade: lower,
            kind: AttemptKind::PlacementStepDown,
        })
    }

    fn check_placement_upgrade(
        &self,
        stage: PlacementStage,
        history: &[&Attempt],
        grade: Option<Grade>,
    ) -> Result<AllowedAttempt, PolicyError> {
        let PlacementStage::Assigned { grade: current } = stage else {
            return Err(PolicyError::NotAllowed);
        };
        let Some(higher) = current.next() else {
            return Err(PolicyError::NoAdjacentGrade);
        };
        if grade.is_some_and(|g| g != higher) {
            return Err(PolicyError::NotAllowed);
        }
        // The up-test is offered once, after the assigning attempt, and only if that attempt
        // was a confident placement pass. Any later attempt forecloses it. A recent failure at
        // the higher grade postpones it (see `check_kind`) but does not cancel it.
        let Some(last) = history.last() else {
            return Err(PolicyError::NotAllowed);
        };
        let eligible = last.assigned
            && last.grade == current
            && matches!(
                last.kind,
                AttemptKind::Placement | AttemptKind::PlacementStepDown
            );
        if !eligible {
            return Err(PolicyError::NotAllowed);
        }
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
        wait: &mut Wait,
    ) -> Result<AllowedAttempt, PolicyError> {
        let PlacementStage::Assigned { grade: current } = stage else {
            return Err(PolicyError::NotAllowed);
        };
        let Some(target) = step(current) else {
            return Err(PolicyError::NoAdjacentGrade);
        };
        if grade.is_some_and(|g| g != target) {
            return Err(PolicyError::NotAllowed);
        }
        let kind = if target > current {
            AttemptKind::Upgrade
        } else {
            AttemptKind::Downgrade
        };
        // A failed upgrade never lowers the grade and blocks the next attempt for 30 days.
        if let Some(until) = self.failed_upgrade_until(spec, attempts) {
            wait.retry(until);
        }
        // Grade changes are limited to once per cooldown from the last assignment.
        if let Some(assigned_at) = self.last_assignment_at(spec, attempts) {
            wait.cooldown(assigned_at + self.config.cooldown());
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
        wait: &mut Wait,
    ) -> Result<AllowedAttempt, PolicyError> {
        if !matches!(stage, PlacementStage::Unplaced) {
            return Err(PolicyError::NotAllowed);
        }
        let Some(block_until) = self.junior_fail_until(history) else {
            return Err(PolicyError::NotAllowed);
        };
        wait.retry(block_until);
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
        ) && last.unsuccessful()
            && !last.assigned
            && last.grade.prev().is_none();
        failed_junior.then(|| last.at + self.config.retry_fail())
    }

    /// If the most recent post-assignment attempt is a failed upgrade, when the next one opens.
    fn failed_upgrade_until(&self, spec: Specialization, attempts: &[Attempt]) -> Option<i64> {
        let history = self.history(spec, attempts);
        let last_assigned_idx = history.iter().rposition(|a| a.assigned)?;
        let last = history.last()?;
        let failed_upgrade = history.len() - 1 > last_assigned_idx
            && matches!(
                last.kind,
                AttemptKind::Upgrade | AttemptKind::PlacementUpgrade
            )
            && last.unsuccessful();
        failed_upgrade.then(|| last.at + self.config.retry_upgrade())
    }
}

/// A retest a moderator ordered under the camera ([`Outcome::RetestOrdered`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraOrder {
    pub specialization: Specialization,
    pub grade: Grade,
    /// The decision, unix seconds.
    pub at: i64,
}

/// A test as far as the camera order cares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraTest {
    pub specialization: Specialization,
    pub grade: Grade,
    pub started: i64,
    pub camera: bool,
    /// Ended passed or failed. An abandoned test settles nothing.
    pub completed: bool,
}

/// Whether a test at `grade` must be taken with the camera. An order stands until a test at its
/// grade or above, started after the order with the camera, ends with a result: abandoning that
/// test and returning after the retry window must not lift it, and aiming a grade higher must not
/// dodge it. A lower grade is a weaker claim and stays free.
pub fn camera_required(
    orders: &[CameraOrder],
    tests: &[CameraTest],
    spec: Specialization,
    grade: Grade,
) -> bool {
    orders
        .iter()
        .filter(|o| o.specialization == spec && grade >= o.grade)
        .any(|o| {
            !tests.iter().any(|t| {
                t.specialization == spec
                    && t.grade >= o.grade
                    && t.started >= o.at
                    && t.camera
                    && t.completed
            })
        })
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
            outcome: if passed {
                Outcome::Passed
            } else {
                Outcome::Failed
            },
            assigned,
            at,
        }
    }

    fn abandoned(grade: Grade, kind: AttemptKind, at: i64) -> Attempt {
        Attempt {
            outcome: Outcome::Abandoned,
            ..attempt(grade, kind, false, false, at)
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
    fn placement_upgrade_goes_one_grade_up_only() {
        let history = [attempt(
            Grade::Junior,
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
            Err(PolicyError::NotAllowed)
        );
        let up = policy()
            .check(
                Backend,
                &history,
                AttemptKind::PlacementUpgrade,
                Grade::Middle,
                T0 + 60,
            )
            .unwrap();
        assert_eq!(up.grade, Grade::Middle);
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

    /// A failed up-test right after placement: 30 days, but not before the 90-day window from
    /// the assignment ends — the date names the window that really opens last.
    #[test]
    fn failed_placement_upgrade_waits_for_the_change_cooldown() {
        let p = policy();
        for up in [
            attempt(
                Grade::Senior,
                AttemptKind::PlacementUpgrade,
                false,
                false,
                T0 + 3600,
            ),
            abandoned(Grade::Senior, AttemptKind::PlacementUpgrade, T0 + 3600),
        ] {
            let history = [
                attempt(Grade::Middle, AttemptKind::Placement, true, true, T0),
                up,
            ];
            assert_eq!(
                p.check(
                    Backend,
                    &history,
                    AttemptKind::PlacementUpgrade,
                    Grade::Senior,
                    T0 + 31 * DAY
                ),
                Err(PolicyError::NotAllowed),
                "the one up-test is used up"
            );
            for at in [T0 + DAY, T0 + 40 * DAY] {
                assert_eq!(
                    p.check(Backend, &history, AttemptKind::Upgrade, Grade::Senior, at),
                    Err(PolicyError::ChangeCooldown {
                        available_at: T0 + 90 * DAY
                    })
                );
            }
            assert!(
                p.check(
                    Backend,
                    &history,
                    AttemptKind::Upgrade,
                    Grade::Senior,
                    T0 + 90 * DAY
                )
                .is_ok()
            );
        }
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

    /// Hole (a): walking away from a placement must not reset the clock.
    #[test]
    fn abandoned_placement_counts_as_a_failure() {
        let p = policy();
        let history = [abandoned(Grade::Middle, AttemptKind::Placement, T0)];
        for grade in [Grade::Middle, Grade::Senior] {
            assert_eq!(
                p.check(Backend, &history, AttemptKind::Placement, grade, T0 + DAY),
                Err(PolicyError::RetryBlocked {
                    available_at: T0 + 30 * DAY
                })
            );
        }
        let down = p
            .check(
                Backend,
                &history,
                AttemptKind::PlacementStepDown,
                Grade::Junior,
                T0 + 60,
            )
            .unwrap();
        assert_eq!(down.grade, Grade::Junior);
        assert!(
            p.check(
                Backend,
                &history,
                AttemptKind::Placement,
                Grade::Middle,
                T0 + 30 * DAY
            )
            .is_ok()
        );
    }

    #[test]
    fn abandoned_junior_blocks_like_a_failed_one() {
        let p = policy();
        let history = [abandoned(Grade::Junior, AttemptKind::Placement, T0)];
        for kind in [
            AttemptKind::Placement,
            AttemptKind::PlacementStepDown,
            AttemptKind::Retry,
        ] {
            assert_eq!(
                p.check(Backend, &history, kind, Grade::Junior, T0 + DAY),
                Err(PolicyError::RetryBlocked {
                    available_at: T0 + 30 * DAY
                }),
                "{kind:?}"
            );
        }
        assert!(
            p.check(
                Backend,
                &history,
                AttemptKind::Retry,
                Grade::Junior,
                T0 + 30 * DAY
            )
            .is_ok()
        );
    }

    #[test]
    fn abandoned_upgrade_counts_as_a_failed_upgrade() {
        let p = policy();
        let history = [
            attempt(Grade::Middle, AttemptKind::Placement, true, true, T0),
            abandoned(Grade::Senior, AttemptKind::Upgrade, T0 + 100 * DAY),
        ];
        assert_eq!(p.current_grade(Backend, &history), Some(Grade::Middle));
        assert_eq!(
            p.check(
                Backend,
                &history,
                AttemptKind::Upgrade,
                Grade::Senior,
                T0 + 101 * DAY
            ),
            Err(PolicyError::RetryBlocked {
                available_at: T0 + 130 * DAY
            })
        );
        assert_eq!(
            p.check(
                Backend,
                &history,
                AttemptKind::Downgrade,
                Grade::Junior,
                T0 + 101 * DAY
            ),
            Err(PolicyError::RetryBlocked {
                available_at: T0 + 130 * DAY
            })
        );
    }

    /// Hole (b): failed Middle, then a confident Junior pass. The up-test to Middle is still
    /// offered once, but only after the 30 days since the Middle attempt.
    #[test]
    fn up_test_waits_for_a_recent_failure_at_that_grade() {
        let p = policy();
        for first in [
            attempt(Grade::Middle, AttemptKind::Placement, false, false, T0),
            abandoned(Grade::Middle, AttemptKind::Placement, T0),
        ] {
            let history = [
                first,
                attempt(
                    Grade::Junior,
                    AttemptKind::PlacementStepDown,
                    true,
                    true,
                    T0 + 3600,
                ),
            ];
            assert_eq!(
                p.check(
                    Backend,
                    &history,
                    AttemptKind::PlacementUpgrade,
                    Grade::Middle,
                    T0 + 2 * 3600
                ),
                Err(PolicyError::RetryBlocked {
                    available_at: T0 + 30 * DAY
                })
            );
            let up = p
                .check(
                    Backend,
                    &history,
                    AttemptKind::PlacementUpgrade,
                    Grade::Middle,
                    T0 + 30 * DAY,
                )
                .unwrap();
            assert_eq!(up.grade, Grade::Middle);
            // The regular upgrade path is still governed by the 90-day window.
            assert!(matches!(
                p.check(
                    Backend,
                    &history,
                    AttemptKind::Upgrade,
                    Grade::Middle,
                    T0 + 30 * DAY
                ),
                Err(PolicyError::ChangeCooldown { .. })
            ));
        }
    }

    #[test]
    fn up_test_is_immediate_without_a_recent_failure_at_that_grade() {
        let p = policy();
        let history = [
            attempt(Grade::Middle, AttemptKind::Placement, false, false, T0),
            attempt(
                Grade::Junior,
                AttemptKind::PlacementStepDown,
                true,
                true,
                T0 + 31 * DAY,
            ),
        ];
        let up = p
            .check(
                Backend,
                &history,
                AttemptKind::PlacementUpgrade,
                Grade::Middle,
                T0 + 31 * DAY + 60,
            )
            .unwrap();
        assert_eq!(up.grade, Grade::Middle);
    }

    #[test]
    fn up_test_waits_for_the_latest_failure_at_that_grade() {
        let p = policy();
        let history = [
            attempt(Grade::Senior, AttemptKind::Placement, false, false, T0),
            attempt(
                Grade::Middle,
                AttemptKind::PlacementStepDown,
                false,
                false,
                T0 + 3600,
            ),
            attempt(
                Grade::Junior,
                AttemptKind::PlacementStepDown,
                true,
                true,
                T0 + 2 * 3600,
            ),
        ];
        assert_eq!(
            p.check(
                Backend,
                &history,
                AttemptKind::PlacementUpgrade,
                Grade::Middle,
                T0 + 3 * 3600
            ),
            Err(PolicyError::RetryBlocked {
                available_at: T0 + 30 * DAY + 3600
            })
        );
    }

    fn voided(grade: Grade, kind: AttemptKind, outcome: Outcome, decided: i64) -> Attempt {
        Attempt {
            outcome,
            ..attempt(grade, kind, true, false, decided)
        }
    }

    #[test]
    fn annulled_placement_leaves_the_candidate_unplaced() {
        let p = policy();
        let decided = T0 + 2 * DAY;
        let history = [voided(
            Grade::Middle,
            AttemptKind::Placement,
            Outcome::Annulled,
            decided,
        )];
        assert_eq!(p.current_grade(Backend, &history), None);
        for grade in [Grade::Middle, Grade::Senior] {
            assert_eq!(
                p.check(
                    Backend,
                    &history,
                    AttemptKind::Placement,
                    grade,
                    decided + DAY
                ),
                Err(PolicyError::RetryBlocked {
                    available_at: decided + 30 * DAY
                })
            );
        }
        let down = p
            .check(
                Backend,
                &history,
                AttemptKind::PlacementStepDown,
                Grade::Junior,
                decided + 60,
            )
            .unwrap();
        assert_eq!(down.grade, Grade::Junior);
        assert_eq!(p.ordered_retest(Backend, &history), None);
    }

    /// The window runs from the decision: a review that took 40 days does not hand back an
    /// immediate retake.
    #[test]
    fn annulment_blocks_its_grade_from_the_decision() {
        let p = policy();
        let decided = T0 + 40 * DAY;
        let history = [voided(
            Grade::Middle,
            AttemptKind::Placement,
            Outcome::Annulled,
            decided,
        )];
        assert_eq!(
            p.check(
                Backend,
                &history,
                AttemptKind::Placement,
                Grade::Middle,
                T0 + 41 * DAY
            ),
            Err(PolicyError::RetryBlocked {
                available_at: T0 + 70 * DAY
            })
        );
        assert!(
            p.check(
                Backend,
                &history,
                AttemptKind::Placement,
                Grade::Middle,
                T0 + 70 * DAY
            )
            .is_ok()
        );
    }

    /// Junior since T0, Middle by an upgrade at T0 + 100 days, annulled at T0 + 101 days: Junior is
    /// current again and its own 90-day window (long over) governs changes, not the window of
    /// the voided Middle assignment, which would have run to T0 + 190 days.
    #[test]
    fn annulled_upgrade_restores_the_previous_grade_and_its_cooldown() {
        let p = policy();
        let decided = T0 + 101 * DAY;
        let history = [
            attempt(Grade::Junior, AttemptKind::Placement, true, true, T0),
            voided(
                Grade::Middle,
                AttemptKind::Upgrade,
                Outcome::Annulled,
                decided,
            ),
        ];
        assert_eq!(p.current_grade(Backend, &history), Some(Grade::Junior));
        assert_eq!(
            p.check(
                Backend,
                &history,
                AttemptKind::Upgrade,
                Grade::Middle,
                decided + DAY
            ),
            Err(PolicyError::RetryBlocked {
                available_at: decided + 30 * DAY
            })
        );
        assert!(
            p.check(
                Backend,
                &history,
                AttemptKind::Upgrade,
                Grade::Middle,
                decided + 30 * DAY
            )
            .is_ok()
        );
        assert!(decided + 30 * DAY < T0 + 190 * DAY);
    }

    /// A voided assignment that a later one already replaced: the current grade stays.
    #[test]
    fn annulling_a_superseded_assignment_keeps_the_current_grade() {
        let p = policy();
        let history = [
            attempt(Grade::Junior, AttemptKind::Placement, true, true, T0),
            attempt(
                Grade::Senior,
                AttemptKind::Upgrade,
                true,
                true,
                T0 + 200 * DAY,
            ),
            voided(
                Grade::Middle,
                AttemptKind::Upgrade,
                Outcome::Annulled,
                T0 + 201 * DAY,
            ),
        ];
        assert_eq!(p.current_grade(Backend, &history), Some(Grade::Senior));
        assert_eq!(
            p.check(
                Backend,
                &history,
                AttemptKind::Downgrade,
                Grade::Middle,
                T0 + 300 * DAY
            )
            .map(|a| a.grade),
            Ok(Grade::Middle)
        );
    }

    #[test]
    fn ordered_retest_is_open_right_away() {
        let p = policy();
        let decided = T0 + 3 * DAY;
        let history = [voided(
            Grade::Middle,
            AttemptKind::Placement,
            Outcome::RetestOrdered,
            decided,
        )];
        assert_eq!(
            p.ordered_retest(Backend, &history),
            Some(AllowedAttempt {
                grade: Grade::Middle,
                kind: AttemptKind::Placement
            })
        );
        assert_eq!(
            p.check(
                Backend,
                &history,
                AttemptKind::Placement,
                Grade::Middle,
                decided + 60
            )
            .map(|a| a.grade),
            Ok(Grade::Middle)
        );
        assert_eq!(
            p.check(
                Backend,
                &history,
                AttemptKind::Placement,
                Grade::Senior,
                decided + 60
            ),
            Err(PolicyError::RetryBlocked {
                available_at: decided + 30 * DAY
            }),
            "the retest does not open other grades"
        );

        let upgrade = [
            attempt(Grade::Junior, AttemptKind::Placement, true, true, T0),
            voided(
                Grade::Middle,
                AttemptKind::Upgrade,
                Outcome::RetestOrdered,
                T0 + 101 * DAY,
            ),
        ];
        assert_eq!(p.current_grade(Backend, &upgrade), Some(Grade::Junior));
        assert!(
            p.check(
                Backend,
                &upgrade,
                AttemptKind::Upgrade,
                Grade::Middle,
                T0 + 101 * DAY + 60
            )
            .is_ok()
        );
    }

    #[test]
    fn another_test_first_gives_up_the_ordered_retest() {
        let p = policy();
        let decided = T0 + 3 * DAY;
        let history = [
            voided(
                Grade::Middle,
                AttemptKind::Placement,
                Outcome::RetestOrdered,
                decided,
            ),
            attempt(
                Grade::Junior,
                AttemptKind::PlacementStepDown,
                true,
                true,
                decided + 3600,
            ),
        ];
        assert_eq!(p.ordered_retest(Backend, &history), None);
        assert_eq!(p.current_grade(Backend, &history), Some(Grade::Junior));
        assert_eq!(
            p.check(
                Backend,
                &history,
                AttemptKind::PlacementUpgrade,
                Grade::Middle,
                decided + 2 * 3600
            ),
            Err(PolicyError::RetryBlocked {
                available_at: decided + 30 * DAY
            })
        );
    }

    #[test]
    fn camera_order_covers_its_grade_and_above_until_a_completed_camera_test() {
        let order = [CameraOrder {
            specialization: Backend,
            grade: Grade::Middle,
            at: T0,
        }];
        let test = |grade, started, camera, completed| CameraTest {
            specialization: Backend,
            grade,
            started,
            camera,
            completed,
        };
        assert!(!camera_required(&order, &[], Backend, Grade::Junior));
        assert!(camera_required(&order, &[], Backend, Grade::Middle));
        assert!(camera_required(&order, &[], Backend, Grade::Senior));
        assert!(!camera_required(
            &order,
            &[],
            catalog::Specialization::Qa,
            Grade::Senior
        ));
        for unsettled in [
            test(Grade::Middle, T0 + 60, true, false),
            test(Grade::Middle, T0 + 60, false, true),
            test(Grade::Middle, T0 - 60, true, true),
            test(Grade::Junior, T0 + 60, true, true),
        ] {
            assert!(
                camera_required(&order, &[unsettled], Backend, Grade::Middle),
                "{unsettled:?}"
            );
        }
        for settled in [
            test(Grade::Middle, T0 + 60, true, true),
            test(Grade::Senior, T0 + 60, true, true),
        ] {
            assert!(!camera_required(&order, &[settled], Backend, Grade::Middle));
        }
    }

    #[test]
    fn step_down_chain_stays_immediate_after_abandoning() {
        let p = policy();
        let history = [
            attempt(Grade::Senior, AttemptKind::Placement, false, false, T0),
            abandoned(Grade::Middle, AttemptKind::PlacementStepDown, T0 + 3600),
        ];
        let down = p
            .check(
                Backend,
                &history,
                AttemptKind::PlacementStepDown,
                Grade::Junior,
                T0 + 2 * 3600,
            )
            .unwrap();
        assert_eq!(down.grade, Grade::Junior);
    }
}
