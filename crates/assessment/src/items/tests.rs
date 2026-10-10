use catalog::Specialization;

use super::*;

const SEEDS: u64 = 500;

fn distinct_count<'a>(items: impl Iterator<Item = &'a str>) -> usize {
    let mut v: Vec<&str> = items.collect();
    v.sort_unstable();
    v.dedup();
    v.len()
}

fn item(t: &Template, level: u8, seed: u64) -> ItemRef {
    ItemRef {
        template_id: t.id.to_owned(),
        level,
        seed,
    }
}

fn wrong_answer(instance: &ItemInstance) -> Answer {
    match instance.correct_answer() {
        Answer::Choice(k) => Answer::Choice((k + 1) % instance.view.options.len()),
        Answer::MultiChoice(mut k) => {
            if k.contains(&0) {
                k.retain(|i| *i != 0);
            } else {
                k.push(0);
            }
            Answer::MultiChoice(k)
        }
        Answer::Number(v) => Answer::Number(v + 1.0 + v.abs()),
        Answer::Text(_) => Answer::Text("заведомо неверный ответ".to_owned()),
    }
}

#[test]
fn every_template_level_generates_valid_items() {
    for t in bank().templates() {
        for level in t.levels {
            let mut prompts: Vec<String> = Vec::new();
            for seed in 0..SEEDS {
                let r = item(t, level.level, seed);
                let inst = generate(&r).unwrap_or_else(|e| panic!("{}: {e}", t.id));
                let v = &inst.view;
                let ctx = format!("{} L{} seed {seed}", t.id, level.level);
                assert!(!v.prompt.trim().is_empty(), "{ctx}: empty prompt");
                assert_eq!(v.time_limit_secs, u32::from(level.time_limit_secs));

                let c = level.params.c;
                match v.kind {
                    ItemKind::SingleChoice => {
                        assert_eq!(v.options.len(), CHOICE_OPTIONS, "{ctx}: {:?}", v.options);
                        assert!((c - C_CHOICE).abs() < 1e-9, "{ctx}: c={c}");
                    }
                    ItemKind::MultiChoice => {
                        let subsets = (1u64 << v.options.len()) - 1;
                        assert!((c - 1.0 / subsets as f64).abs() < 1e-9, "{ctx}: c={c}");
                    }
                    ItemKind::Number | ItemKind::Text => {
                        assert!(v.options.is_empty(), "{ctx}");
                        assert_eq!(c, C_OPEN, "{ctx}");
                    }
                }

                assert_eq!(
                    distinct_count(v.options.iter().map(|o| o.trim())),
                    v.options.len(),
                    "{ctx}: duplicate options {:?}",
                    v.options
                );
                assert!(
                    v.options.iter().all(|o| !o.trim().is_empty()),
                    "{ctx}: blank option"
                );

                let correct = inst.correct_answer();
                match &correct {
                    Answer::Choice(k) => assert!(*k < v.options.len(), "{ctx}"),
                    Answer::MultiChoice(k) => {
                        assert!(!k.is_empty() && k.len() < v.options.len(), "{ctx}: {k:?}");
                    }
                    Answer::Number(x) => assert!(x.is_finite(), "{ctx}"),
                    Answer::Text(s) => assert!(!s.trim().is_empty(), "{ctx}"),
                }
                assert!(inst.check(&correct), "{ctx}: own answer rejected");
                assert!(check(&r, &correct), "{ctx}: regenerated check failed");
                assert!(
                    !inst.check(&wrong_answer(&inst)),
                    "{ctx}: wrong answer accepted"
                );
                assert!(!inst.correct_answer_text().is_empty(), "{ctx}");

                if seed < 50 {
                    prompts.push(format!("{}|{:?}|{:?}", v.prompt, v.code, v.options));
                }
            }
            let distinct = distinct_count(prompts.iter().map(String::as_str));
            assert!(
                distinct >= 10,
                "{} L{}: only {} distinct instances in 50 seeds",
                t.id,
                level.level,
                distinct
            );
        }
    }
}

/// Leak resistance (architecture section 5.2) needs a leaked instance to practically never recur:
/// with N equally likely variants, 500 seeds collide about 500² / 2N times, so ≥ 490 distinct
/// means N in the tens of thousands. Pools are capped by their hand-written combinations.
#[test]
fn instances_rarely_repeat() {
    const VARIETY_SEEDS: u64 = 500;
    for t in bank().templates() {
        let min = 490;
        for level in t.levels {
            let rendered: Vec<String> = (0..VARIETY_SEEDS)
                .map(|seed| {
                    let v = render(&item(t, level.level, seed)).unwrap();
                    format!("{}|{:?}|{:?}", v.prompt, v.code, v.options)
                })
                .collect();
            let distinct = distinct_count(rendered.iter().map(String::as_str));
            assert!(
                distinct >= min,
                "{} L{}: only {distinct} distinct instances in {VARIETY_SEEDS} seeds",
                t.id,
                level.level
            );
        }
    }
}

#[test]
fn generation_is_deterministic() {
    for t in bank().templates() {
        for level in t.levels {
            for seed in [0, 1, 42, 9_007_199_254_740_991] {
                let r = item(t, level.level, seed);
                assert_eq!(generate(&r).unwrap(), generate(&r).unwrap(), "{}", t.id);
            }
        }
    }
}

#[test]
fn unknown_references_fail_cleanly() {
    let r = ItemRef {
        template_id: "nope".into(),
        level: 1,
        seed: 1,
    };
    assert!(matches!(generate(&r), Err(ItemError::UnknownTemplate(_))));
    assert!(!check(&r, &Answer::Choice(0)));
    let t = bank().templates()[0];
    let r = ItemRef {
        template_id: t.id.into(),
        level: 99,
        seed: 1,
    };
    assert!(matches!(generate(&r), Err(ItemError::UnknownLevel { .. })));
}

#[test]
fn bank_metadata_is_consistent() {
    let templates = bank().templates();
    assert!(templates.len() >= 36, "only {} templates", templates.len());
    assert_eq!(
        distinct_count(templates.iter().map(|t| t.id)),
        templates.len(),
        "duplicate template ids"
    );

    for t in templates {
        assert!(!t.specializations.is_empty(), "{}", t.id);
        assert!(!t.skills.is_empty(), "{}", t.id);
        for s in t.skills {
            assert!(SKILLS.contains(s), "{}: unknown skill `{s}`", t.id);
        }
        assert!((1..=3).contains(&t.levels.len()), "{}", t.id);
        for (i, l) in t.levels.iter().enumerate() {
            assert_eq!(l.level as usize, i + 1, "{}: levels must be 1..n", t.id);
            assert!(
                (0.8..=2.0).contains(&l.params.a),
                "{}: a={}",
                t.id,
                l.params.a
            );
            assert!(
                (-1.8..=1.8).contains(&l.params.b),
                "{}: b={}",
                t.id,
                l.params.b
            );
            assert!((60..=180).contains(&l.time_limit_secs), "{}", t.id);
        }
        for pair in t.levels.windows(2) {
            assert!(
                pair[0].params.b < pair[1].params.b,
                "{}: levels must get harder",
                t.id
            );
        }
        if let Source::Pool(levels) = &t.source {
            assert_eq!(levels.len(), t.levels.len(), "{}", t.id);
            for entries in levels.iter() {
                assert!(!entries.is_empty(), "{}", t.id);
                for e in entries.iter() {
                    assert!(e.stems.len() >= 2, "{}: {:?}", t.id, e.stems);
                    assert!(e.correct.len() >= 2, "{}: {:?}", t.id, e.stems);
                    assert!(e.distractors.len() >= 4, "{}: {:?}", t.id, e.stems);
                    for c in e.correct {
                        assert!(!e.distractors.contains(c), "{}: {c}", t.id);
                    }
                    let all = distinct_count(e.correct.iter().chain(e.distractors.iter()).copied());
                    assert_eq!(all, e.correct.len() + e.distractors.len(), "{}", t.id);
                }
            }
        }
    }
}

#[test]
fn every_specialization_is_covered_around_all_cuts() {
    for &spec in Specialization::ALL {
        let templates: Vec<&Template> = bank().for_specialization(spec).collect();
        assert!(
            templates.len() >= 7,
            "{spec}: {} templates",
            templates.len()
        );
        let mut topics: Vec<Topic> = templates.iter().map(|t| t.topic).collect();
        topics.sort_unstable();
        topics.dedup();
        assert!(topics.len() >= 3, "{spec}: {} topics", topics.len());
        let bs: Vec<f64> = templates
            .iter()
            .flat_map(|t| t.levels.iter().map(|l| l.params.b))
            .collect();
        let count = |lo: f64, hi: f64| bs.iter().filter(|b| (lo..=hi).contains(*b)).count();
        assert!(count(-1.5, -0.5) >= 3, "{spec}: junior band");
        assert!(count(-0.5, 0.5) >= 3, "{spec}: middle band");
        assert!(count(0.5, 1.8) >= 3, "{spec}: senior band");
    }
}

#[test]
fn hot_tables_mirror_templates() {
    let b = bank();
    let total: usize = b.templates().iter().map(|t| t.levels.len()).sum();
    assert_eq!(b.hot.a.len(), total);
    for &spec in Specialization::ALL {
        for &slot in b.slots(spec) {
            let t = b.template_at(b.hot.template[slot as usize]);
            assert!(t.applies_to(spec));
            let level = t.level(b.hot.level[slot as usize]).unwrap();
            let p = b.slot_params(slot);
            assert!((p.b - level.params.b).abs() < 1e-6 && (p.a - level.params.a).abs() < 1e-6);
        }
    }
}

#[test]
fn views_serialize_without_answer() {
    let t = bank().template("http.status_codes").unwrap();
    let inst = generate(&item(t, 1, 3)).unwrap();
    let json = serde_json::to_string(&inst.view).unwrap();
    assert!(!json.contains("seed"));
    let answer: Answer = serde_json::from_str(r#"{"type":"choice","value":2}"#).unwrap();
    assert_eq!(answer, Answer::Choice(2));
}

/// `cargo test -p assessment dump_samples -- --ignored --nocapture` prints one instance per
/// template level for manual review.
#[test]
#[ignore]
fn dump_samples() {
    let filter = std::env::var("DUMP").unwrap_or_default();
    for t in bank().templates() {
        if !t.id.contains(&filter) {
            continue;
        }
        for level in t.levels {
            for seed in 0..2 {
                let inst = generate(&item(t, level.level, seed)).unwrap();
                let v = &inst.view;
                println!(
                    "==== {} L{} seed {seed} [{:?}] ====",
                    t.id, level.level, v.kind
                );
                println!("{}", v.prompt);
                if let Some(code) = &v.code {
                    println!("```{}\n{}\n```", code.language, code.source);
                }
                for (i, o) in v.options.iter().enumerate() {
                    println!("  {i}) {o}");
                }
                println!("  => {}", inst.correct_answer_text());
            }
        }
    }
}
