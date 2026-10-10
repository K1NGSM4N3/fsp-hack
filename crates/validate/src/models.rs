//! `validate models`: the embedding models side by side on the same synthetic matching set and
//! the same paraphrase set (docs/validation/embeddings.md). Speed is measured in this process on
//! this machine; the memory of the service is measured separately (README.md, ML section).

use std::fmt::Write;

use embedder::model::MODELS;
use serde::Serialize;

use crate::{matching_eval, model::Choice, paraphrase};

#[derive(Serialize)]
pub struct ModelRow {
    pub model: String,
    pub dim: usize,
    pub load_ms: f64,
    pub embed_ms_per_text: Option<f64>,
    /// Correlation of the cosine with the real must-have coverage (holdout needs).
    pub meaning_signal: Option<f64>,
    pub keywords_signal: Option<f64>,
    /// Ranking by the cosine alone: P@10, nDCG@10, MAP.
    pub meaning_only: Option<[f64; 3]>,
    /// Full matching with closeness by meaning: P@10, nDCG@10, MAP.
    pub full: Option<[f64; 3]>,
    pub paraphrase_auc: Option<f64>,
    pub paraphrase_recall: Option<f64>,
    pub paraphrase_threshold: Option<f32>,
}

#[derive(Serialize)]
pub struct ModelsReport {
    pub rows: Vec<ModelRow>,
    pub missing: Vec<String>,
}

fn triple(r: &matching_eval::RankerResult) -> [f64; 3] {
    [r.precision_at_10, r.ndcg_at_10, r.map]
}

pub fn run(seed: u64, dir: &std::path::Path) -> ModelsReport {
    let mut rows = Vec::new();
    let mut missing = Vec::new();
    for spec in MODELS {
        let choice = Choice {
            name: spec.name.to_owned(),
            dir: dir.to_path_buf(),
        };
        let started = std::time::Instant::now();
        let Some(model) = choice.load() else {
            missing.push(spec.name.to_owned());
            continue;
        };
        let load_ms = started.elapsed().as_secs_f64() * 1000.0;
        let m = matching_eval::run(seed, Some(&model));
        let p = paraphrase::run(&model);
        rows.push(ModelRow {
            model: model.id().to_owned(),
            dim: model.dim(),
            load_ms,
            embed_ms_per_text: m.text.embed_ms_per_text,
            meaning_signal: m.text.meaning_signal,
            keywords_signal: m.text.keywords_signal,
            meaning_only: m
                .results
                .iter()
                .find(|r| r.name.starts_with("Только близость"))
                .map(triple),
            full: m.text_ablation.first().map(triple),
            paraphrase_auc: p.as_ref().map(|p| p.auc),
            paraphrase_recall: p.as_ref().map(|p| p.recall_without_false_alarms),
            paraphrase_threshold: p.as_ref().and_then(|p| p.suggested),
        });
    }
    ModelsReport { rows, missing }
}

pub fn markdown(r: &ModelsReport) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "## Модели эмбеддингов: сравнение на наших данных\n");
    let _ = writeln!(
        s,
        "Одна и та же синтетика подбора (`matching.md`) и один и тот же набор пересказов (`paraphrase.md`). \
         Каждая модель — как в её карточке: rubert-tiny2 — вектор `[CLS]`, multilingual-e5-small — среднее по \
         токенам с префиксами `query: ` / `passage: `; оба нормированы. Скорость — в этом процессе на этой машине, \
         все ядра; память сервиса измеряется отдельно (README.md, раздел об ML).\n"
    );
    let f = |v: Option<f64>, d: usize| v.map_or_else(|| "—".to_owned(), |v| format!("{v:.d$}"));
    let t = |v: Option<[f64; 3]>| {
        v.map_or_else(
            || "—".to_owned(),
            |[p, n, m]| format!("{p:.3} / {n:.3} / {m:.3}"),
        )
    };
    let _ = writeln!(
        s,
        "| Модель | Размерность | Загрузка, мс | мс на текст | Корреляция с реальными навыками | Только по смыслу: P@10 / nDCG@10 / MAP | Полный подбор: P@10 / nDCG@10 / MAP | Пересказы: ROC AUC | Пойманы без ложных срабатываний | Порог |\n|---|---|---|---|---|---|---|---|---|---|"
    );
    for row in &r.rows {
        let _ = writeln!(
            s,
            "| `{}` | {} | {:.0} | {} | {} (ключевые слова: {}) | {} | {} | {} | {} | {} |",
            row.model,
            row.dim,
            row.load_ms,
            f(row.embed_ms_per_text, 1),
            f(row.meaning_signal, 2),
            f(row.keywords_signal, 2),
            t(row.meaning_only),
            t(row.full),
            f(row.paraphrase_auc, 3),
            row.paraphrase_recall
                .map_or_else(|| "—".to_owned(), |v| format!("{:.0} %", v * 100.0)),
            row.paraphrase_threshold
                .map_or_else(|| "—".to_owned(), |v| format!("{v:.3}")),
        );
    }
    if !r.missing.is_empty() {
        let _ = writeln!(
            s,
            "\nНе найдены (`make model MODEL=…`): {}.",
            r.missing.join(", ")
        );
    }
    s
}
