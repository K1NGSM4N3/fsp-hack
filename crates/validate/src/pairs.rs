//! `validate pairs`: runs a set of vacancies, candidates and expert labels through the production
//! matching (`matching::pairs`) and writes a Markdown and a JSON report.

use std::collections::HashMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use catalog::WorkFormat;
use embedder::TextKind;
use matching::{
    pairs::{
        CandidateSummary, Mean, MeanMetrics, Metrics, Origin, PairsError, PairsInput, PairsReport,
        PairsVectors, TextSource, VacancyReport,
    },
    semantic,
};
use serde::{Deserialize, Serialize};

use crate::{
    csv::{self, CsvError},
    model::Choice,
};

#[derive(Debug, thiserror::Error)]
pub enum PairsCliError {
    #[error(
        "usage: validate pairs --input FILE.json|DIR [--k N] [--now YYYY-MM-DD] [--out REPORT.md] [--model NAME|none] [--models DIR]\n\
         DIR — каталог с vacancies.csv, candidates.csv и необязательным labels.csv"
    )]
    Usage,
    #[error("{path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Json {
        path: String,
        source: serde_json::Error,
    },
    #[error(transparent)]
    Csv(#[from] CsvError),
    #[error("Ошибка во входных данных — {0}")]
    Input(#[from] PairsError),
    #[error("{path}: {source}")]
    Write {
        path: String,
        source: std::io::Error,
    },
    #[error("{0}: отчёт перезаписал бы входной файл — укажите другое имя в --out")]
    WouldOverwriteInput(String),
}

struct Args {
    input: PathBuf,
    k: Option<usize>,
    now: Option<String>,
    out: Option<PathBuf>,
    model: Choice,
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Args, PairsCliError> {
    let (mut input, mut k, mut now, mut out) = (None, None, None, None);
    let mut model = Choice::default();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(PairsCliError::Usage);
        match arg.as_str() {
            "--input" => input = Some(PathBuf::from(value()?)),
            "--k" => k = Some(value()?.parse().map_err(|_| PairsCliError::Usage)?),
            "--now" => now = Some(value()?),
            "--out" => out = Some(PathBuf::from(value()?)),
            "--model" => model.name = value()?,
            "--models" => model.dir = PathBuf::from(value()?),
            _ => return Err(PairsCliError::Usage),
        }
    }
    Ok(Args {
        input: input.ok_or(PairsCliError::Usage)?,
        k,
        now,
        out,
        model,
    })
}

/// The vectors a report was computed with, as `i8` codes: what the ranking used exactly, so the
/// report can be checked again without the model. Written next to the report.
#[derive(Serialize, Deserialize)]
pub struct StoredVectors {
    pub model: String,
    pub vacancies: Vec<Option<StoredVector>>,
    pub candidates: Vec<Option<StoredVector>>,
}

#[derive(Serialize, Deserialize)]
pub struct StoredVector {
    pub max: f32,
    pub codes: Vec<i8>,
}

impl StoredVectors {
    fn of(v: &PairsVectors) -> Self {
        let store = |list: &[Option<Vec<f32>>]| -> Vec<Option<StoredVector>> {
            list.iter()
                .map(|x| {
                    x.as_deref()
                        .and_then(semantic::to_codes)
                        .map(|(codes, max)| StoredVector { max, codes })
                })
                .collect()
        };
        StoredVectors {
            model: v.model.clone(),
            vacancies: store(&v.vacancies),
            candidates: store(&v.candidates),
        }
    }

    #[cfg(test)]
    pub fn vectors(&self) -> PairsVectors {
        let load = |list: &[Option<StoredVector>]| -> Vec<Option<Vec<f32>>> {
            list.iter()
                .map(|x| x.as_ref().map(|s| semantic::from_codes(&s.codes, s.max)))
                .collect()
        };
        PairsVectors {
            model: self.model.clone(),
            vacancies: load(&self.vacancies),
            candidates: load(&self.candidates),
        }
    }
}

/// Embeds the input with the local model; `Err` carries the reason for the report.
fn embed(input: &PairsInput, choice: &Choice) -> Result<PairsVectors, String> {
    let model = choice.load().ok_or_else(|| {
        format!(
            "модель {} не найдена в {}",
            choice.name,
            choice.dir.display()
        )
    })?;
    let (vacancies, candidates) = matching::pairs::texts(input);
    let run = |texts: &[String], kind| -> Result<Vec<Option<Vec<f32>>>, String> {
        let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
        model
            .embed(&refs, kind)
            .map(|v| v.into_iter().map(Some).collect())
            .map_err(|e| format!("эмбеддинги не посчитаны: {e}"))
    };
    Ok(PairsVectors {
        model: model.id().to_owned(),
        vacancies: run(&vacancies, TextKind::Query)?,
        candidates: run(&candidates, TextKind::Passage)?,
    })
}

fn read_input(path: &Path) -> Result<PairsInput, PairsCliError> {
    if path.is_dir() {
        let mut warnings = Vec::new();
        let input = csv::load_dir(path, &mut warnings)?;
        for w in warnings {
            eprintln!("{w}");
        }
        return Ok(input);
    }
    let text = std::fs::read_to_string(path).map_err(|source| PairsCliError::Read {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| PairsCliError::Json {
        path: path.display().to_string(),
        source,
    })
}

fn write(path: &Path, content: &str) -> Result<(), PairsCliError> {
    std::fs::write(path, content).map_err(|source| PairsCliError::Write {
        path: path.display().to_string(),
        source,
    })
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

pub fn run(args: impl Iterator<Item = String>) -> Result<(), PairsCliError> {
    let args = parse_args(args)?;
    if let Some(out) = &args.out {
        for target in [out.clone(), out.with_extension("json")] {
            if same_file(&target, &args.input) {
                return Err(PairsCliError::WouldOverwriteInput(
                    target.display().to_string(),
                ));
            }
        }
    }
    let mut input = read_input(&args.input)?;
    if args.k.is_some() {
        input.k = args.k;
    }
    if args.now.is_some() {
        input.now = args.now;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    input.check()?;
    let vectors = if args.model.name == "none" {
        Err("эмбеддинги выключены (--model none)".to_owned())
    } else {
        embed(&input, &args.model)
    };
    let source = match &vectors {
        Ok(v) => TextSource::Meaning(v),
        Err(reason) => TextSource::Keywords(reason.clone()),
    };
    let report = matching::pairs::evaluate(&input, now, source)?;
    let md = markdown(&report);
    match args.out {
        Some(out) => {
            let json_path = out.with_extension("json");
            let vectors_path = out.with_extension("vectors.json");
            write(&out, &md)?;
            match &vectors {
                Ok(v) => {
                    let stored =
                        serde_json::to_string(&StoredVectors::of(v)).map_err(|source| {
                            PairsCliError::Json {
                                path: vectors_path.display().to_string(),
                                source,
                            }
                        })?;
                    write(&vectors_path, &(stored + "\n"))?;
                }
                Err(_) if vectors_path.exists() => {
                    std::fs::remove_file(&vectors_path).map_err(|source| PairsCliError::Write {
                        path: vectors_path.display().to_string(),
                        source,
                    })?;
                }
                Err(_) => {}
            }
            let json =
                serde_json::to_string_pretty(&report).map_err(|source| PairsCliError::Json {
                    path: json_path.display().to_string(),
                    source,
                })?;
            write(&json_path, &(json + "\n"))?;
            print!("{}", summary(&report));
            println!("\nОтчёт: {}, {}", out.display(), json_path.display());
        }
        None => print!("{md}"),
    }
    Ok(())
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

fn mean(m: Mean) -> String {
    format!("{:.3} ± {:.3}", m.mean, m.ci95)
}

fn origin(o: Origin) -> &'static str {
    match o {
        Origin::Input => "из входных данных",
        Origin::Text => "из текста",
        Origin::Missing => "не найдено",
    }
}

fn format_name(f: WorkFormat) -> &'static str {
    match f {
        WorkFormat::Office => "офис",
        WorkFormat::Hybrid => "гибрид",
        WorkFormat::Remote => "удалённо",
    }
}

fn rub(v: i64) -> String {
    let digits = v.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

fn metrics_row(s: &mut String, name: &str, m: &MeanMetrics) {
    let _ = writeln!(
        s,
        "| {name} | {} | {} | {} | {} | {} | {:.0} % |",
        mean(m.precision_at_5),
        mean(m.precision_at_k),
        mean(m.ndcg_at_k),
        mean(m.average_precision),
        mean(m.reciprocal_rank),
        m.judged_at_k.mean * 100.0
    );
}

fn pair(ours: Option<Metrics>, base: Option<Metrics>, f: fn(&Metrics) -> f64) -> String {
    match (ours, base) {
        (Some(o), Some(b)) => format!("{:.2} / {:.2}", f(&o), f(&b)),
        _ => "—".into(),
    }
}

/// Headline: metrics, per-vacancy table and coverage.
pub fn summary(r: &PairsReport) -> String {
    let mut s = String::new();
    let k = r.k;
    let c = &r.coverage;
    let _ = writeln!(s, "# Проверка подбора на своих данных\n");
    let _ = writeln!(
        s,
        "Дата расчёта {} · вакансий {} · кандидатов {} · оценок {} (размечено вакансий: {}) · k = {k}. \
         Релевантный кандидат — оценка ≥ {}; неразмеченные пары считаются нерелевантными.\n",
        r.now, c.vacancies, c.candidates, c.labels, c.labeled_vacancies, r.relevant_from
    );
    let _ = match (&r.text.model, &r.text.fallback) {
        (Some(model), _) => writeln!(
            s,
            "Близость описаний — по смыслу: эмбеддинги `{model}`, посчитанные локально (векторы — в `*.vectors.json` рядом с отчётом).\n"
        ),
        (None, Some(reason)) => writeln!(
            s,
            "Близость описаний — по ключевым словам: {reason}. С моделью (`make model`) она считается по смыслу.\n"
        ),
        (None, None) => writeln!(s, "Близость описаний — по ключевым словам.\n"),
    };
    let _ = writeln!(s, "## Наш подбор и поиск по ключевым словам\n");
    match &r.metrics {
        Some(m) => {
            let _ = writeln!(
                s,
                "Среднее по {} вакансиям, где в разметке есть хотя бы один релевантный кандидат; \
                 ± — полуширина 95 % доверительного интервала. Базовая линия — та же, что в \
                 `matching.md`: фильтр по заявленному стажу под грейд вакансии и число навыков \
                 вакансии среди ключевых слов резюме.\n",
                m.vacancies
            );
            let _ = writeln!(
                s,
                "| Ранжирование | Precision@5 | Precision@{k} | nDCG@{k} | MAP | MRR | Размечено в топ-{k} |\n|---|---|---|---|---|---|---|"
            );
            metrics_row(&mut s, "Наш подбор", &m.ours);
            metrics_row(
                &mut s,
                "Ключевые слова резюме + заявленный стаж",
                &m.baseline,
            );
        }
        None => {
            let _ = writeln!(
                s,
                "Разметки с релевантными кандидатами нет — метрики не считаются, ниже выдача для ручной проверки."
            );
        }
    }
    let _ = writeln!(
        s,
        "\n### По вакансиям\n\n| Вакансия | Оценок (релевантных) | Прошли фильтры | P@{k}: наш / базовая | nDCG@{k}: наш / базовая | AP: наш / базовая |\n|---|---|---|---|---|---|"
    );
    for v in &r.vacancies {
        let _ = writeln!(
            s,
            "| {} · {} | {} ({}) | {} | {} | {} | {} |",
            cell(&v.id),
            cell(&v.title),
            v.labels,
            v.relevant_labels,
            v.passed_filters,
            pair(v.ours, v.baseline, |m| m.precision_at_k),
            pair(v.ours, v.baseline, |m| m.ndcg_at_k),
            pair(v.ours, v.baseline, |m| m.average_precision),
        );
    }

    let _ = writeln!(
        s,
        "\n## Покрытие: что взято из данных, что распознано\n\n| Поле | Из входных данных | Распознано из текста | Не найдено |\n|---|---|---|---|"
    );
    for (name, o) in [
        ("Специализация вакансии", c.vacancy_specialization),
        ("Грейд вакансии", c.vacancy_grades),
        ("Навыки вакансии", c.vacancy_skills),
        ("Специализация кандидата", c.candidate_specialization),
        ("Грейд кандидата (у кого есть категория)", c.candidate_grade),
    ] {
        let _ = writeln!(s, "| {name} | {} | {} | {} |", o.input, o.text, o.missing);
    }
    let _ = writeln!(
        s,
        "\n- Категория, распознанная из резюме, помечается «не подтверждена тестом»; если грейд не найден, принимается Middle с пометкой в объяснении."
    );
    let _ = writeln!(
        s,
        "- Результат теста передан у {} из {} кандидатов; остальным взято нейтральное значение {:.0} % — середина диапазона грейда, в объяснении это сказано.",
        c.candidates_with_test,
        c.candidates,
        r.neutral_test_strength * 100.0
    );
    let _ = writeln!(
        s,
        "- Достижения ФСП переданы у {} кандидатов.",
        c.candidates_with_fsp
    );
    let list = |ids: &[String]| {
        if ids.is_empty() {
            "нет".to_owned()
        } else {
            format!("{} — {}", ids.len(), cell(&ids.join(", ")))
        }
    };
    let _ = writeln!(
        s,
        "- Без категории (специализация не распознана, в наш подбор не попадают): {}.",
        list(&c.candidates_without_category)
    );
    let _ = writeln!(
        s,
        "- Вакансии с неуверенным разбором (специализация с уверенностью ниже 0,5 или не найдена, нет навыков): {}.",
        list(&c.vacancies_low_confidence)
    );
    s
}

fn need_line(v: &VacancyReport) -> String {
    let n = &v.need;
    let mut parts: Vec<String> = Vec::new();
    parts.push(match n.specialization {
        Some(spec) => match n.specialization_confidence {
            Some(conf) => format!(
                "{} ({}, уверенность {conf:.2})",
                spec.name_ru(),
                origin(n.specialization_origin)
            ),
            None => format!("{} ({})", spec.name_ru(), origin(n.specialization_origin)),
        },
        None => "специализация не распознана".into(),
    });
    if n.grades.is_empty() {
        parts.push("грейд любой".into());
    } else {
        let grades: Vec<&str> = n.grades.iter().map(|g| g.name_ru()).collect();
        parts.push(format!(
            "грейд {} ({}){}",
            grades.join("/"),
            origin(n.grades_origin),
            if n.allow_adjacent_grades {
                ", соседний допускается"
            } else {
                ""
            }
        ));
    }
    if !n.must_skills.is_empty() {
        parts.push(format!("обязательные: {}", n.must_skills.join(", ")));
    }
    if !n.nice_skills.is_empty() {
        parts.push(format!("желательные: {}", n.nice_skills.join(", ")));
    }
    match (n.salary_min, n.salary_max) {
        (Some(a), Some(b)) => parts.push(format!("вилка {}–{} ₽", rub(a), rub(b))),
        (None, Some(b)) => parts.push(format!("до {} ₽", rub(b))),
        (Some(a), None) => parts.push(format!("от {} ₽", rub(a))),
        (None, None) => {}
    }
    if !n.work_formats.is_empty() {
        let f: Vec<&str> = n.work_formats.iter().map(|f| format_name(*f)).collect();
        parts.push(f.join(", "));
    }
    if let Some(city) = &n.city {
        parts.push(city.clone());
    }
    parts.join(" · ")
}

fn category_cell(c: Option<&CandidateSummary>) -> String {
    let Some(c) = c else {
        return "—".into();
    };
    let Some(cat) = c.category else {
        return "нет".into();
    };
    let src = match (c.specialization_origin, c.grade_origin) {
        (Origin::Input, Origin::Input) => "данные",
        (_, Origin::Missing) => "грейд не найден",
        _ => "резюме",
    };
    let test = c
        .test_result
        .map_or_else(|| "без теста".to_owned(), |t| format!("тест {t:.0}"));
    format!("{} · {src} · {test}", cat.key())
}

fn relevance(r: Option<u8>) -> String {
    r.map_or_else(|| "—".into(), |g| g.to_string())
}

pub fn markdown(r: &PairsReport) -> String {
    let mut s = summary(r);
    let by_id: HashMap<&str, &CandidateSummary> =
        r.candidates.iter().map(|c| (c.id.as_str(), c)).collect();
    let k = r.k;
    let _ = writeln!(s, "\n## Выдача по вакансиям");
    for v in &r.vacancies {
        let _ = writeln!(s, "\n### {} · {}\n", cell(&v.id), cell(&v.title));
        let _ = writeln!(s, "Распознано: {}.\n", cell(&need_line(v)));
        let _ = writeln!(
            s,
            "Прошли жёсткие фильтры: {} из {}. Оценок: {}, релевантных: {}.",
            v.passed_filters, r.coverage.candidates, v.labels, v.relevant_labels
        );
        for note in &v.notes {
            let _ = writeln!(s, "\n> {}", cell(note));
        }
        let _ = writeln!(
            s,
            "\n**Наш подбор, топ-{k}**\n\n| # | Кандидат | Балл | Оценка | Категория | Почему | Пробелы |\n|---|---|---|---|---|---|---|"
        );
        if v.top.is_empty() {
            let _ = writeln!(s, "| — | никто не прошёл фильтры | | | | | |");
        }
        for p in &v.top {
            let reasons: Vec<&str> = p
                .breakdown
                .reasons
                .iter()
                .take(2)
                .map(String::as_str)
                .collect();
            let gaps: Vec<&str> = p
                .breakdown
                .gaps
                .iter()
                .take(2)
                .map(String::as_str)
                .collect();
            let _ = writeln!(
                s,
                "| {} | {} | {:.1} | {} | {} | {} | {} |",
                p.rank,
                cell(&p.candidate_id),
                p.score,
                relevance(p.relevance),
                category_cell(by_id.get(p.candidate_id.as_str()).copied()),
                cell(&reasons.join("; ")),
                cell(&gaps.join("; ")),
            );
        }
        let _ = writeln!(
            s,
            "\n**Ключевые слова резюме + стаж, топ-{k}**\n\n| # | Кандидат | Совпало навыков вакансии | Стаж, лет | Оценка |\n|---|---|---|---|---|"
        );
        let total = v.need.must_skills.len() + v.need.nice_skills.len();
        for p in &v.baseline_top {
            let _ = writeln!(
                s,
                "| {} | {} | {} из {total}{} | {} | {} |",
                p.rank,
                cell(&p.candidate_id),
                p.keywords.len(),
                if p.keywords.is_empty() {
                    String::new()
                } else {
                    format!(": {}", cell(&p.keywords.join(", ")))
                },
                p.experience_years
                    .map_or_else(|| "—".to_owned(), |y| format!("{y:.1}")),
                relevance(p.relevance),
            );
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn example_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/validation/pairs-example")
    }

    fn example() -> PairsInput {
        let text = std::fs::read_to_string(example_dir().join("input.json")).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    /// Numbers may differ in the last bits across platforms' libm; structure and text may not.
    fn assert_close(got: &Value, want: &Value, path: &str) {
        match (got, want) {
            (Value::Number(a), Value::Number(b)) => {
                let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
                assert!(
                    (a - b).abs() <= 1e-4 * b.abs().max(1.0),
                    "{path}: {a} != {b}"
                );
            }
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path}: length");
                for (i, (x, y)) in a.iter().zip(b).enumerate() {
                    assert_close(x, y, &format!("{path}[{i}]"));
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                assert_eq!(
                    a.keys().collect::<Vec<_>>(),
                    b.keys().collect::<Vec<_>>(),
                    "{path}: keys"
                );
                for (key, x) in a {
                    assert_close(x, &b[key], &format!("{path}.{key}"));
                }
            }
            _ => assert_eq!(got, want, "{path}"),
        }
    }

    /// The committed report was computed with embeddings when its vectors are committed next to
    /// it; the check then uses them, so it needs no model.
    fn example_source() -> Option<PairsVectors> {
        let path = example_dir().join("report.vectors.json");
        let text = std::fs::read_to_string(path).ok()?;
        Some(
            serde_json::from_str::<StoredVectors>(&text)
                .unwrap()
                .vectors(),
        )
    }

    #[test]
    fn example_reproduces_committed_report() {
        let vectors = example_source();
        let source = match &vectors {
            Some(v) => TextSource::Meaning(v),
            None => TextSource::Keywords("эмбеддинги выключены (--model none)".into()),
        };
        let report = matching::pairs::evaluate(&example(), 0, source).unwrap();
        let got = serde_json::to_value(&report).unwrap();
        let committed = std::fs::read_to_string(example_dir().join("report.json")).unwrap();
        assert_close(&got, &serde_json::from_str(&committed).unwrap(), "$");
        let md = std::fs::read_to_string(example_dir().join("report.md")).unwrap();
        assert_eq!(markdown(&report), md);
    }

    #[test]
    fn csv_example_equals_json_example() {
        let mut warnings = Vec::new();
        let from_csv = csv::load_dir(&example_dir().join("csv"), &mut warnings).unwrap();
        let json = example();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(from_csv.vacancies, json.vacancies);
        assert_eq!(from_csv.candidates, json.candidates);
        assert_eq!(from_csv.labels, json.labels);
    }

    #[test]
    fn args() {
        let a = parse_args(
            [
                "--input", "x.json", "--k", "5", "--out", "r.md", "--model", "none",
            ]
            .into_iter()
            .map(String::from),
        )
        .unwrap();
        assert_eq!((a.k, a.out.as_deref()), (Some(5), Some(Path::new("r.md"))));
        assert_eq!(a.model.name, "none");
        assert!(parse_args(["--k", "5"].into_iter().map(String::from)).is_err());
        assert!(parse_args(["--input"].into_iter().map(String::from)).is_err());
    }
}
