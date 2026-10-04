//! Computerized adaptive testing (architecture §5.3).
//!
//! Each step re-estimates ability θ with the EAP posterior, then picks the next item that is
//! both informative at the current θ and needed for content balance, with randomesque
//! exposure control. The test is a *classification* test against the claimed grade's cut: it
//! stops as soon as the 95% credible interval lies wholly on one side of that cut (the grade
//! is decided), or at the item-count bounds.

use catalog::{Grade, Specialization};
use rand::seq::IndexedRandom;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::irt::{self, ItemParams, Posterior};
use crate::items::{self, Bank, ItemRef, ItemView, Slot, Topic, bank};

/// Tunables for the adaptive algorithm. Defaults follow architecture §5.3.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CatConfig {
    pub min_items: u16,
    pub max_items: u16,
    /// Stop when `P(θ below cut)` or `P(θ above cut)` reaches this (the 95% rule → 0.975 one-sided).
    pub classify_confidence: f64,
    /// Credence for "confidently above the next grade's cut" → offer an upgrade test.
    pub upgrade_confidence: f64,
    /// Pick the next item uniformly among this many most-informative candidates (randomesque).
    pub exposure_top: usize,
    /// Below this fraction of a topic's target quota, the topic is prioritized.
    pub balance_weight: f64,
    /// Person-fit `l_z` at or below this flags an aberrant response pattern.
    pub aberrant_lz: f64,
    /// Multiplicative selection boost for templates overlapping `focus_skills`.
    pub focus_boost: f32,
}

impl Default for CatConfig {
    fn default() -> Self {
        Self {
            min_items: 8,
            max_items: 20,
            classify_confidence: 0.975,
            upgrade_confidence: 0.9,
            exposure_top: 5,
            balance_weight: 1.0,
            aberrant_lz: -2.0,
            focus_boost: 1.5,
        }
    }
}

/// One administered item with its outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdministeredItem {
    pub item: ItemRef,
    pub correct: bool,
    pub elapsed_ms: u32,
    pub topic: Topic,
    pub params: ItemParams,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PosteriorSummary {
    pub theta: f64,
    pub se: f64,
    pub ci95_low: f64,
    pub ci95_high: f64,
    pub prob_above_cut: f64,
}

/// Serializable session (stored by the API as JSONB). Everything needed to pick the next item
/// and to replay the posterior lives here; the posterior itself is recomputed from responses so
/// state stays small and tamper-evident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionState {
    pub specialization: Specialization,
    pub claimed_grade: Grade,
    pub seed: u64,
    #[serde(default)]
    pub focus_skills: Vec<String>,
    #[serde(default)]
    pub administered: Vec<AdministeredItem>,
    pub summary: PosteriorSummary,
    #[serde(default = "CatConfig::default")]
    pub config: CatConfig,
}

/// Final classification once the session stops.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Decision {
    pub passed: bool,
    pub theta: f64,
    pub se: f64,
    /// Grade band θ̂ falls into (`None` below the junior cut).
    pub grade_measured: Option<Grade>,
    /// `P(θ ≥ next grade cut) ≥ upgrade_confidence`.
    pub confident_above_next: bool,
    pub person_fit_lz: f64,
    /// `l_z < aberrant_lz`: inconsistent pattern, result needs review.
    pub aberrant: bool,
    pub items_administered: u16,
}

impl SessionState {
    fn prior_mean(spec: Specialization, grade: Grade) -> f64 {
        let _ = spec;
        grade.theta_cut()
    }

    fn rebuild_posterior(&self) -> Posterior {
        let mut post = Posterior::new(Self::prior_mean(self.specialization, self.claimed_grade));
        for a in &self.administered {
            post.update(&a.params, a.correct);
        }
        post
    }

    fn refresh_summary(&mut self, post: &Posterior) {
        let (lo, hi) = post.credible_interval(0.95);
        self.summary = PosteriorSummary {
            theta: post.mean(),
            se: post.sd(),
            ci95_low: lo,
            ci95_high: hi,
            prob_above_cut: post.prob_above(self.claimed_grade.theta_cut()),
        };
    }
}

/// Starts a session: prior centred on the claimed grade's cut.
pub fn start(
    spec: Specialization,
    claimed_grade: Grade,
    seed: u64,
    focus_skills: Vec<String>,
) -> SessionState {
    start_with(
        spec,
        claimed_grade,
        seed,
        focus_skills,
        CatConfig::default(),
    )
}

pub fn start_with(
    spec: Specialization,
    claimed_grade: Grade,
    seed: u64,
    focus_skills: Vec<String>,
    config: CatConfig,
) -> SessionState {
    let mut state = SessionState {
        specialization: spec,
        claimed_grade,
        seed,
        focus_skills,
        administered: Vec::new(),
        summary: PosteriorSummary {
            theta: claimed_grade.theta_cut(),
            se: 1.0,
            ci95_low: claimed_grade.theta_cut() - 1.96,
            ci95_high: claimed_grade.theta_cut() + 1.96,
            prob_above_cut: 0.5,
        },
        config,
    };
    let post = state.rebuild_posterior();
    state.refresh_summary(&post);
    state
}

/// The specialization's balancing topics and their target share (counts of administered items).
fn topic_targets(bank: &Bank, spec: Specialization) -> Vec<Topic> {
    let mut topics: Vec<Topic> = Vec::new();
    for t in bank.for_specialization(spec) {
        if !topics.contains(&t.topic) {
            topics.push(t.topic);
        }
    }
    topics
}

/// Deterministic per-step RNG: session seed mixed with the step index.
fn step_rng(seed: u64, step: usize) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed ^ (0x9E37_79B9_7F4A_7C15u64.wrapping_mul(step as u64 + 1)))
}

/// Number of distinct templates that can still be administered for this specialization.
fn remaining_templates(bank: &Bank, state: &SessionState) -> usize {
    let used = state.administered.len();
    bank.for_specialization(state.specialization)
        .count()
        .saturating_sub(used)
}

/// Selects the next item or `None` if the session should not continue.
pub fn next_item(state: &SessionState) -> Option<ItemRef> {
    if decision(state).is_some() {
        return None;
    }
    select(bank(), state)
}

fn select(bank: &Bank, state: &SessionState) -> Option<ItemRef> {
    let theta = state.summary.theta;
    let step = state.administered.len();
    let focus_mask = items::skill_mask(state.focus_skills.iter());

    let used_templates: Vec<u16> = state
        .administered
        .iter()
        .filter_map(|a| bank.template_index(&a.item.template_id))
        .collect();
    let used_seeds: Vec<(u16, u64)> = state
        .administered
        .iter()
        .filter_map(|a| {
            bank.template_index(&a.item.template_id)
                .map(|t| (t, a.item.seed))
        })
        .collect();

    let topics = topic_targets(bank, state.specialization);
    let per_topic_target = state.config.max_items as f64 / topics.len().max(1) as f64;
    let topic_counts = |topic: Topic| {
        state
            .administered
            .iter()
            .filter(|a| a.topic == topic)
            .count() as f64
    };

    let mut scored: Vec<(f32, Slot)> = Vec::new();
    for &slot in bank.slots(state.specialization) {
        let template_idx = bank.hot.template[slot as usize];
        if used_templates.contains(&template_idx) {
            continue;
        }
        let params = bank.slot_params(slot);
        let info = irt::information(theta, &params) as f32;
        let topic = bank.template_topic[template_idx as usize];
        let deficit = (per_topic_target - topic_counts(topic)).max(0.0) / per_topic_target.max(1.0);
        let balance = 1.0 + state.config.balance_weight as f32 * deficit as f32;
        let focus =
            if focus_mask != 0 && bank.template_skills[template_idx as usize] & focus_mask != 0 {
                state.config.focus_boost
            } else {
                1.0
            };
        scored.push((info * balance * focus, slot));
    }
    if scored.is_empty() {
        return None;
    }

    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let top = scored.len().min(state.config.exposure_top.max(1));
    let mut rng = step_rng(state.seed, step);
    let &(_, slot) = scored[..top].choose(&mut rng).unwrap_or(&scored[0]);

    let template_idx = bank.hot.template[slot as usize];
    let template = bank.template_at(template_idx);
    let level = bank.hot.level[slot as usize];
    let seed = loop {
        let candidate = rng.next_u64();
        if !used_seeds.contains(&(template_idx, candidate)) {
            break candidate;
        }
    };
    Some(ItemRef {
        template_id: template.id.to_owned(),
        level,
        seed,
    })
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RecordError {
    #[error("item `{0}` is not a known template")]
    UnknownItem(String),
    #[error("item `{template}` was already administered in this session")]
    DuplicateTemplate { template: String },
    #[error("session already has {max} items")]
    SessionFull { max: u16 },
}

/// Records an outcome and advances the posterior. The caller passes `correct = false` for
/// timeouts (architecture §5.2). Idempotent per `ItemRef` is *not* assumed; the server should
/// only record the item it issued via `next_item`.
pub fn record(
    state: &mut SessionState,
    item: ItemRef,
    correct: bool,
    elapsed_ms: u32,
) -> Result<(), RecordError> {
    if state.administered.len() >= state.config.max_items as usize {
        return Err(RecordError::SessionFull {
            max: state.config.max_items,
        });
    }
    let bank = bank();
    let template = bank
        .template(&item.template_id)
        .ok_or_else(|| RecordError::UnknownItem(item.template_id.clone()))?;
    if state
        .administered
        .iter()
        .any(|a| a.item.template_id == item.template_id)
    {
        return Err(RecordError::DuplicateTemplate {
            template: item.template_id,
        });
    }
    let params = template
        .level(item.level)
        .ok_or_else(|| RecordError::UnknownItem(item.template_id.clone()))?
        .params;
    state.administered.push(AdministeredItem {
        item,
        correct,
        elapsed_ms,
        topic: template.topic,
        params,
    });
    let post = state.rebuild_posterior();
    state.refresh_summary(&post);
    Ok(())
}

/// `Some` once the session should stop: either the grade is classified with 95% confidence and
/// at least `min_items` have been shown, or `max_items` is reached.
pub fn decision(state: &SessionState) -> Option<Decision> {
    let n = state.administered.len();
    if n == 0 {
        return None;
    }
    let cut = state.claimed_grade.theta_cut();
    let post = state.rebuild_posterior();
    let p_above = post.prob_above(cut);
    let classified = p_above >= state.config.classify_confidence
        || (1.0 - p_above) >= state.config.classify_confidence;
    let reached_max = n >= state.config.max_items as usize;
    let reached_min = n >= state.config.min_items as usize;
    let exhausted = remaining_templates(bank(), state) == 0;
    if !(reached_max || exhausted || (classified && reached_min)) {
        return None;
    }

    let theta = post.mean();
    let next_cut = state.claimed_grade.next().map(|g| g.theta_cut());
    let confident_above_next =
        next_cut.is_some_and(|c| post.prob_above(c) >= state.config.upgrade_confidence);
    let lz = irt::person_fit_lz(
        theta,
        state.administered.iter().map(|a| (&a.params, a.correct)),
    );
    Some(Decision {
        passed: theta >= cut,
        theta,
        se: post.sd(),
        grade_measured: Grade::for_theta(theta),
        confident_above_next,
        person_fit_lz: lz,
        aberrant: lz < state.config.aberrant_lz,
        items_administered: n as u16,
    })
}

/// Which topics and templates a test for `(spec, grade)` would emphasise, for the T-4 demo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TestPlan {
    pub specialization: Specialization,
    pub grade: Grade,
    pub target_theta: f64,
    pub min_items: u16,
    pub max_items: u16,
    pub topics: Vec<TopicPlan>,
    /// Template ids boosted because their skills overlap the requested stack.
    pub focus_templates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TopicPlan {
    pub topic: String,
    pub topic_ru: String,
    /// Template ids under this topic, ordered by information at the grade's cut.
    pub templates: Vec<String>,
}

pub fn plan_for(spec: Specialization, grade: Grade, skills: &[String]) -> TestPlan {
    let bank = bank();
    let cut = grade.theta_cut();
    let focus_mask = items::skill_mask(skills.iter());
    let config = CatConfig::default();

    let mut topics: Vec<TopicPlan> = Vec::new();
    let mut focus_templates: Vec<String> = Vec::new();
    for topic in topic_targets(bank, spec) {
        let mut templates: Vec<(f64, &str)> = bank
            .for_specialization(spec)
            .filter(|t| t.topic == topic)
            .map(|t| {
                let best = t
                    .levels
                    .iter()
                    .map(|l| irt::information(cut, &l.params))
                    .fold(0.0, f64::max);
                (best, t.id)
            })
            .collect();
        templates.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(b.1)));
        topics.push(TopicPlan {
            topic: format!("{topic:?}"),
            topic_ru: topic.name_ru().to_owned(),
            templates: templates.iter().map(|(_, id)| (*id).to_owned()).collect(),
        });
    }
    if focus_mask != 0 {
        for t in bank.for_specialization(spec) {
            let idx = bank.template_index(t.id).unwrap_or(0);
            if bank.template_skills[idx as usize] & focus_mask != 0 {
                focus_templates.push(t.id.to_owned());
            }
        }
    }

    TestPlan {
        specialization: spec,
        grade,
        target_theta: cut,
        min_items: config.min_items,
        max_items: config.max_items,
        topics,
        focus_templates,
    }
}

/// Renders `n` example items spanning the grade's difficulty, for the T-4 demo.
pub fn sample_items(
    spec: Specialization,
    grade: Grade,
    skills: &[String],
    n: usize,
    seed: u64,
) -> Vec<ItemView> {
    let bank = bank();
    let cut = grade.theta_cut();
    let focus_mask = items::skill_mask(skills.iter());
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let mut candidates: Vec<(f32, Slot)> = bank
        .slots(spec)
        .iter()
        .map(|&slot| {
            let idx = bank.hot.template[slot as usize];
            let info = irt::information(cut, &bank.slot_params(slot)) as f32;
            let focus = if focus_mask != 0 && bank.template_skills[idx as usize] & focus_mask != 0 {
                1.5
            } else {
                1.0
            };
            (info * focus, slot)
        })
        .collect();
    candidates.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));

    let mut views = Vec::new();
    let mut used_templates: Vec<u16> = Vec::new();
    for &(_, slot) in &candidates {
        if views.len() >= n {
            break;
        }
        let idx = bank.hot.template[slot as usize];
        if used_templates.contains(&idx) {
            continue;
        }
        used_templates.push(idx);
        let template = bank.template_at(idx);
        let item = ItemRef {
            template_id: template.id.to_owned(),
            level: bank.hot.level[slot as usize],
            seed: rng.next_u64(),
        };
        if let Ok(view) = items::render(&item) {
            views.push(view);
        }
    }
    views
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng as _;

    fn simulate(spec: Specialization, claimed: Grade, true_theta: f64, seed: u64) -> Decision {
        let mut state = start(spec, claimed, seed, Vec::new());
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0xABCD);
        while let Some(item) = next_item(&state) {
            let params = bank().params(&item).expect("known item");
            let correct = rng.random::<f64>() < irt::p_correct(true_theta, &params);
            record(&mut state, item, correct, 30_000).unwrap();
        }
        decision(&state).expect("session stops")
    }

    #[test]
    fn classification_accuracy_for_middle() {
        let runs = 200;
        let mut pass_high = 0;
        let mut fail_low = 0;
        let mut lengths = Vec::new();
        for s in 0..runs {
            let hi = simulate(Specialization::Backend, Grade::Middle, 1.5, s);
            let lo = simulate(Specialization::Backend, Grade::Middle, -0.7, s + 10_000);
            if hi.passed {
                pass_high += 1;
            }
            if !lo.passed {
                fail_low += 1;
            }
            lengths.push(hi.items_administered as f64);
            lengths.push(lo.items_administered as f64);
            assert!((8..=20).contains(&hi.items_administered));
            assert!((8..=20).contains(&lo.items_administered));
        }
        assert!(
            pass_high as f64 / runs as f64 >= 0.9,
            "θ=1.5 passed {pass_high}/{runs}"
        );
        assert!(
            fail_low as f64 / runs as f64 >= 0.9,
            "θ=-0.7 failed {fail_low}/{runs}"
        );
        let mean_len = lengths.iter().sum::<f64>() / lengths.len() as f64;
        assert!((8.0..=20.0).contains(&mean_len), "mean length {mean_len}");
    }

    #[test]
    fn every_specialization_terminates() {
        for &spec in Specialization::ALL {
            for &grade in Grade::ALL {
                let d = simulate(spec, grade, grade.theta_cut() + 0.5, 42);
                assert!((8..=20).contains(&d.items_administered), "{spec} {grade}");
            }
        }
    }

    #[test]
    fn next_item_is_deterministic_and_unique() {
        let mut state = start(Specialization::Ml, Grade::Middle, 123, Vec::new());
        assert_eq!(next_item(&state), next_item(&state));
        let mut seen: Vec<String> = Vec::new();
        while let Some(item) = next_item(&state) {
            assert!(
                !seen.contains(&item.template_id),
                "repeated template {}",
                item.template_id
            );
            seen.push(item.template_id.clone());
            record(&mut state, item, true, 10_000).unwrap();
        }
        assert!(seen.len() >= 8);
    }

    #[test]
    fn strong_candidate_gets_upgrade_offer() {
        let d = simulate(Specialization::Frontend, Grade::Middle, 2.3, 7);
        assert!(d.passed && d.confident_above_next);
        assert_eq!(d.grade_measured, Some(Grade::Senior));
    }

    #[test]
    fn aberrant_pattern_is_flagged() {
        // Build a session whose administered items span difficulty, then answer them in the
        // inverted "hard right, easy wrong" pattern that l_z is designed to catch. (Adaptive
        // selection converges on θ and removes the spread, so this is tested directly.)
        let mut state = start(Specialization::Backend, Grade::Middle, 5, Vec::new());
        state.config.max_items = 12;
        for k in 0..12i32 {
            let b = -2.0 + f64::from(k) * 0.35;
            state.administered.push(AdministeredItem {
                item: ItemRef {
                    template_id: format!("synthetic.{k}"),
                    level: 1,
                    seed: 1,
                },
                correct: b >= 0.0,
                elapsed_ms: 5_000,
                topic: Topic::Algorithms,
                params: ItemParams::new(1.5, b, 0.0),
            });
        }
        let d = decision(&state).unwrap();
        assert!(
            d.aberrant && d.person_fit_lz < -2.0,
            "lz = {}",
            d.person_fit_lz
        );
    }

    #[test]
    fn state_round_trips_through_json() {
        let mut state = start(
            Specialization::Devops,
            Grade::Senior,
            99,
            vec!["kubernetes".into()],
        );
        for _ in 0..3 {
            if let Some(item) = next_item(&state) {
                record(&mut state, item, true, 12_000).unwrap();
            }
        }
        let json = serde_json::to_string(&state).unwrap();
        let back: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, back);
        assert_eq!(next_item(&state), next_item(&back));
    }

    #[test]
    fn duplicate_and_unknown_records_are_rejected() {
        let mut state = start(Specialization::Qa, Grade::Junior, 1, Vec::new());
        let item = next_item(&state).unwrap();
        record(&mut state, item.clone(), true, 1000).unwrap();
        let same_template = ItemRef {
            seed: item.seed + 1,
            ..item.clone()
        };
        assert!(matches!(
            record(&mut state, same_template, true, 1000),
            Err(RecordError::DuplicateTemplate { .. })
        ));
        let bogus = ItemRef {
            template_id: "nope".into(),
            level: 1,
            seed: 1,
        };
        assert!(matches!(
            record(&mut state, bogus, true, 1000),
            Err(RecordError::UnknownItem(_))
        ));
    }

    #[test]
    fn focus_skills_bias_selection() {
        let plan = plan_for(
            Specialization::Devops,
            Grade::Middle,
            &["kubernetes".into()],
        );
        assert!(plan.focus_templates.iter().any(|t| t.contains("k8s")));
        let views = sample_items(
            Specialization::Devops,
            Grade::Middle,
            &["kubernetes".into()],
            5,
            3,
        );
        assert_eq!(views.len(), 5);
        assert!(views.iter().all(|v| !v.title.is_empty()));
    }
}
