//! End-to-end: free-text need → draft → ranked shortlist with explanations.

use catalog::{Category, Grade, Industry, Specialization, WorkFormat};
use matching::{
    Achievement, AchievementKind, CandidatePool, CandidateSkill, CandidateSnapshot, Discipline,
    EventLevel, FspSummary, Place, SkillSource, Weights, fsp_score, parse_need, rank,
};

const NOW: i64 = 1_791_158_400; // 2026-10-05
const DAY: i64 = 86_400;

fn skills(list: &[(&str, SkillSource)]) -> Vec<CandidateSkill> {
    list.iter()
        .map(|(slug, source)| CandidateSkill {
            slug: (*slug).into(),
            source: *source,
        })
        .collect()
}

fn base(id: &str, grade: Grade, theta: f64) -> CandidateSnapshot {
    CandidateSnapshot {
        id: id.into(),
        category: Category::new(Specialization::Backend, grade),
        theta,
        skills: Vec::new(),
        fsp: FspSummary::default(),
        salary_expectation_min: Some(380_000),
        work_formats: vec![WorkFormat::Remote],
        city: Some("Новосибирск".into()),
        relocation: false,
        last_activity_unix: NOW - 5 * DAY,
        industries: Vec::new(),
        text_relevance: 0.2,
    }
}

fn pool() -> CandidatePool {
    use SkillSource::*;
    let fsp = fsp_score(
        &[Achievement {
            kind: AchievementKind::Competition,
            discipline: Discipline::Product,
            rank: None,
            event_name: Some("Кубок России по продуктовому программированию".into()),
            event_level: Some(EventLevel::National),
            place: Some(Place::Second),
            team: Some("Null Pointers".into()),
            role: Some("бэкенд".into()),
            date: "2025-11-20".into(),
            protocol_url: None,
        }],
        Specialization::Backend,
        NOW,
    );
    let mut perfect = base("perfect", Grade::Senior, 1.9);
    perfect.skills = skills(&[
        ("go", Test),
        ("postgresql", Test),
        ("redis", Resume),
        ("kafka", Resume),
        ("docker", Resume),
        ("kubernetes", Resume),
        ("grpc", Declared),
    ]);
    perfect.industries = vec![Industry::Fintech];
    let mut fsp_twin = perfect.clone();
    fsp_twin.id = "perfect_fsp".into();
    fsp_twin.fsp = FspSummary::from(&fsp);
    let mut weak_test = perfect.clone();
    weak_test.id = "weak_test".into();
    weak_test.theta = 1.05;
    let mut partial = base("partial", Grade::Senior, 1.9);
    partial.skills = skills(&[
        ("go", Test),
        ("mysql", Resume),
        ("rabbitmq", Resume),
        ("docker", Resume),
    ]);
    let mut wrong_stack = base("java_dev", Grade::Senior, 1.9);
    wrong_stack.skills = skills(&[
        ("java", Test),
        ("spring", Test),
        ("postgresql", Test),
        ("kafka", Test),
    ]);
    let mut middle = base("middle", Grade::Middle, 0.9);
    middle.skills = perfect.skills.clone();
    let mut office_only = base("office_only", Grade::Senior, 2.0);
    office_only.work_formats = vec![WorkFormat::Office];
    office_only.skills = perfect.skills.clone();
    let mut frontend = base("frontend", Grade::Senior, 2.0);
    frontend.category.specialization = Specialization::Frontend;
    CandidatePool::from_snapshots(&[
        wrong_stack,
        partial,
        weak_test,
        perfect,
        fsp_twin,
        middle,
        office_only,
        frontend,
    ])
}

#[test]
fn need_text_to_explained_shortlist() {
    let text = std::fs::read_to_string(format!(
        "{}/tests/fixtures/need_backend_go.txt",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let need = parse_need(&text)
        .into_need(&text)
        .expect("specialization detected");
    let pool = pool();
    let ranked = rank(&need, &pool, &Weights::default(), NOW, 3);
    let ids: Vec<&str> = ranked.iter().map(|r| pool.id(r.index).unwrap()).collect();
    let pos = |id: &str| {
        ids.iter()
            .position(|x| *x == id)
            .unwrap_or_else(|| panic!("{id} missing: {ids:?}"))
    };
    // Middle is outside the requested grade, office-only cannot work remotely/hybrid,
    // frontend is another specialization.
    assert_eq!(ids.len(), 5, "{ids:?}");
    assert_eq!(
        ids[0], "perfect_fsp",
        "FSP result lifts an otherwise identical profile"
    );
    assert!(
        pos("perfect") < pos("weak_test"),
        "same skills, stronger test"
    );
    assert!(
        pos("perfect") < pos("partial"),
        "same test, full stack beats similar skills"
    );
    assert!(
        pos("partial") < pos("java_dev"),
        "similar skills beat a different stack"
    );

    let top = ranked[0].breakdown.as_ref().unwrap();
    assert!(top.total > 65.0, "{}", top.total);
    assert!(
        top.reasons.iter().any(|r| r.starts_with(
            "ФСП: Призёр (2 место) всероссийских соревнований по продуктовому программированию"
        )),
        "{:?}",
        top.reasons
    );
    assert!(
        top.reasons
            .iter()
            .any(|r| r.contains("обязательных навыков")),
        "{:?}",
        top.reasons
    );
    assert!(ranked[3].breakdown.is_none());
    for r in &ranked {
        assert!(r.total >= 0.0 && r.total <= 100.0);
    }
    let partial = matching::score(&need, &partial_overpriced(), &Weights::default(), NOW);
    for gap in [
        "Вместо PostgreSQL — близкий навык MySQL",
        "Нет обязательного навыка: Redis",
        "Навык Docker только заявлен, не подтверждён",
        "Ожидания 500\u{a0}000\u{a0}₽ выше вилки на 11 %",
    ] {
        assert!(
            partial.gaps.iter().any(|g| g == gap),
            "{gap}: {:?}",
            partial.gaps
        );
    }
    assert_eq!(
        partial.features[0].explanation_ru,
        "Обязательные навыки: 2 из 6, ещё 3 — близкими навыками (нет Redis)"
    );
}

fn partial_overpriced() -> CandidateSnapshot {
    use SkillSource::*;
    let mut partial = base("partial", Grade::Senior, 1.2);
    partial.skills = skills(&[
        ("go", Test),
        ("mysql", Resume),
        ("rabbitmq", Resume),
        ("docker", Declared),
    ]);
    partial.salary_expectation_min = Some(500_000);
    partial
}
