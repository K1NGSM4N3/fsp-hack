//! What the demo employer shows: one need per specialization for the T-4 walkthrough
//! (vacancy text → test plan → shortlist), and the corporate-domain check of every demo company.
//! No randomness, so the rest of the seed stays byte-for-byte the same.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

use super::corpus;
use crate::error::ApiResult;

/// Index of the demo employer (`employer@demo.fsp`) in [`corpus::COMPANIES`].
pub const DEMO_COMPANY: usize = 0;

pub struct DemoNeed {
    pub title: &'static str,
    pub description: &'static str,
    pub specialization: &'static str,
    pub grade: &'static str,
    pub must: &'static [&'static str],
    pub nice: &'static [&'static str],
    pub salary: (i64, i64),
    pub work_formats: &'static [&'static str],
}

/// Newest first: the needs list opens with the Go need the end-to-end demo walks through.
pub const DEMO_NEEDS: &[DemoNeed] = &[
    DemoNeed {
        title: "Go-разработчик в команду платежей",
        description: "Ищем бэкенд-разработчика уровня Middle в команду процессинга. Стек: Go, PostgreSQL, Kafka, gRPC, Docker. \
            Будет плюсом опыт с Kubernetes и ClickHouse. Задачи: идемпотентные платёжные API, outbox, шардирование. \
            Зарплата 280–360 тыс. ₽, удалённо или гибрид в Москве.",
        specialization: "backend",
        grade: "middle",
        must: &["go", "postgresql", "kafka"],
        nice: &["grpc", "docker"],
        salary: (280_000, 360_000),
        work_formats: &["remote"],
    },
    DemoNeed {
        title: "Фронтенд-разработчик в личный кабинет мерчанта",
        description: "Делаем кабинет, в котором малый бизнес видит платежи, возвраты и выплаты. Нужен Middle на React и TypeScript: \
            таблицы на десятки тысяч строк, фильтры, графики оборота, формы подключения эквайринга. Next.js, своя дизайн-система \
            на CSS-переменных, тесты на компоненты. Работаем в паре с дизайнером и бэкендом на Go.",
        specialization: "frontend",
        grade: "middle",
        must: &["react", "typescript", "nextjs"],
        nice: &["css", "redux", "unit_testing"],
        salary: (230_000, 300_000),
        work_formats: &["remote", "hybrid"],
    },
    DemoNeed {
        title: "DevOps-инженер платформы процессинга",
        description: "Процессинг работает в Kubernetes в двух дата-центрах. Ищем DevOps-инженера уровня Middle: поддерживать кластеры \
            и Helm-чарты, описывать инфраструктуру в Terraform, развивать CI/CD, настраивать алертинг в Prometheus. \
            Дежурства по графику с доплатой, удалённо.",
        specialization: "devops",
        grade: "middle",
        must: &["kubernetes", "terraform", "ci_cd"],
        nice: &["helm", "prometheus", "linux"],
        salary: (270_000, 350_000),
        work_formats: &["remote"],
    },
    DemoNeed {
        title: "QA-инженер по автоматизации API выплат",
        description: "Тестируем API выплат и подключения мерчантов. Нужен QA уровня Middle: тест-дизайн, автотесты на pytest, \
            проверки в Postman, SQL для сверки данных в базе. Будет плюсом опыт нагрузочного тестирования. \
            Релиз каждую неделю, автотесты запускаются в CI на каждый merge request.",
        specialization: "qa",
        grade: "middle",
        must: &["api_testing", "pytest", "test_design"],
        nice: &["postman", "sql", "load_testing"],
        salary: (180_000, 240_000),
        work_formats: &["remote", "hybrid"],
    },
    DemoNeed {
        title: "ML-инженер в антифрод",
        description: "Модели антифрода оценивают каждую транзакцию за 50 мс. Ищем ML-инженера уровня Middle: признаки из истории \
            платежей, градиентный бустинг и scikit-learn, метрики качества при сильном дисбалансе классов, A/B-проверка \
            новых моделей. Python, pandas, SQL; модели выкатываем вместе с бэкендом.",
        specialization: "ml",
        grade: "middle",
        must: &["python", "scikit_learn", "ml_metrics"],
        nice: &["pandas", "sql", "statistics"],
        salary: (280_000, 360_000),
        work_formats: &["remote", "hybrid"],
    },
    DemoNeed {
        title: "AppSec-инженер платёжного API",
        description: "Отвечаем за безопасность платёжного API и личных кабинетов. Ищем AppSec-инженера уровня Middle: ревью кода \
            на уязвимости из OWASP Top 10, пентесты веб-приложений перед релизами, моделирование угроз для новых сервисов, \
            помощь разработчикам с криптографией и хранением секретов.",
        specialization: "security",
        grade: "middle",
        must: &["appsec", "owasp", "pentest"],
        nice: &["cryptography", "network_security", "python"],
        salary: (270_000, 350_000),
        work_formats: &["remote", "hybrid"],
    },
];

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

/// The demo employer's needs, an hour apart so the list order never depends on insert timing.
pub async fn seed_demo_needs(
    tx: &mut PgConnection,
    company: Uuid,
    now: DateTime<Utc>,
) -> ApiResult<Vec<Uuid>> {
    let industries = [corpus::COMPANIES[DEMO_COMPANY].industry.to_owned()];
    let mut ids = Vec::with_capacity(DEMO_NEEDS.len());
    for (i, n) in DEMO_NEEDS.iter().enumerate() {
        let id = Uuid::now_v7();
        db::needs::insert_need(
            &mut *tx,
            id,
            company,
            &db::needs::NeedFields {
                title: n.title,
                description: n.description,
                specialization: n.specialization,
                grades: &[n.grade.to_owned()],
                allow_adjacent_grades: true,
                must_skills: &strings(n.must),
                nice_skills: &strings(n.nice),
                salary_min: n.salary.0,
                salary_max: n.salary.1,
                work_formats: &strings(n.work_formats),
                city: None,
                industries: &industries,
            },
        )
        .await?;
        db::needs::set_need_created_at(&mut *tx, id, now - Duration::hours(i as i64)).await?;
        ids.push(id);
    }
    Ok(ids)
}

/// Seeded accounts are verified, so the domain check runs on the plain emails from the corpus.
pub async fn verify_domains(tx: &mut PgConnection, companies: &[Uuid]) -> ApiResult<()> {
    for (c, &id) in corpus::COMPANIES.iter().zip(companies) {
        let check = matching::trust::check_domain(c.owner, true, Some(c.website));
        db::trust::set_verified_domain(&mut *tx, id, check.verified_domain()).await?;
    }
    Ok(())
}
