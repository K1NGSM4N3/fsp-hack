//! Three-parameter logistic IRT model and a grid (quadrature) posterior over ability θ.

use serde::{Deserialize, Serialize};

/// Logistic scaling constant: with `D = 1.702` the logistic curve stays within 0.01 of the
/// normal ogive, so `a` reads on the conventional "normal metric" (Lord, 1980).
pub const D: f64 = 1.702;

pub const GRID_POINTS: usize = 81;
pub const GRID_MIN: f64 = -4.0;
pub const GRID_MAX: f64 = 4.0;
const GRID_STEP: f64 = (GRID_MAX - GRID_MIN) / (GRID_POINTS - 1) as f64;
const P_EPS: f64 = 1e-12;

/// Discrimination `a`, difficulty `b`, pseudo-guessing `c` (lower asymptote).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ItemParams {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl ItemParams {
    pub const fn new(a: f64, b: f64, c: f64) -> Self {
        Self { a, b, c }
    }

    pub fn p_correct(&self, theta: f64) -> f64 {
        p_correct(theta, self)
    }

    pub fn information(&self, theta: f64) -> f64 {
        information(theta, self)
    }
}

pub fn p_correct(theta: f64, item: &ItemParams) -> f64 {
    let z = D * item.a * (theta - item.b);
    item.c + (1.0 - item.c) * logistic(z)
}

/// Birnbaum's 3PL information: `I(θ) = D²a² · (Q/P) · ((P − c) / (1 − c))²`.
/// Guessing lowers information most for low θ, which is why CAT prefers
/// open-answer items (`c = 0`) when they are on target.
pub fn information(theta: f64, item: &ItemParams) -> f64 {
    let p = p_correct(theta, item).clamp(P_EPS, 1.0 - P_EPS);
    let ratio = (p - item.c) / (1.0 - item.c);
    let da = D * item.a;
    da * da * ((1.0 - p) / p) * ratio * ratio
}

fn logistic(z: f64) -> f64 {
    if z >= 0.0 {
        1.0 / (1.0 + (-z).exp())
    } else {
        let e = z.exp();
        e / (1.0 + e)
    }
}

pub fn grid_node(i: usize) -> f64 {
    GRID_MIN + i as f64 * GRID_STEP
}

/// Posterior over θ on a fixed 81-point grid on [-4, 4].
///
/// Weights are kept as log-densities and re-centred after every update, so long runs of
/// correct (or incorrect) answers never underflow. Each node stands for a bin of width
/// `GRID_STEP`; tail probabilities and quantiles integrate the bins piecewise-uniformly,
/// which keeps `prob_above` continuous in the cut instead of jumping at grid nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct Posterior {
    log_w: [f64; GRID_POINTS],
}

impl Posterior {
    /// Normal prior `N(mean, 1)`.
    pub fn new(prior_mean: f64) -> Self {
        Self::with_prior(prior_mean, 1.0)
    }

    pub fn with_prior(mean: f64, sd: f64) -> Self {
        let sd = sd.max(1e-3);
        let mut log_w = [0.0; GRID_POINTS];
        for (i, w) in log_w.iter_mut().enumerate() {
            let z = (grid_node(i) - mean) / sd;
            *w = -0.5 * z * z;
        }
        Self { log_w }
    }

    pub fn from_responses<'a>(
        prior_mean: f64,
        responses: impl IntoIterator<Item = (&'a ItemParams, bool)>,
    ) -> Self {
        let mut post = Self::new(prior_mean);
        for (params, correct) in responses {
            post.update(params, correct);
        }
        post
    }

    pub fn update(&mut self, item: &ItemParams, correct: bool) {
        let mut max = f64::NEG_INFINITY;
        for (i, w) in self.log_w.iter_mut().enumerate() {
            let p = p_correct(grid_node(i), item).clamp(P_EPS, 1.0 - P_EPS);
            *w += if correct { p.ln() } else { (1.0 - p).ln() };
            max = max.max(*w);
        }
        for w in &mut self.log_w {
            *w -= max;
        }
    }

    fn weights(&self) -> [f64; GRID_POINTS] {
        let max = self.log_w.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mut w = [0.0; GRID_POINTS];
        let mut total = 0.0;
        for (dst, &lw) in w.iter_mut().zip(&self.log_w) {
            *dst = (lw - max).exp();
            total += *dst;
        }
        for dst in &mut w {
            *dst /= total;
        }
        w
    }

    /// Expected a posteriori estimate of θ.
    pub fn mean(&self) -> f64 {
        self.weights()
            .iter()
            .enumerate()
            .map(|(i, w)| w * grid_node(i))
            .sum()
    }

    /// Posterior standard deviation — the standard error of the EAP estimate.
    pub fn sd(&self) -> f64 {
        let w = self.weights();
        let mean: f64 = w.iter().enumerate().map(|(i, w)| w * grid_node(i)).sum();
        w.iter()
            .enumerate()
            .map(|(i, w)| {
                let d = grid_node(i) - mean;
                w * d * d
            })
            .sum::<f64>()
            .sqrt()
    }

    /// `P(θ ≥ cut)`.
    pub fn prob_above(&self, cut: f64) -> f64 {
        self.weights()
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let hi = grid_node(i) + GRID_STEP / 2.0;
                w * ((hi - cut) / GRID_STEP).clamp(0.0, 1.0)
            })
            .sum::<f64>()
            .clamp(0.0, 1.0)
    }

    pub fn quantile(&self, q: f64) -> f64 {
        let q = q.clamp(0.0, 1.0);
        let mut cum = 0.0;
        for (i, w) in self.weights().iter().enumerate() {
            if *w > 0.0 && cum + w >= q {
                let lo = grid_node(i) - GRID_STEP / 2.0;
                return lo + (q - cum) / w * GRID_STEP;
            }
            cum += w;
        }
        GRID_MAX + GRID_STEP / 2.0
    }

    /// Equal-tailed credible interval, e.g. `credible_interval(0.95)`.
    pub fn credible_interval(&self, level: f64) -> (f64, f64) {
        let tail = (1.0 - level.clamp(0.0, 1.0)) / 2.0;
        (self.quantile(tail), self.quantile(1.0 - tail))
    }
}

/// Standardized log-likelihood person-fit statistic `l_z` (Drasgow, Levine & Williams, 1985).
///
/// Strongly negative values mean the response pattern is less likely than the model expects
/// at the candidate's own θ — typically "hard items right, easy items wrong", the signature
/// of leaked answers or assisted testing.
pub fn person_fit_lz<'a>(
    theta: f64,
    responses: impl IntoIterator<Item = (&'a ItemParams, bool)>,
) -> f64 {
    let (mut l0, mut expected, mut variance) = (0.0, 0.0, 0.0);
    for (item, correct) in responses {
        let p = p_correct(theta, item).clamp(P_EPS, 1.0 - P_EPS);
        let q = 1.0 - p;
        let (lp, lq) = (p.ln(), q.ln());
        l0 += if correct { lp } else { lq };
        expected += p * lp + q * lq;
        let logit = lp - lq;
        variance += p * q * logit * logit;
    }
    if variance <= 1e-12 {
        0.0
    } else {
        (l0 - expected) / variance.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEM: ItemParams = ItemParams::new(1.2, 0.0, 0.25);

    #[test]
    fn p_correct_is_monotone_and_bounded_by_c() {
        let mut prev = 0.0;
        for i in 0..GRID_POINTS {
            let p = p_correct(grid_node(i), &ITEM);
            assert!((ITEM.c..=1.0).contains(&p) && p >= prev);
            prev = p;
        }
        assert!((p_correct(0.0, &ITEM) - 0.625).abs() < 1e-12);
        assert!(p_correct(-1e6, &ITEM) >= ITEM.c);
    }

    #[test]
    fn information_peaks_slightly_above_b_with_guessing() {
        let best = (0..GRID_POINTS)
            .map(grid_node)
            .max_by(|x, y| information(*x, &ITEM).total_cmp(&information(*y, &ITEM)))
            .unwrap();
        // Closed form for the 3PL maximum: b + ln((1 + √(1 + 8c)) / 2) / (D·a).
        let expected = ITEM.b + ((1.0 + (1.0 + 8.0 * ITEM.c).sqrt()) / 2.0).ln() / (D * ITEM.a);
        assert!(
            (best - expected).abs() <= 0.05 + 1e-9,
            "{best} vs {expected}"
        );
        let two_pl = ItemParams::new(1.2, 0.0, 0.0);
        assert!((information(0.0, &two_pl) - (D * 1.2f64).powi(2) * 0.25).abs() < 1e-9);
    }

    #[test]
    fn prior_summaries_match_standard_normal() {
        let post = Posterior::new(0.0);
        assert!(post.mean().abs() < 1e-9);
        assert!((post.sd() - 1.0).abs() < 0.01);
        assert!((post.prob_above(0.0) - 0.5).abs() < 1e-9);
        assert!((post.prob_above(1.0) - 0.1587).abs() < 0.005);
        let (lo, hi) = post.credible_interval(0.95);
        assert!(
            (lo + 1.96).abs() < 0.03 && (hi - 1.96).abs() < 0.03,
            "{lo} {hi}"
        );
    }

    #[test]
    fn updates_move_the_posterior_and_shrink_it() {
        let mut post = Posterior::new(0.0);
        post.update(&ITEM, true);
        let after_correct = post.mean();
        assert!(after_correct > 0.0);
        assert!(post.sd() < 1.0);
        post.update(&ITEM, false);
        assert!(post.mean() < after_correct);
    }

    #[test]
    fn long_streaks_stay_finite() {
        let mut post = Posterior::new(0.0);
        let hard = ItemParams::new(2.0, 3.5, 0.0);
        for _ in 0..500 {
            post.update(&hard, true);
        }
        assert!(post.mean().is_finite() && post.mean() > 3.5);
        assert!(post.prob_above(3.0) > 0.99);
        let mut post = Posterior::new(0.0);
        let easy = ItemParams::new(2.0, -3.5, 0.0);
        for _ in 0..500 {
            post.update(&easy, false);
        }
        assert!(post.mean().is_finite() && post.mean() < -3.5);
    }

    #[test]
    fn eap_recovers_true_theta_with_many_items() {
        use rand::{Rng, SeedableRng};
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);
        let true_theta = 0.8;
        let mut post = Posterior::new(0.0);
        for k in 0..200 {
            let item = ItemParams::new(1.3, -2.0 + (k % 9) as f64 * 0.5, 0.0);
            let correct = rng.random::<f64>() < p_correct(true_theta, &item);
            post.update(&item, correct);
        }
        assert!(
            (post.mean() - true_theta).abs() < 3.0 * post.sd(),
            "{}",
            post.mean()
        );
        assert!(post.sd() < 0.2);
        let (lo, hi) = post.credible_interval(0.95);
        assert!(lo < post.mean() && post.mean() < hi);
        assert!((post.quantile(0.5) - post.mean()).abs() < 0.1);
    }

    #[test]
    fn person_fit_flags_inverted_patterns() {
        let items: Vec<ItemParams> = (0..12)
            .map(|k| ItemParams::new(1.5, -2.0 + k as f64 * 0.35, 0.0))
            .collect();
        let consistent: Vec<bool> = items.iter().map(|it| it.b < 0.0).collect();
        let inverted: Vec<bool> = items.iter().map(|it| it.b >= 0.0).collect();
        let lz_ok = person_fit_lz(0.0, items.iter().zip(consistent));
        let lz_bad = person_fit_lz(0.0, items.iter().zip(inverted));
        assert!(lz_ok > -1.0, "{lz_ok}");
        assert!(lz_bad < -2.0, "{lz_bad}");
    }
}
