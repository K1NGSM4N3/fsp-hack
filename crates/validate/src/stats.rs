/// Graded relevance 0..=3; "relevant" for precision means grade ≥ 2.
pub const RELEVANT: u8 = 2;

pub fn precision_at(ranked: &[u8], k: usize) -> f64 {
    let k = k.min(ranked.len());
    if k == 0 {
        return 0.0;
    }
    ranked[..k].iter().filter(|&&g| g >= RELEVANT).count() as f64 / k as f64
}

fn dcg(grades: &[u8], k: usize) -> f64 {
    grades
        .iter()
        .take(k)
        .enumerate()
        .map(|(i, &g)| (2f64.powi(i32::from(g)) - 1.0) / ((i + 2) as f64).log2())
        .sum()
}

/// nDCG@k against the ideal ordering of all judged candidates (not only the retrieved ones),
/// so a ranker is penalized for missing strong candidates entirely.
pub fn ndcg_at(ranked: &[u8], all: &[u8], k: usize) -> f64 {
    let mut ideal = all.to_vec();
    ideal.sort_unstable_by(|a, b| b.cmp(a));
    let best = dcg(&ideal, k);
    if best == 0.0 {
        0.0
    } else {
        dcg(ranked, k) / best
    }
}

pub fn reciprocal_rank(ranked: &[u8]) -> f64 {
    ranked
        .iter()
        .position(|&g| g >= RELEVANT)
        .map_or(0.0, |p| 1.0 / (p + 1) as f64)
}

pub struct Summary {
    pub mean: f64,
    /// Half-width of the 95 % normal-approximation interval.
    pub ci95: f64,
}

pub fn summarize(values: &[f64]) -> Summary {
    let n = values.len().max(1) as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    Summary {
        mean,
        ci95: 1.96 * (var / n).sqrt(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics() {
        assert_eq!(precision_at(&[3, 0, 2, 1], 4), 0.5);
        assert_eq!(reciprocal_rank(&[0, 1, 3]), 1.0 / 3.0);
        assert!((ndcg_at(&[3, 2, 0], &[3, 2, 0], 3) - 1.0).abs() < 1e-9);
        assert!(ndcg_at(&[0, 2, 3], &[3, 2, 0], 3) < 1.0);
    }
}
