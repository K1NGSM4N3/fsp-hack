//! Variant pools: conceptual questions as data. An entry has several paraphrased stems,
//! several equally correct answers and a larger set of distractors; an instance draws one
//! stem, one correct answer and three distractors, so a leaked instance covers only a
//! small fraction of the entry's combinations.

use super::util::{pick, pick_n};
use super::{CHOICE_OPTIONS, Draft, Rng};

pub(crate) struct Entry {
    pub stems: &'static [&'static str],
    pub code: Option<(&'static str, &'static str)>,
    pub correct: &'static [&'static str],
    pub distractors: &'static [&'static str],
}

pub(crate) fn generate(entries: &[Entry], rng: &mut Rng) -> Draft {
    let Some(entry) = (!entries.is_empty()).then(|| pick(rng, entries)) else {
        return Draft::choice("—", "—", Vec::new());
    };
    let stem = *pick(rng, entry.stems);
    let correct = *pick(rng, entry.correct);
    let distractors = pick_n(rng, entry.distractors, CHOICE_OPTIONS - 1)
        .into_iter()
        .map(str::to_owned);
    let draft = Draft::choice(stem, correct, distractors);
    match entry.code {
        Some((language, source)) => draft.code(language, source),
        None => draft,
    }
}
