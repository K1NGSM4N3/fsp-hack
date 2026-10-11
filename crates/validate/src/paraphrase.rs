//! Calibration of the anti-fake paraphrase signal (docs/validation/paraphrase.md): retold copies
//! of the demo vacancies against different vacancies, with the embedding cosine next to the
//! MinHash similarity of `matching::trust::similarity`.
//!
//! The originals repeat the demo seed (`crates/api/src/seed/corpus.rs`). Each retelling keeps the
//! facts of its original — role, stack, tasks, salary, format — in other words, the way a copied
//! vacancy is disguised. Hard negatives are different vacancies of the same specialization: the
//! seed pairs within one specialization and five written for this set (same role and stack,
//! another product).

use std::fmt::Write;

use embedder::{Embedder, TextKind};
use matching::{
    semantic,
    trust::{DUPLICATE_THRESHOLD, Signature},
};
use serde::Serialize;

struct Posting {
    title: &'static str,
    text: &'static str,
    spec: &'static str,
}

#[rustfmt::skip]
const ORIGINALS: [Posting; 12] = [
    Posting { spec: "backend", title: "Go-разработчик в команду платежей", text: "Ищем бэкенд-разработчика уровня Middle в команду процессинга. Стек: Go, PostgreSQL, Kafka, gRPC, Docker. Будет плюсом опыт с Kubernetes и ClickHouse. Задачи: идемпотентные платёжные API, outbox, шардирование. Зарплата 280–360 тыс. ₽, удалённо или гибрид в Москве." },
    Posting { spec: "backend", title: "Senior Java-разработчик антифрода", text: "Senior Backend: Java, Spring, PostgreSQL, Kafka, системный дизайн. Нагрузка десятки тысяч RPS, проектирование правил скоринга в реальном времени. От 5 лет опыта. 400 000 – 520 000 руб." },
    Posting { spec: "frontend", title: "Фронтенд-разработчик (React) в лайв-опс", text: "Мидл фронтендер на React и TypeScript: админка событий и витрина магазина внутри игры. Next.js, Redux, CSS-анимации. Желательно знание WebSocket. 220–290 тыс. ₽, офис в Петербурге или гибрид." },
    Posting { spec: "qa", title: "Junior QA-инженер", text: "Джун в тестирование мобильных игр: ручное тестирование, тест-дизайн, баг-репорты, Postman для API. Обучим автоматизации на Playwright. 90–130 тыс. ₽, офис." },
    Posting { spec: "devops", title: "DevOps/SRE инженер Kubernetes-платформы", text: "Middle+ DevOps: Kubernetes, Helm, Terraform, Prometheus, Linux, сети. Дежурства по графику, построение SLO и алертинга. Ansible будет плюсом. 260–340 тыс. ₽, удалённо." },
    Posting { spec: "devops", title: "Senior SRE", text: "Senior SRE для мультирегионального облака: Kubernetes, Linux internals, сети, CI/CD, наблюдаемость. Опыт инцидент-менеджмента обязателен. 380–480 тыс. ₽." },
    Posting { spec: "ml", title: "ML-инженер (компьютерное зрение)", text: "Middle ML Engineer: PyTorch, компьютерное зрение, метрики качества, pandas, numpy. Вывод моделей в прод вместе с бэкендом. 270–350 тыс. ₽, Казань или удалённо." },
    Posting { spec: "ml", title: "Junior Data Scientist", text: "Джун в прогноз загрузки клиник: Python, pandas, scikit-learn, статистика, SQL. Ментор и код-ревью. 110–150 тыс. ₽." },
    Posting { spec: "frontend", title: "Senior Frontend (TypeScript) кабинетов партнёров", text: "Синьор фронтенд: TypeScript, React, архитектура больших SPA, производительность, тесты. Руководство небольшой командой. 350–450 тыс. ₽, удалённо." },
    Posting { spec: "backend", title: "Python-разработчик маршрутизации", text: "Middle Python: FastAPI, PostgreSQL, Redis, алгоритмы на графах. Будет плюсом опыт с Kafka. 230–300 тыс. ₽, Екатеринбург или удалённо." },
    Posting { spec: "security", title: "Специалист по тестированию на проникновение", text: "Middle пентестер: веб-приложения, OWASP Top 10, сетевая безопасность, отчёты для заказчиков. Участие в CTF приветствуется. 250–330 тыс. ₽." },
    Posting { spec: "security", title: "AppSec-инженер", text: "Senior AppSec: безопасная разработка, ревью кода, криптография, SAST/DAST в CI. Опыт в финтехе будет плюсом. 380–470 тыс. ₽, удалённо." },
];

#[rustfmt::skip]
const RETOLD: [Posting; 12] = [
    Posting { spec: "backend", title: "Golang-инженер в процессинг платежей", text: "В команду, которая обрабатывает платежи, нужен мидл-бэкендер. Пишем на Golang, данные храним в Postgres, события гоняем через Kafka, сервисы общаются по gRPC и живут в контейнерах Docker. Знание k8s и ClickHouse — преимущество. Предстоит делать платёжные API с идемпотентностью, паттерн outbox и шардирование базы. Оплата от 280 до 360 тысяч рублей, можно работать из дома или приходить в московский офис." },
    Posting { spec: "backend", title: "Ведущий Java-инженер в систему противодействия мошенничеству", text: "Нужен опытный бэкенд-разработчик на Java и Spring со знанием PostgreSQL и Kafka и навыками проектирования систем. Сервис держит десятки тысяч запросов в секунду, вы будете проектировать скоринговые правила, которые срабатывают в реальном времени. Ждём опыт не меньше пяти лет. Зарплата 400–520 тысяч." },
    Posting { spec: "frontend", title: "React-разработчик для игровых событий", text: "Ищем фронтенд-разработчика среднего уровня: React, TypeScript, Next.js и Redux. Будете делать панель управления игровыми ивентами и внутриигровой магазин, оживлять интерфейс анимациями на CSS. Опыт с вебсокетами приветствуется. Предлагаем 220–290 тысяч рублей, офис в Санкт-Петербурге, возможен гибридный график." },
    Posting { spec: "qa", title: "Начинающий тестировщик мобильных игр", text: "Возьмём тестировщика без большого опыта проверять наши мобильные игры вручную: составлять тест-кейсы, оформлять баги, проверять API через Postman. Научим писать автотесты на Playwright. Зарплата от 90 до 130 тысяч, работа в офисе." },
    Posting { spec: "devops", title: "Инженер платформы на Kubernetes", text: "Ищем DevOps-инженера уровня выше среднего для нашей платформы: кластеры Kubernetes, чарты Helm, инфраструктура в Terraform, мониторинг на Prometheus, уверенный Linux и понимание сетей. Участие в дежурствах по расписанию, настройка SLO и оповещений. Знание Ansible — плюс. 260–340 тысяч рублей, полностью удалённо." },
    Posting { spec: "devops", title: "Ведущий инженер по надёжности облака", text: "Облако работает в нескольких регионах, и нам нужен опытный SRE: глубокое знание Linux и Kubernetes, сетей, пайплайнов CI/CD и средств наблюдаемости. Без опыта управления инцидентами не рассматриваем. Доход 380–480 тысяч рублей." },
    Posting { spec: "ml", title: "Инженер машинного обучения по распознаванию изображений", text: "Нужен ML-инженер среднего уровня для задач компьютерного зрения: обучение на PyTorch, подготовка данных в pandas и numpy, оценка качества моделей. Вместе с бэкенд-командой будете выводить модели в эксплуатацию. Зарплата 270–350 тысяч, офис в Казани или удалёнка." },
    Posting { spec: "ml", title: "Младший специалист по анализу данных", text: "Ищем начинающего дата-сайентиста, который будет предсказывать загруженность клиник. Нужны Python, pandas, scikit-learn, основы статистики и SQL. Дадим наставника и будем ревьюить код. Оплата 110–150 тысяч рублей." },
    Posting { spec: "frontend", title: "Ведущий фронтенд-разработчик партнёрского кабинета", text: "Нужен опытный фронтендер на TypeScript и React, который умеет проектировать крупные одностраничные приложения, ускорять их и покрывать тестами. Предстоит руководить маленькой командой. 350–450 тысяч рублей, удалённая работа." },
    Posting { spec: "backend", title: "Бэкенд-разработчик на Python для построения маршрутов", text: "Ищем питониста среднего уровня: сервисы на FastAPI, база PostgreSQL, кеш в Redis и графовые алгоритмы для маршрутизации. Опыт с Kafka будет преимуществом. 230–300 тысяч рублей, Екатеринбург или удалённо." },
    Posting { spec: "security", title: "Пентестер веб-приложений", text: "Нужен специалист среднего уровня по взлому веб-приложений: уязвимости из OWASP Top 10, безопасность сетей, подготовка отчётов клиентам. Будет плюсом опыт соревнований CTF. Зарплата 250–330 тысяч." },
    Posting { spec: "security", title: "Ведущий инженер по безопасности приложений", text: "Ищем опытного специалиста по защищённой разработке: проверка кода на уязвимости, прикладная криптография, встраивание SAST и DAST в пайплайны. Опыт в финансовых компаниях приветствуется. 380–470 тысяч, удалённо." },
];

/// Different vacancies with the role and stack of an original: `(original index, posting)`.
#[rustfmt::skip]
const LOOKALIKES: [(usize, Posting); 5] = [
    (0, Posting { spec: "backend", title: "Go-разработчик в сервис доставки", text: "Middle Go-разработчик в команду логистики: сервисы расчёта маршрутов курьеров, PostgreSQL, Kafka, Redis, Docker. Будет плюсом опыт с Kubernetes. 270–350 тыс. ₽, удалённо." }),
    (1, Posting { spec: "backend", title: "Senior Java-разработчик кредитного конвейера", text: "Senior Java/Spring в банк: кредитный конвейер, интеграции с бюро кредитных историй, PostgreSQL, Kafka, микросервисы. От 5 лет опыта. 400–500 тыс. ₽, гибрид в Москве." }),
    (2, Posting { spec: "frontend", title: "React-разработчик интернет-магазина", text: "Middle фронтенд на React и TypeScript: каталог, корзина и оформление заказа интернет-магазина. Next.js, Redux Toolkit, адаптивная вёрстка. 200–270 тыс. ₽, удалённо." }),
    (4, Posting { spec: "devops", title: "DevOps-инженер в облачную команду", text: "Middle DevOps: Kubernetes, Terraform, GitLab CI, Prometheus и Grafana, Linux. Перенос сервисов в облако, автоматизация инфраструктуры. 250–320 тыс. ₽, удалённо." }),
    (6, Posting { spec: "ml", title: "ML-инженер по NLP", text: "Middle ML: PyTorch, трансформеры, классификация текстов и поиск, pandas. Вывод моделей в прод. 260–340 тыс. ₽, удалённо." }),
];

#[derive(Serialize, Clone, Copy)]
pub struct PairScore {
    pub cosine: f32,
    /// MinHash estimate of shared word pairs; `None` when a text is too short to sign.
    pub minhash: Option<f32>,
}

#[derive(Serialize)]
pub struct Group {
    pub name: String,
    pub pairs: usize,
    pub cosine_min: f32,
    pub cosine_mean: f32,
    pub cosine_max: f32,
    /// Pairs at or above the MinHash duplicate threshold.
    pub minhash_flagged: usize,
    /// Pairs at or above the cosine threshold in use.
    pub flagged: usize,
    /// Every pair's cosine, highest first.
    pub cosines: Vec<f32>,
}

#[derive(Serialize)]
pub struct ParaphraseReport {
    pub model: String,
    /// Paraphrase threshold of `matching::semantic` for this model.
    pub threshold: Option<f32>,
    pub minhash_threshold: f32,
    pub groups: Vec<Group>,
    /// Probability that a retelling scores above a negative pair (ROC AUC).
    pub auc: f64,
    /// Share of retellings above the closest negative pair: caught with no false alarm here.
    pub recall_without_false_alarms: f64,
    /// Threshold without false alarms on this set: midway between the closest negative pair and
    /// the weakest retelling above it.
    pub suggested: Option<f32>,
}

fn text(p: &Posting) -> String {
    semantic::posting_text(p.title, p.text)
}

/// Exact cosine of unit vectors, as pgvector computes it for the paraphrase check.
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn score(a: &Posting, b: &Posting, qa: &[f32], qb: &[f32]) -> PairScore {
    let sig = |p: &Posting| {
        Signature::of_text(&matching::trust::similarity::vacancy_text(p.title, p.text))
    };
    PairScore {
        cosine: cosine(qa, qb),
        minhash: match (sig(a), sig(b)) {
            (Some(x), Some(y)) => Some(x.similarity(&y)),
            _ => None,
        },
    }
}

fn group(name: &str, scores: &[PairScore], threshold: Option<f32>) -> Group {
    let cos: Vec<f32> = scores.iter().map(|s| s.cosine).collect();
    Group {
        name: name.to_owned(),
        pairs: scores.len(),
        cosine_min: cos.iter().copied().fold(f32::INFINITY, f32::min),
        cosine_mean: cos.iter().sum::<f32>() / cos.len().max(1) as f32,
        cosine_max: cos.iter().copied().fold(f32::NEG_INFINITY, f32::max),
        minhash_flagged: scores
            .iter()
            .filter(|s| s.minhash.is_some_and(|m| m >= DUPLICATE_THRESHOLD))
            .count(),
        flagged: threshold.map_or(0, |t| cos.iter().filter(|&&c| c >= t).count()),
        cosines: {
            let mut sorted = cos.clone();
            sorted.sort_by(|a, b| b.total_cmp(a));
            sorted
        },
    }
}

pub fn run(model: &Embedder) -> Option<ParaphraseReport> {
    let all: Vec<&Posting> = ORIGINALS
        .iter()
        .chain(&RETOLD)
        .chain(LOOKALIKES.iter().map(|(_, p)| p))
        .collect();
    let texts: Vec<String> = all.iter().map(|p| text(p)).collect();
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let vectors = match model.embed(&refs, TextKind::Query) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("пересказы: эмбеддинги не посчитаны: {e}");
            return None;
        }
    };
    let q = &vectors;
    if q.len() != all.len() {
        return None;
    }
    let n = ORIGINALS.len();
    let lookalike = |i: usize| 2 * n + i;
    let retold: Vec<PairScore> = (0..n)
        .map(|i| score(&ORIGINALS[i], &RETOLD[i], &q[i], &q[n + i]))
        .collect();
    let mut same_spec = Vec::new();
    let mut other_spec = Vec::new();
    for i in 0..n {
        for j in i + 1..n {
            let s = score(&ORIGINALS[i], &ORIGINALS[j], &q[i], &q[j]);
            if ORIGINALS[i].spec == ORIGINALS[j].spec {
                same_spec.push(s);
            } else {
                other_spec.push(s);
            }
        }
    }
    let lookalikes: Vec<PairScore> = LOOKALIKES
        .iter()
        .enumerate()
        .map(|(k, (i, p))| score(&ORIGINALS[*i], p, &q[*i], &q[lookalike(k)]))
        .chain(
            LOOKALIKES
                .iter()
                .enumerate()
                .map(|(k, (i, p))| score(&RETOLD[*i], p, &q[n + *i], &q[lookalike(k)])),
        )
        .collect();
    let threshold = semantic::calibration(model.id()).map(|c| c.paraphrase);
    let hardest = same_spec
        .iter()
        .chain(&lookalikes)
        .chain(&other_spec)
        .map(|s| s.cosine)
        .fold(f32::NEG_INFINITY, f32::max);
    let negatives: Vec<f32> = same_spec
        .iter()
        .chain(&lookalikes)
        .chain(&other_spec)
        .map(|s| s.cosine)
        .collect();
    let mut wins = 0.0;
    for p in &retold {
        for &n in &negatives {
            wins += if p.cosine > n {
                1.0
            } else if p.cosine == n {
                0.5
            } else {
                0.0
            };
        }
    }
    let auc = wins / (retold.len() * negatives.len()).max(1) as f64;
    let caught = retold.iter().filter(|p| p.cosine > hardest).count();
    Some(ParaphraseReport {
        model: model.id().to_owned(),
        threshold,
        minhash_threshold: DUPLICATE_THRESHOLD,
        groups: vec![
            group("Пересказ вакансии (копия)", &retold, threshold),
            group(
                "Похожая вакансия той же специализации (не копия)",
                &lookalikes,
                threshold,
            ),
            group("Разные вакансии одной специализации", &same_spec, threshold),
            group("Разные специализации", &other_spec, threshold),
        ],
        auc,
        recall_without_false_alarms: caught as f64 / retold.len().max(1) as f64,
        suggested: retold
            .iter()
            .map(|p| p.cosine)
            .filter(|&c| c > hardest)
            .reduce(f32::min)
            .map(|above| (above + hardest) / 2.0),
    })
}

pub fn markdown(r: &ParaphraseReport) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "## Антифейк: пересказанные копии вакансий\n");
    let _ = writeln!(
        s,
        "12 демо-вакансий и для каждой — пересказ теми же фактами другими словами (так маскируют скопированную \
         вакансию). Отрицательные пары: разные демо-вакансии одной специализации, пять похожих вакансий с той же \
         ролью и стеком, но другим продуктом (сравниваются и с оригиналом, и с пересказом), и вакансии разных \
         специализаций. Модель `{}`; текст — заголовок и описание, как у вакансии в API. MinHash — подпись пар \
         соседних слов (порог {:.2}), которая уже работает для дословных копий.\n",
        r.model, r.minhash_threshold
    );
    let _ = writeln!(
        s,
        "| Пары | Сколько | Косинус: мин. | средн. | макс. | MinHash ≥ {:.2} | Косинус ≥ порога |\n|---|---|---|---|---|---|---|",
        r.minhash_threshold
    );
    for g in &r.groups {
        let _ = writeln!(
            s,
            "| {} | {} | {:.3} | {:.3} | {:.3} | {} | {} |",
            g.name,
            g.pairs,
            g.cosine_min,
            g.cosine_mean,
            g.cosine_max,
            g.minhash_flagged,
            g.flagged
        );
    }
    let _ = writeln!(
        s,
        "\nROC AUC пересказов против всех отрицательных пар — {:.3}; выше самой близкой отрицательной пары — {:.0} % пересказов.",
        r.auc,
        r.recall_without_false_alarms * 100.0
    );
    match r.threshold {
        Some(t) => {
            let _ = writeln!(
                s,
                "\nПорог пересказа в `matching::semantic`: косинус ≥ {t:.3}."
            );
        }
        None => {
            let _ = writeln!(s, "\nДля этой модели порог пересказа не задан.");
        }
    }
    if let Some(t) = r.suggested {
        let _ = writeln!(
            s,
            "Порог без ложных срабатываний на этом наборе — середина между самой близкой отрицательной парой и \
             ближайшим пересказом выше неё: {t:.3}. Набор маленький, поэтому сигнал ничего не блокирует и квоту \
             не снижает — он только отправляет вакансию модератору (README.md, раздел 10.1)."
        );
    }
    s
}
