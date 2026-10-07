//! Splitting a request into forward passes. A pass pads every text to its longest one, and
//! attention costs grow with the square of that length, so texts of similar length go together.

/// Tokens a shorter text may add to the pass's padding before it starts a new pass.
pub const SLACK: usize = 32;

/// Indices of the texts for each pass: longest first, at most `max_texts` per pass and at most
/// `max_tokens` padded tokens (texts × longest length). A text less than half as long as the pass's
/// longest (give or take [`SLACK`] tokens) starts a new pass instead of being padded. A text longer
/// than `max_tokens` gets a pass of its own.
pub fn plan(lengths: &[usize], max_texts: usize, max_tokens: usize) -> Vec<Vec<usize>> {
    let mut order: Vec<usize> = (0..lengths.len()).collect();
    order.sort_by(|&a, &b| lengths[b].cmp(&lengths[a]).then(a.cmp(&b)));
    let mut passes: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut longest = 0;
    for i in order {
        let fits = current.len() < max_texts.max(1)
            && (current.len() + 1) * longest.max(lengths[i]) <= max_tokens
            && 2 * lengths[i] + SLACK >= longest;
        if !current.is_empty() && !fits {
            passes.push(std::mem::take(&mut current));
        }
        if current.is_empty() {
            longest = lengths[i];
        }
        current.push(i);
    }
    if !current.is_empty() {
        passes.push(current);
    }
    passes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_is_in_exactly_one_pass() {
        let lengths = [5, 120, 7, 300, 64, 64, 2, 512, 9, 10];
        let passes = plan(&lengths, 4, 1024);
        let mut seen: Vec<usize> = passes.concat();
        seen.sort_unstable();
        assert_eq!(seen, (0..lengths.len()).collect::<Vec<_>>());
        for p in &passes {
            assert!(!p.is_empty() && p.len() <= 4);
            let longest = p.iter().map(|&i| lengths[i]).max().unwrap();
            assert!(p.len() * longest <= 1024 || p.len() == 1, "{p:?}");
        }
    }

    #[test]
    fn similar_lengths_share_a_pass() {
        let passes = plan(&[10, 500, 12, 480, 11], 8, 2048);
        assert_eq!(passes, [vec![1, 3], vec![2, 4, 0]]);
    }

    #[test]
    fn an_oversized_text_runs_alone() {
        assert_eq!(plan(&[3000, 4], 8, 1024), [vec![0], vec![1]]);
        assert!(plan(&[], 8, 1024).is_empty());
    }
}
