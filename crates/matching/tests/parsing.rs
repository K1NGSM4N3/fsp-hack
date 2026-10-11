//! Need and resume parsing on realistic Russian/English samples.

use catalog::{Grade, Industry, Specialization, WorkFormat};
use matching::{ParsedNeed, ParsedResume, parse_need, parse_resume};

const NOW: i64 = 1_791_158_400; // 2026-10-05

fn need(name: &str) -> ParsedNeed {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    parse_need(&std::fs::read_to_string(path).expect("fixture"))
}

fn resume(name: &str) -> ParsedResume {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    parse_resume(&std::fs::read_to_string(path).expect("fixture"), NOW)
}

fn spec(p: &Option<matching::SpecializationGuess>) -> (Specialization, f32) {
    let g = p.expect("specialization");
    (g.specialization, g.confidence)
}

#[test]
fn need_senior_backend_go() {
    let p = need("need_backend_go.txt");
    let (s, conf) = spec(&p.specialization);
    assert_eq!(s, Specialization::Backend);
    assert!(conf >= 0.8, "{conf}");
    assert_eq!(p.grade, Some(Grade::Senior));
    assert_eq!(
        p.must_skills,
        ["go", "postgresql", "redis", "kafka", "docker", "kubernetes"]
    );
    assert_eq!(p.nice_skills, ["clickhouse", "grpc"]);
    assert_eq!((p.salary_min, p.salary_max), (Some(350_000), Some(450_000)));
    assert_eq!(p.work_formats, [WorkFormat::Remote, WorkFormat::Hybrid]);
    assert_eq!(p.city.as_deref(), Some("Москва"));
    assert_eq!(p.industries, [Industry::Fintech]);
    assert_eq!(p.experience_years_min, Some(5.0));
}

#[test]
fn need_middle_frontend_react() {
    let p = need("need_frontend_react.txt");
    assert_eq!(spec(&p.specialization).0, Specialization::Frontend);
    assert_eq!(p.grade, Some(Grade::Middle));
    assert_eq!(p.must_skills, ["react", "typescript", "redux", "html"]);
    assert_eq!(p.nice_skills, ["nextjs"]);
    assert_eq!((p.salary_min, p.salary_max), (Some(200_000), Some(280_000)));
    assert_eq!(p.work_formats, [WorkFormat::Office, WorkFormat::Hybrid]);
    assert_eq!(p.city.as_deref(), Some("Санкт-Петербург"));
    assert_eq!(p.industries, [Industry::Ecommerce]);
}

#[test]
fn need_junior_qa() {
    let p = need("need_qa_junior.txt");
    assert_eq!(spec(&p.specialization).0, Specialization::Qa);
    assert_eq!(p.grade, Some(Grade::Junior));
    assert_eq!(
        p.must_skills,
        ["manual_testing", "test_design", "postman", "sql"]
    );
    assert_eq!(p.nice_skills, ["python", "pytest"]);
    assert_eq!((p.salary_min, p.salary_max), (Some(80_000), Some(110_000)));
    assert_eq!(p.work_formats, [WorkFormat::Office]);
    assert_eq!(p.city.as_deref(), Some("Казань"));
    assert_eq!(p.experience_years_min, Some(0.0));
}

#[test]
fn need_devops_remote() {
    let p = need("need_devops.txt");
    assert_eq!(spec(&p.specialization).0, Specialization::Devops);
    assert_eq!(p.grade, Some(Grade::Middle));
    assert_eq!(
        p.must_skills,
        [
            "kubernetes",
            "helm",
            "terraform",
            "ansible",
            "ci_cd",
            "prometheus",
            "grafana",
            "linux"
        ]
    );
    assert_eq!(p.nice_skills, ["kafka", "go", "python"]);
    assert_eq!((p.salary_min, p.salary_max), (None, Some(300_000)));
    assert_eq!(p.work_formats, [WorkFormat::Remote]);
    assert_eq!(p.city, None);
    assert_eq!(p.experience_years_min, Some(3.0));
    let n = p.into_need("DevOps").expect("need");
    assert_eq!(n.grades, [Grade::Middle]);
}

#[test]
fn resume_hh_backend() {
    let r = resume("resume_backend_hh.txt");
    assert_eq!(r.full_name.as_deref(), Some("Иванов Иван Иванович"));
    assert_eq!(r.email.as_deref(), Some("ivan.ivanov@mail.ru"));
    assert_eq!(r.phone.as_deref(), Some("+79161234567"));
    assert_eq!(r.telegram.as_deref(), Some("@ivan_dev"));
    assert_eq!(r.city.as_deref(), Some("Москва"));
    assert_eq!(r.relocation, Some(false));
    assert_eq!(r.grade_hint, Some(Grade::Senior));
    assert_eq!(r.experience_years, Some(7.2));
    assert_eq!(spec(&r.specialization_hint).0, Specialization::Backend);
    for s in [
        "python",
        "fastapi",
        "concurrency",
        "postgresql",
        "kafka",
        "redis",
        "django",
        "docker",
        "ci_cd",
        "kubernetes",
        "git",
        "linux",
        "sql",
        "rest",
    ] {
        assert!(
            r.skills.iter().any(|x| x == s),
            "missing {s}: {:?}",
            r.skills
        );
    }
    assert_eq!(
        r.roles,
        [
            "Senior Python-разработчик",
            "Ведущий backend-разработчик",
            "Python-разработчик"
        ]
    );
    assert_eq!(
        r.soft_skills,
        [
            "Менторство",
            "Коммуникабельность",
            "Ответственность",
            "Работа в команде",
            "Обучаемость"
        ]
    );
    assert_eq!(r.industries, [Industry::Fintech, Industry::Telecom]);
    assert_eq!(r.work_formats, [WorkFormat::Remote]);
    assert_eq!(r.salary_expectation, Some(300_000));
}

#[test]
fn resume_junior_frontend() {
    let r = resume("resume_frontend_junior.txt");
    assert_eq!(r.full_name.as_deref(), Some("Петрова Анна"));
    assert_eq!(r.phone.as_deref(), Some("+79215551234"));
    assert_eq!(r.email.as_deref(), Some("anna.petrova@yandex.ru"));
    assert_eq!(r.telegram.as_deref(), Some("@anna_front"));
    assert_eq!(r.city.as_deref(), Some("Санкт-Петербург"));
    assert_eq!(r.grade_hint, Some(Grade::Junior));
    // 03.2025–10.2026 (20 months) + 09.2024–02.2025 (6 months); education years excluded.
    assert_eq!(r.experience_years, Some(2.2));
    assert_eq!(spec(&r.specialization_hint).0, Specialization::Frontend);
    for s in [
        "react",
        "typescript",
        "redux",
        "html",
        "css",
        "javascript",
        "vue",
        "git",
        "unit_testing",
        "rest",
    ] {
        assert!(
            r.skills.iter().any(|x| x == s),
            "missing {s}: {:?}",
            r.skills
        );
    }
    assert_eq!(r.roles, ["Frontend-разработчик", "Стажёр-разработчик"]);
    assert_eq!(r.soft_skills, ["Внимательность к деталям", "Обучаемость"]);
    assert_eq!(r.industries, [Industry::Ecommerce, Industry::Outsourcing]);
}

#[test]
fn resume_english_qa() {
    let r = resume("resume_qa_english.txt");
    assert_eq!(r.full_name.as_deref(), Some("Smirnov Alexey"));
    assert_eq!(r.phone.as_deref(), Some("+79876543210"));
    assert_eq!(r.telegram.as_deref(), Some("@asmirnov_qa"));
    assert_eq!(r.city.as_deref(), Some("Казань"));
    assert_eq!(r.grade_hint, Some(Grade::Middle));
    // Jun 2023–Oct 2026 (41) + Feb 2022–May 2023 (16) months.
    assert_eq!(r.experience_years, Some(4.8));
    assert_eq!(spec(&r.specialization_hint).0, Specialization::Qa);
    for s in [
        "api_testing",
        "postman",
        "pytest",
        "selenium",
        "test_design",
        "manual_testing",
        "load_testing",
        "sql",
    ] {
        assert!(
            r.skills.iter().any(|x| x == s),
            "missing {s}: {:?}",
            r.skills
        );
    }
    assert_eq!(r.roles, ["QA Engineer", "Junior QA Engineer"]);
    assert_eq!(
        r.soft_skills,
        [
            "Работа в команде",
            "Внимательность к деталям",
            "Коммуникабельность"
        ]
    );
    assert_eq!(r.industries, [Industry::Ecommerce, Industry::Outsourcing]);
}

#[test]
fn resume_ml_with_fsp_mention() {
    let r = resume("resume_ml_fsp.txt");
    assert_eq!(r.full_name.as_deref(), Some("Кузнецов Дмитрий Сергеевич"));
    assert_eq!(r.phone.as_deref(), Some("+79130001122"));
    assert_eq!(r.email.as_deref(), Some("d.kuznetsov@ngs.ru"));
    assert_eq!(r.telegram.as_deref(), Some("@kuznetsov_ml"));
    assert_eq!(r.city.as_deref(), Some("Новосибирск"));
    assert_eq!(r.relocation, Some(true));
    assert_eq!(r.experience_years, Some(2.5));
    assert_eq!(r.grade_hint, Some(Grade::Middle));
    assert_eq!(spec(&r.specialization_hint).0, Specialization::Ml);
    for s in [
        "pytorch",
        "computer_vision",
        "nlp",
        "docker",
        "pandas",
        "numpy",
        "scikit_learn",
        "machine_learning",
        "sql",
        "statistics",
        "python",
        "algorithms",
    ] {
        assert!(
            r.skills.iter().any(|x| x == s),
            "missing {s}: {:?}",
            r.skills
        );
    }
    assert_eq!(
        r.roles,
        [
            "Data Scientist / ML-инженер",
            "ML-инженер",
            "Младший аналитик данных"
        ]
    );
    assert_eq!(
        r.soft_skills,
        ["Аналитическое мышление", "Самоорганизация", "Лидерство"]
    );
    assert_eq!(r.industries, [Industry::Healthtech]);
}
