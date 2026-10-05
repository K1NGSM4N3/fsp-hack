//! Ranking quality metrics over graded relevance 0..=3 (TREC conventions): `ranked` holds the
//! grades of a ranking in order, unjudged positions as 0.

/// "Relevant" for precision, AP and MRR means grade ≥ 2.
pub const RELEVANT: u8 = 2;

/// Relevant results among the first `k` positions, divided by `k` even when the ranking is
/// shorter: returning fewer candidates is not a way to look more precise.
pub fn precision_at(ranked: &[u8], k: usize) -> f64 {
    if k == 0 {
        return 0.0;
    }
    ranked.iter().take(k).filter(|&&g| g >= RELEVANT).count() as f64 / k as f64
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

/// Mean of precision at each relevant position over `total_relevant` — the number of relevant
/// judgments, so relevant candidates the ranking never returned count as zeros.
pub fn average_precision(ranked: &[u8], total_relevant: usize) -> f64 {
    if total_relevant == 0 {
        return 0.0;
    }
    let mut hits = 0usize;
    let mut sum = 0.0;
    for (i, &g) in ranked.iter().enumerate() {
        if g >= RELEVANT {
            hits += 1;
            sum += hits as f64 / (i + 1) as f64;
        }
    }
    sum / total_relevant as f64
}

pub fn reciprocal_rank(ranked: &[u8]) -> f64 {
    ranked
        .iter()
        .position(|&g| g >= RELEVANT)
        .map_or(0.0, |p| 1.0 / (p + 1) as f64)
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn precision_divides_by_k() {
        assert_eq!(precision_at(&[3, 0, 2, 1], 4), 0.5);
        assert_eq!(precision_at(&[3, 2, 0, 0, 1], 5), 0.4);
        assert_eq!(precision_at(&[3, 2], 5), 0.4);
        assert_eq!(precision_at(&[], 10), 0.0);
        assert_eq!(precision_at(&[3], 0), 0.0);
    }

    #[test]
    fn ndcg_known_values() {
        assert!(close(ndcg_at(&[3, 2, 0], &[3, 2, 0], 3), 1.0));
        // DCG = 3/log2(3) + 7/log2(4) = 1.8928 + 3.5; ideal = 7 + 3/log2(3) = 8.8928.
        let expected = (3.0 / 3f64.log2() + 3.5) / (7.0 + 3.0 / 3f64.log2());
        assert!(close(ndcg_at(&[0, 2, 3], &[3, 2, 0], 3), expected));
        // A judged grade-3 candidate that was never retrieved lowers the ideal-normalized score.
        assert!(close(ndcg_at(&[2], &[3, 2], 1), 3.0 / 7.0));
        assert_eq!(ndcg_at(&[0, 0], &[0, 1], 2), 0.0);
        assert_eq!(ndcg_at(&[1], &[0, 0], 1), 0.0);
    }

    #[test]
    fn average_precision_counts_missed_relevant() {
        // Relevant at ranks 1 and 3: (1/1 + 2/3) / 2.
        assert!(close(
            average_precision(&[3, 0, 2, 0], 2),
            (1.0 + 2.0 / 3.0) / 2.0
        ));
        // Same ranking, but a third relevant candidate was never returned.
        assert!(close(
            average_precision(&[3, 0, 2, 0], 3),
            (1.0 + 2.0 / 3.0) / 3.0
        ));
        assert!(close(average_precision(&[1, 1, 3], 1), 1.0 / 3.0));
        assert_eq!(average_precision(&[0, 0], 0), 0.0);
    }

    #[test]
    fn reciprocal_rank_of_first_relevant() {
        assert_eq!(reciprocal_rank(&[0, 1, 3]), 1.0 / 3.0);
        assert_eq!(reciprocal_rank(&[2]), 1.0);
        assert_eq!(reciprocal_rank(&[1, 1]), 0.0);
    }

    #[test]
    fn summary_interval() {
        let s = summarize(&[0.5, 1.0]);
        assert!(close(s.mean, 0.75));
        // sd = 0.3536, se = 0.25, 1.96 × 0.25.
        assert!(close(s.ci95, 0.49));
        assert_eq!(summarize(&[0.7]).ci95, 0.0);
    }
}
