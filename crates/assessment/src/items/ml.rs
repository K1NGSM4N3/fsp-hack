//! ML / data science templates: metrics, probability, descriptive statistics, training
//! dynamics, pandas and numpy semantics, optimisation steps.

use catalog::Specialization::{Ml, Security};

use super::util::{chance, fmt_number, join, md_table, pick, pick_n, range, shuffled};
use super::{C_CHOICE, C_OPEN, Draft, Level, Rng, Source, Template, Topic};

pub(crate) const TEMPLATES: &[Template] = &[
    Template {
        id: "ml.confusion_metrics",
        title: "Метрики по матрице ошибок",
        topic: Topic::MlMetrics,
        specializations: &[Ml],
        skills: &["ml_metrics", "statistics"],
        levels: &[
            Level::new(1, 1.0, -0.9, C_OPEN, 120),
            Level::new(2, 1.2, 0.0, C_OPEN, 150),
            Level::new(3, 1.4, 0.9, C_OPEN, 180),
        ],
        source: Source::Computed(confusion_metrics),
    },
    Template {
        id: "stats.bayes_base_rate",
        title: "Теорема Байеса и базовая частота",
        topic: Topic::Statistics,
        specializations: &[Ml, Security],
        skills: &["statistics"],
        levels: &[
            Level::new(1, 1.0, 0.0, C_OPEN, 150),
            Level::new(2, 1.3, 0.6, C_OPEN, 150),
            Level::new(3, 1.4, 1.4, C_OPEN, 180),
        ],
        source: Source::Computed(bayes),
    },
    Template {
        id: "stats.descriptive",
        title: "Медиана и перцентили",
        topic: Topic::Statistics,
        specializations: &[Ml],
        skills: &["statistics", "numpy"],
        levels: &[
            Level::new(1, 0.9, -1.3, C_OPEN, 90),
            Level::new(2, 1.0, -0.6, C_OPEN, 120),
            Level::new(3, 1.3, 0.5, C_OPEN, 150),
        ],
        source: Source::Computed(descriptive),
    },
    Template {
        id: "ml.learning_curves",
        title: "Кривые обучения и ранняя остановка",
        topic: Topic::Modeling,
        specializations: &[Ml],
        skills: &["scikit_learn", "pytorch"],
        levels: &[
            Level::new(1, 1.1, -0.5, C_CHOICE, 90),
            Level::new(2, 1.3, 0.4, C_CHOICE, 120),
        ],
        source: Source::Computed(learning_curves),
    },
    Template {
        id: "ml.pandas_groupby",
        title: "Группировки в pandas",
        topic: Topic::DataTools,
        specializations: &[Ml],
        skills: &["pandas", "python"],
        levels: &[
            Level::new(1, 1.0, -0.7, C_OPEN, 120),
            Level::new(2, 1.2, 0.1, C_OPEN, 150),
            Level::new(3, 1.4, 0.8, C_OPEN, 150),
        ],
        source: Source::Computed(pandas_groupby),
    },
    Template {
        id: "ml.numpy_shapes",
        title: "Формы массивов numpy",
        topic: Topic::DataTools,
        specializations: &[Ml],
        skills: &["numpy"],
        levels: &[
            Level::new(1, 1.0, -0.8, C_CHOICE, 60),
            Level::new(2, 1.3, 0.2, C_CHOICE, 90),
            Level::new(3, 1.4, 1.0, C_CHOICE, 120),
        ],
        source: Source::Computed(numpy_shapes),
    },
    Template {
        id: "ml.kmeans_step",
        title: "Шаг алгоритма k-means",
        topic: Topic::Modeling,
        specializations: &[Ml],
        skills: &["scikit_learn", "statistics"],
        levels: &[
            Level::new(1, 1.0, -0.4, C_OPEN, 120),
            Level::new(2, 1.2, 0.4, C_OPEN, 150),
            Level::new(3, 1.4, 1.0, C_OPEN, 180),
        ],
        source: Source::Computed(kmeans),
    },
    Template {
        id: "ml.gradient_step",
        title: "Шаг градиентного спуска",
        topic: Topic::Modeling,
        specializations: &[Ml],
        skills: &["pytorch", "numpy"],
        levels: &[
            Level::new(1, 0.9, -1.0, C_OPEN, 90),
            Level::new(2, 1.2, 0.5, C_OPEN, 150),
            Level::new(3, 1.4, 1.2, C_OPEN, 180),
        ],
        source: Source::Computed(gradient_step),
    },
];

fn round_to(x: f64, digits: i32) -> f64 {
    let k = 10f64.powi(digits);
    (x * k).round() / k
}

/// Accepts the exact value and both neighbours of its rounding to `digits` decimals.
fn rounded(prompt: String, value: f64, digits: i32) -> Draft {
    let hint = match digits {
        0 => "Число, округлите до целого",
        1 => "Число, округлите до десятых",
        2 => "Число, округлите до сотых",
        _ => "Число, округлите до тысячных",
    };
    Draft::number(prompt, round_to(value, digits), 10f64.powi(-digits) * 1.01).hint(hint)
}

fn confusion_metrics(level: u8, rng: &mut Rng) -> Draft {
    if level < 3 {
        let tp = range(rng, 20, 90);
        let fp = range(rng, 4, 45);
        let fn_ = range(rng, 4, 45);
        let tn = range(rng, 50, 400);
        let table = md_table(
            &["", "Предсказано: 1", "Предсказано: 0"],
            &[
                vec!["**Факт: 1**".into(), tp.to_string(), fn_.to_string()],
                vec!["**Факт: 0**".into(), fp.to_string(), tn.to_string()],
            ],
        );
        let (name, value) = if level == 2 {
            (
                "F1-мера (для класса 1)",
                2.0 * tp as f64 / (2 * tp + fp + fn_) as f64,
            )
        } else if chance(rng, 0.5) {
            ("precision (для класса 1)", tp as f64 / (tp + fp) as f64)
        } else {
            ("recall (для класса 1)", tp as f64 / (tp + fn_) as f64)
        };
        return rounded(
            format!(
                "Матрица ошибок бинарного классификатора на тестовой выборке:\n\n{table}\nКаково значение метрики {name}?"
            ),
            value,
            2,
        );
    }
    let classes = pick_n(rng, &["кошка", "собака", "птица", "рыба", "хомяк"], 3);
    let m: Vec<Vec<i64>> = (0..3)
        .map(|i| {
            (0..3)
                .map(|j| {
                    if i == j {
                        range(rng, 20, 80)
                    } else {
                        range(rng, 1, 15)
                    }
                })
                .collect()
        })
        .collect();
    let macro_recall = chance(rng, 0.5);
    let per_class: Vec<f64> = (0..3)
        .map(|k| {
            let denom: i64 = if macro_recall {
                m[k].iter().sum()
            } else {
                (0..3).map(|i| m[i][k]).sum()
            };
            m[k][k] as f64 / denom as f64
        })
        .collect();
    let value = per_class.iter().sum::<f64>() / 3.0;
    let mut header = vec!["Факт \\ Прогноз".to_owned()];
    header.extend(classes.iter().map(|c| (*c).to_owned()));
    let rows: Vec<Vec<String>> = (0..3)
        .map(|i| {
            let mut r = vec![format!("**{}**", classes[i])];
            r.extend(m[i].iter().map(|v| v.to_string()));
            r
        })
        .collect();
    let header_refs: Vec<&str> = header.iter().map(String::as_str).collect();
    let table = md_table(&header_refs, &rows);
    let name = if macro_recall {
        "macro-averaged recall"
    } else {
        "macro-averaged precision"
    };
    rounded(
        format!(
            "Матрица ошибок классификатора на три класса (строки — факт, столбцы — прогноз):\n\n{table}\nКаково значение метрики {name}?"
        ),
        value,
        2,
    )
}

fn pct(x: f64) -> String {
    fmt_number(round_to(x * 100.0, 2))
}

fn bayes(level: u8, rng: &mut Rng) -> Draft {
    let posterior =
        |prior: f64, tpr: f64, fpr: f64| tpr * prior / (tpr * prior + fpr * (1.0 - prior));
    match level {
        1 => {
            let prior: f64 = *pick(rng, &[0.01, 0.02, 0.05, 0.1]);
            let sens: f64 = *pick(rng, &[0.9, 0.95, 0.99]);
            let spec: f64 = *pick(rng, &[0.9, 0.95, 0.98, 0.99]);
            rounded(
                format!(
                    "Заболеванием страдает {}% обследуемых. Тест выявляет болезнь у {}% больных (чувствительность) и даёт отрицательный результат у {}% здоровых (специфичность). \
                     Тест у пациента положительный. Какова вероятность, что он действительно болен? Ответ — в процентах.",
                    pct(prior),
                    pct(sens),
                    pct(spec)
                ),
                posterior(prior, sens, 1.0 - spec) * 100.0,
                0,
            )
        }
        2 => {
            let prior: f64 = *pick(rng, &[0.001, 0.002, 0.005, 0.01]);
            let tpr: f64 = *pick(rng, &[0.8, 0.9, 0.95]);
            let fpr: f64 = *pick(rng, &[0.005, 0.01, 0.02, 0.05]);
            let (subject, positive, hit) = *pick(
                rng,
                &[
                    (
                        "транзакций — мошеннические",
                        "модель пометила транзакцию как мошенническую",
                        "мошенничество",
                    ),
                    (
                        "сетевых сессий — атаки",
                        "IDS подняла тревогу по сессии",
                        "атака",
                    ),
                    (
                        "писем — фишинг",
                        "фильтр пометил письмо как фишинг",
                        "фишинг",
                    ),
                ],
            );
            rounded(
                format!(
                    "{}% {subject}. Детектор срабатывает на {}% положительных случаев и ложно срабатывает на {}% остальных. \
                     Если {positive}, какова вероятность, что это действительно {hit}? Ответ — в процентах.",
                    pct(prior),
                    pct(tpr),
                    pct(fpr)
                ),
                posterior(prior, tpr, fpr) * 100.0,
                1,
            )
        }
        _ => {
            let prior: f64 = *pick(rng, &[0.01, 0.02, 0.05]);
            let sens: f64 = *pick(rng, &[0.9, 0.95]);
            let spec: f64 = *pick(rng, &[0.9, 0.95, 0.97]);
            let after_one = posterior(prior, sens, 1.0 - spec);
            let after_two = posterior(after_one, sens, 1.0 - spec);
            rounded(
                format!(
                    "Распространённость заболевания — {}%. Тест имеет чувствительность {}% и специфичность {}%; результаты повторных тестов у одного человека независимы при условии его состояния. \
                     У пациента два теста подряд оказались положительными. Какова вероятность, что он болен? Ответ — в процентах.",
                    pct(prior),
                    pct(sens),
                    pct(spec)
                ),
                after_two * 100.0,
                1,
            )
        }
    }
}

fn numpy_percentile(sorted: &[i64], q: f64) -> f64 {
    let pos = (sorted.len() - 1) as f64 * q / 100.0;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    let frac = pos - lo as f64;
    sorted[lo] as f64 + (sorted[hi] - sorted[lo]) as f64 * frac
}

fn descriptive(level: u8, rng: &mut Rng) -> Draft {
    let n = match level {
        1 => range(rng, 3, 4) * 2 + 1,
        2 => range(rng, 4, 5) * 2,
        _ => range(rng, 6, 10),
    } as usize;
    let mut values: Vec<i64> = (0..n).map(|_| range(rng, 1, 60)).collect();
    if level == 2 {
        values[0] = range(rng, 150, 900);
        values = shuffled(rng, values);
    }
    let mut sorted = values.clone();
    sorted.sort_unstable();
    let list = join(&values, ", ");
    match level {
        1 => Draft::integer(format!("Найдите медиану выборки: {list}."), sorted[n / 2]),
        2 => {
            let median = (sorted[n / 2 - 1] + sorted[n / 2]) as f64 / 2.0;
            Draft::number(
                format!("Найдите медиану выборки (в ней есть выброс): {list}."),
                median,
                0.001,
            )
        }
        _ => {
            let q = *pick(rng, &[10.0, 25.0, 75.0, 90.0]);
            rounded(
                format!(
                    "Что выведет код? Используется `numpy.percentile` с методом по умолчанию (`linear`).\n\n```python\nimport numpy as np\n\nx = np.array([{list}])\nprint(np.percentile(x, {}))\n```",
                    fmt_number(q)
                ),
                numpy_percentile(&sorted, q),
                2,
            )
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Fit {
    Overfit,
    Underfit,
    Good,
    Diverging,
}

impl Fit {
    fn label(self) -> &'static str {
        match self {
            Fit::Overfit => "Переобучение: модель запоминает обучающую выборку",
            Fit::Underfit => "Недообучение: модель слишком простая или обучается слишком мало",
            Fit::Good => "Обучение идёт нормально, явных проблем нет",
            Fit::Diverging => "Обучение расходится: вероятно, слишком большой learning rate",
        }
    }
}

fn loss_table(train: &[f64], val: &[f64]) -> String {
    let rows: Vec<Vec<String>> = train
        .iter()
        .zip(val)
        .enumerate()
        .map(|(i, (t, v))| vec![(i + 1).to_string(), format!("{t:.3}"), format!("{v:.3}")])
        .collect();
    md_table(&["Эпоха", "train_loss", "val_loss"], &rows)
}

fn noise(rng: &mut Rng, scale: f64) -> f64 {
    (range(rng, -100, 100) as f64 / 100.0) * scale
}

fn learning_curves(level: u8, rng: &mut Rng) -> Draft {
    let epochs = range(rng, 8, 12) as usize;
    if level == 1 {
        let fit = *pick(
            rng,
            &[Fit::Overfit, Fit::Underfit, Fit::Good, Fit::Diverging],
        );
        let start = 0.9 + noise(rng, 0.2);
        let mut train = Vec::new();
        let mut val = Vec::new();
        let turn = range(rng, 3, 5) as usize;
        for e in 0..epochs {
            let t = e as f64;
            let (tr, va) = match fit {
                Fit::Overfit => {
                    let tr = start * (-0.35 * t).exp() + 0.02;
                    let va = if e <= turn {
                        start * (-0.25 * t).exp() + 0.25
                    } else {
                        start * (-0.25 * turn as f64).exp() + 0.25 + 0.06 * (e - turn) as f64
                    };
                    (tr, va)
                }
                Fit::Underfit => (
                    start - 0.01 * t + noise(rng, 0.004),
                    start + 0.03 - 0.008 * t + noise(rng, 0.004),
                ),
                Fit::Good => {
                    let tr = start * (-0.3 * t).exp() + 0.18;
                    (tr, tr + 0.03 + noise(rng, 0.005))
                }
                Fit::Diverging => {
                    let tr = start * (1.0 + 0.45 * t + 0.1 * t * t) + noise(rng, 0.1);
                    (tr, tr * 1.1)
                }
            };
            train.push(tr);
            val.push(va);
        }
        let options = [Fit::Overfit, Fit::Underfit, Fit::Good, Fit::Diverging];
        return Draft::choice(
            format!(
                "Значения функции потерь при обучении нейросети:\n\n{}\nКакой диагноз точнее всего?",
                loss_table(&train, &val)
            ),
            fit.label(),
            options
                .iter()
                .filter(|f| **f != fit)
                .map(|f| f.label().to_owned()),
        );
    }

    let patience = range(rng, 2, 3) as usize;
    for _ in 0..64 {
        let best = range(rng, 3, (epochs - patience - 1) as i64) as usize;
        let mut val = Vec::with_capacity(epochs);
        let mut v = 1.2 + noise(rng, 0.2);
        for e in 0..epochs {
            if e <= best {
                v -= 0.05 + range(rng, 0, 10) as f64 / 100.0;
                if e > 0 && e + 1 < best && chance(rng, 0.25) {
                    v += 0.08;
                }
            } else if e > best {
                v = val[best] + 0.01 + range(rng, 0, 12) as f64 / 100.0;
            }
            val.push(v);
        }
        let (stop, restored) = early_stopping(&val, patience);
        let Some(stop) = stop else { continue };
        let train: Vec<f64> = (0..epochs)
            .map(|e| 1.0 * (-0.3 * e as f64).exp() + 0.05)
            .collect();
        let render = |s: usize, r: usize| {
            format!("Остановится после эпохи {s}; восстановлены веса эпохи {r}")
        };
        let (s, r) = (stop + 1, restored + 1);
        let distractors = vec![
            render(s - 1, r),
            render(s, s),
            render(s + 1, r),
            render(epochs, r),
            render(r, r),
            render(s, r - 1),
        ];
        return Draft::choice(
            format!(
                "Обучение с ранней остановкой: monitor=`val_loss`, patience={patience}, restore_best_weights=True. Счётчик ожидания сбрасывается при любом уменьшении val_loss \
                 и растёт на 1 после каждой эпохи без улучшения; обучение останавливается, когда счётчик достигает patience.\n\n{}\nКогда остановится обучение и веса какой эпохи будут восстановлены?",
                loss_table(&train, &val)
            ),
            render(s, r),
            distractors,
        );
    }
    Draft::choice(
        "patience=2, val_loss по эпохам: 0.9, 0.7, 0.6, 0.65, 0.66, 0.7. Когда остановится обучение?",
        "Остановится после эпохи 5; восстановлены веса эпохи 3",
        [
            "Остановится после эпохи 4; восстановлены веса эпохи 3",
            "Остановится после эпохи 6; восстановлены веса эпохи 3",
            "Остановится после эпохи 5; восстановлены веса эпохи 5",
        ]
        .map(str::to_owned),
    )
}

/// Keras `EarlyStopping` semantics with `min_delta = 0`; returns 0-based (stop, best) epochs.
fn early_stopping(val: &[f64], patience: usize) -> (Option<usize>, usize) {
    let (mut best, mut best_epoch, mut wait) = (f64::INFINITY, 0, 0);
    for (e, &v) in val.iter().enumerate() {
        if v < best {
            best = v;
            best_epoch = e;
            wait = 0;
        } else {
            wait += 1;
            if wait >= patience {
                return (Some(e), best_epoch);
            }
        }
    }
    (None, best_epoch)
}

struct Row {
    city: &'static str,
    product: &'static str,
    qty: Option<i64>,
    price: i64,
}

fn frame(rng: &mut Rng, with_nan: bool) -> (Vec<&'static str>, Vec<Row>) {
    let cities = pick_n(rng, super::util::CITIES, 3);
    let products = pick_n(rng, &["чай", "кофе", "сок", "вода", "молоко"], 3);
    let n = range(rng, 7, 9) as usize;
    let rows = (0..n)
        .map(|i| Row {
            city: if i < 3 { cities[i] } else { pick(rng, &cities) },
            product: pick(rng, &products),
            qty: if with_nan && chance(rng, 0.25) {
                None
            } else {
                Some(range(rng, 1, 12))
            },
            price: range(rng, 5, 40) * 10,
        })
        .collect();
    (cities, shuffled(rng, rows))
}

fn frame_table(rows: &[Row]) -> String {
    let body: Vec<Vec<String>> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            vec![
                i.to_string(),
                r.city.to_owned(),
                r.product.to_owned(),
                r.qty.map_or("NaN".to_owned(), |q| q.to_string()),
                r.price.to_string(),
            ]
        })
        .collect();
    md_table(&["", "city", "product", "qty", "price"], &body)
}

fn pandas_groupby(level: u8, rng: &mut Rng) -> Draft {
    for _ in 0..32 {
        let (cities, rows) = frame(rng, level == 2);
        let city = *pick(rng, &cities);
        let in_city: Vec<&Row> = rows.iter().filter(|r| r.city == city).collect();
        let (code, draft) = match level {
            1 => {
                if chance(rng, 0.5) {
                    let best = cities
                        .iter()
                        .map(|c| {
                            rows.iter()
                                .filter(|r| r.city == *c)
                                .filter_map(|r| r.qty)
                                .sum::<i64>()
                        })
                        .max()
                        .unwrap_or(0);
                    (
                        "print(df.groupby(\"city\")[\"qty\"].sum().max())".to_owned(),
                        Draft::integer("Что выведет код?", best),
                    )
                } else {
                    let total: i64 = in_city.iter().map(|r| r.qty.unwrap_or(0) * r.price).sum();
                    (
                        format!(
                            "df[\"revenue\"] = df[\"qty\"] * df[\"price\"]\nprint(df.groupby(\"city\")[\"revenue\"].sum()[\"{city}\"])"
                        ),
                        Draft::integer("Что выведет код?", total),
                    )
                }
            }
            2 => {
                let present: Vec<i64> = in_city.iter().filter_map(|r| r.qty).collect();
                if present.len() == in_city.len() || present.is_empty() {
                    continue;
                }
                if chance(rng, 0.5) {
                    (
                        format!("print(df.groupby(\"city\")[\"qty\"].count()[\"{city}\"])"),
                        Draft::integer("Что выведет код?", present.len() as i64),
                    )
                } else {
                    let mean = present.iter().sum::<i64>() as f64 / present.len() as f64;
                    (
                        format!(
                            "print(round(df.groupby(\"city\")[\"qty\"].mean()[\"{city}\"], 2))"
                        ),
                        Draft::number("Что выведет код?", round_to(mean, 2), 0.001).hint("Число"),
                    )
                }
            }
            _ => {
                if chance(rng, 0.5) {
                    let mut pairs: Vec<(&str, &str)> =
                        rows.iter().map(|r| (r.city, r.product)).collect();
                    pairs.sort_unstable();
                    pairs.dedup();
                    let call = if chance(rng, 0.5) {
                        "print(df.groupby(\"city\")[\"product\"].nunique().sum())"
                    } else {
                        "print(len(df.drop_duplicates(subset=[\"city\", \"product\"])))"
                    };
                    (
                        call.to_owned(),
                        Draft::integer("Что выведет код?", pairs.len() as i64),
                    )
                } else {
                    let threshold = range(rng, 10, 30) * 10;
                    let best = cities
                        .iter()
                        .map(|c| {
                            rows.iter()
                                .filter(|r| r.city == *c && r.price > threshold)
                                .count() as i64
                        })
                        .max()
                        .unwrap_or(0);
                    if best == 0 {
                        continue;
                    }
                    (
                        format!(
                            "print(df[df[\"price\"] > {threshold}].groupby(\"city\").size().max())"
                        ),
                        Draft::integer("Что выведет код?", best),
                    )
                }
            }
        };
        return draft
            .context(format!("DataFrame `df`:\n\n{}", frame_table(&rows)))
            .code("python", code);
    }
    Draft::integer(
        "Сколько строк в DataFrame из трёх строк вернёт `len(df)`?",
        3,
    )
}

type Shape = Vec<usize>;

fn shape_text(s: &[usize]) -> String {
    match s.len() {
        1 => format!("({},)", s[0]),
        _ => format!("({})", join(s, ", ")),
    }
}

fn broadcast(a: &[usize], b: &[usize]) -> Option<Shape> {
    let n = a.len().max(b.len());
    let mut out = vec![0; n];
    for i in 0..n {
        let da = if i < n - a.len() {
            1
        } else {
            a[i - (n - a.len())]
        };
        let db = if i < n - b.len() {
            1
        } else {
            b[i - (n - b.len())]
        };
        out[i] = match (da, db) {
            (x, y) if x == y => x,
            (1, y) => y,
            (x, 1) => x,
            _ => return None,
        };
    }
    Some(out)
}

const SHAPE_ERROR: &str = "ValueError (несовместимые формы)";

fn numpy_shapes(level: u8, rng: &mut Rng) -> Draft {
    let dim = |rng: &mut Rng| range(rng, 2, 7) as usize;
    match level {
        1 => {
            let (m, n, p) = loop {
                let t = (dim(rng), dim(rng), dim(rng));
                if t.0 != t.1 && t.1 != t.2 && t.0 != t.2 {
                    break t;
                }
            };
            let transposed = chance(rng, 0.5);
            let b_shape = if transposed { vec![p, n] } else { vec![n, p] };
            let expr = if transposed { "A @ B.T" } else { "A @ B" };
            Draft::choice(
                format!(
                    "`A.shape == {}`, `B.shape == {}`. Какой будет форма результата `{expr}`?",
                    shape_text(&[m, n]),
                    shape_text(&b_shape)
                ),
                shape_text(&[m, p]),
                vec![
                    shape_text(&[n, n]),
                    shape_text(&[m, n]),
                    shape_text(&[p, m]),
                    SHAPE_ERROR.to_owned(),
                    shape_text(&[n, p]),
                ],
            )
        }
        2 => loop {
            let rank_a = range(rng, 1, 3) as usize;
            let rank_b = range(rng, 1, 3) as usize;
            let mut a: Shape = (0..rank_a)
                .map(|_| *pick(rng, &[1usize, 3, 4, 5]))
                .collect();
            let mut b: Shape = (0..rank_b)
                .map(|_| *pick(rng, &[1usize, 3, 4, 5]))
                .collect();
            if chance(rng, 0.7) {
                let k = a.len().min(b.len());
                for i in 0..k {
                    let (ia, ib) = (a.len() - 1 - i, b.len() - 1 - i);
                    if a[ia] != b[ib] && a[ia] != 1 && b[ib] != 1 {
                        if chance(rng, 0.5) {
                            a[ia] = 1
                        } else {
                            b[ib] = a[ia]
                        }
                    }
                }
            }
            if a == b {
                continue;
            }
            let result = broadcast(&a, &b);
            let answer = result
                .as_ref()
                .map_or_else(|| SHAPE_ERROR.to_owned(), |s| shape_text(s));
            let mut distractors = vec![SHAPE_ERROR.to_owned(), shape_text(&a), shape_text(&b)];
            let mut left_aligned: Shape = a
                .iter()
                .chain(b.iter())
                .copied()
                .filter(|d| *d != 1)
                .collect();
            left_aligned.dedup();
            if !left_aligned.is_empty() {
                distractors.push(shape_text(&left_aligned));
            }
            let mut elementwise_max: Shape = a.clone();
            elementwise_max.extend(b.iter());
            distractors.push(shape_text(&elementwise_max));
            if let Some(r) = &result {
                let mut flipped = r.clone();
                flipped.reverse();
                distractors.push(shape_text(&flipped));
                distractors.push(shape_text(&[r.iter().product()]));
            }
            return Draft::choice(
                format!(
                    "`a.shape == {}`, `b.shape == {}`. Что получится в результате `a + b`?",
                    shape_text(&a),
                    shape_text(&b)
                ),
                answer,
                distractors,
            );
        },
        _ => {
            let (a, b, c) = loop {
                let t = (dim(rng), dim(rng), dim(rng));
                if t.0 != t.1 && t.1 != t.2 && t.0 != t.2 {
                    break t;
                }
            };
            let ops: [(&str, Shape, Vec<Shape>); 5] = [
                (
                    "x.sum(axis=1)",
                    vec![a, c],
                    vec![vec![a, b], vec![b], vec![a, 1, c], vec![b, c]],
                ),
                (
                    "x.mean(axis=(0, 2), keepdims=True)",
                    vec![1, b, 1],
                    vec![vec![b], vec![a, 1, c], vec![1, b], vec![b, 1]],
                ),
                (
                    "x.reshape(-1, x.shape[-1])",
                    vec![a * b, c],
                    vec![
                        vec![a, b * c],
                        vec![c, a * b],
                        vec![a * b * c],
                        vec![a, b, c],
                    ],
                ),
                (
                    "x.transpose(2, 0, 1).reshape(x.shape[2], -1)",
                    vec![c, a * b],
                    vec![
                        vec![a * b, c],
                        vec![c, a, b],
                        vec![a, b * c],
                        vec![b * c, a],
                    ],
                ),
                (
                    "x[:, None, :, 0]",
                    vec![a, 1, b],
                    vec![vec![a, b], vec![a, 1, b, c], vec![a, b, 1], vec![1, a, b]],
                ),
            ];
            let (expr, answer, wrong) = pick(rng, &ops).clone();
            Draft::choice(
                format!(
                    "`x.shape == {}`. Какой будет форма результата `{expr}`?",
                    shape_text(&[a, b, c])
                ),
                shape_text(&answer),
                shuffled(rng, wrong)
                    .iter()
                    .map(|s| shape_text(s))
                    .collect::<Vec<_>>(),
            )
        }
    }
}

fn kmeans_assign(points: &[i64], centroids: &[f64]) -> Option<Vec<usize>> {
    points
        .iter()
        .map(|p| {
            let d: Vec<f64> = centroids.iter().map(|c| (*p as f64 - c).abs()).collect();
            if (d[0] - d[1]).abs() < 1e-9 {
                None
            } else if d[0] < d[1] {
                Some(0)
            } else {
                Some(1)
            }
        })
        .collect()
}

fn kmeans_update(points: &[i64], labels: &[usize]) -> Option<Vec<f64>> {
    (0..2)
        .map(|k| {
            let members: Vec<i64> = points
                .iter()
                .zip(labels)
                .filter(|(_, l)| **l == k)
                .map(|(p, _)| *p)
                .collect();
            (!members.is_empty()).then(|| members.iter().sum::<i64>() as f64 / members.len() as f64)
        })
        .collect()
}

fn kmeans(level: u8, rng: &mut Rng) -> Draft {
    for _ in 0..128 {
        let n = range(rng, 6, 9) as usize;
        let mut points: Vec<i64> = (0..n).map(|_| range(rng, 0, 40)).collect();
        points.sort_unstable();
        points.dedup();
        if points.len() < 5 {
            continue;
        }
        let points = shuffled(rng, points);
        let c1 = range(rng, 0, 20);
        let c2 = c1 + range(rng, 3, 20);
        let init = [c1 as f64, c2 as f64];
        let Some(labels) = kmeans_assign(&points, &init) else {
            continue;
        };
        let Some(next) = kmeans_update(&points, &labels) else {
            continue;
        };
        let list = join(&points, ", ");
        let intro = format!(
            "Одномерные точки: {list}. Алгоритм k-means с k = 2, начальные центроиды: c₁ = {c1}, c₂ = {c2}. Расстояние — модуль разности."
        );
        return match level {
            1 => Draft::integer(
                format!(
                    "{intro} Сколько точек будет отнесено к кластеру c₁ на первом шаге назначения?"
                ),
                labels.iter().filter(|l| **l == 0).count() as i64,
            ),
            2 => rounded(
                format!(
                    "{intro} Чему станет равен центроид c₁ после первой итерации (назначение точек и пересчёт центроидов)?"
                ),
                next[0],
                2,
            ),
            _ => {
                let Some(labels2) = kmeans_assign(&points, &next) else {
                    continue;
                };
                let Some(after) = kmeans_update(&points, &labels2) else {
                    continue;
                };
                if labels2 == labels {
                    continue;
                }
                rounded(
                    format!(
                        "{intro} Чему будет равен центроид c₂ после второй итерации (два раза: назначение точек и пересчёт центроидов)?"
                    ),
                    after[1],
                    2,
                )
            }
        };
    }
    Draft::integer(
        "Точки 1, 2, 10, 11; k = 2, центроиды 0 и 12. Сколько точек попадёт к первому центроиду?",
        2,
    )
}

fn gradient_step(level: u8, rng: &mut Rng) -> Draft {
    let lr: f64 = *pick(rng, &[0.1, 0.01, 0.05]);
    let w: f64 = *pick(rng, &[0.5, 1.0, 1.5, 2.0, -0.5]);
    match level {
        1 => {
            let grad: f64 = *pick(rng, &[0.4, -0.8, 1.5, 2.0, -3.0, 6.0]);
            rounded(
                format!(
                    "Обычный SGD без momentum. Текущее значение веса w = {}, градиент функции потерь по нему ∂L/∂w = {}, learning rate = {}. Каким станет w после одного шага?",
                    fmt_number(w),
                    fmt_number(grad),
                    fmt_number(lr)
                ),
                w - lr * grad,
                3,
            )
        }
        _ => {
            let b0: f64 = *pick(rng, &[0.0, 0.5, 1.0]);
            let n = if level == 2 {
                1
            } else {
                range(rng, 2, 3) as usize
            };
            let samples: Vec<(i64, i64)> = (0..n)
                .map(|_| (range(rng, 1, 5), range(rng, 0, 10)))
                .collect();
            let grad = samples
                .iter()
                .map(|(x, y)| 2.0 * (w * *x as f64 + b0 - *y as f64) * *x as f64)
                .sum::<f64>()
                / n as f64;
            let data = samples
                .iter()
                .map(|(x, y)| format!("(x = {x}, y = {y})"))
                .collect::<Vec<_>>()
                .join(", ");
            let loss = if n == 1 {
                "L = (ŷ − y)²".to_owned()
            } else {
                "L = (1/N)·Σ(ŷᵢ − yᵢ)² — среднее по батчу".to_owned()
            };
            rounded(
                format!(
                    "Линейная модель ŷ = w·x + b, сейчас w = {}, b = {}. Функция потерь {loss}. Данные: {data}. \
                     Сделайте один шаг градиентного спуска по w (b не меняется) с learning rate = {}. Каким станет w?",
                    fmt_number(w),
                    fmt_number(b0),
                    fmt_number(lr)
                ),
                w - lr * grad,
                3,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_matches_numpy_linear() {
        let x = [1, 2, 3, 4];
        assert!((numpy_percentile(&x, 25.0) - 1.75).abs() < 1e-12);
        assert!((numpy_percentile(&x, 50.0) - 2.5).abs() < 1e-12);
        assert!((numpy_percentile(&[7, 15, 36, 39, 40, 41], 90.0) - 40.5).abs() < 1e-12);
    }

    #[test]
    fn broadcasting_rules() {
        assert_eq!(broadcast(&[3, 1], &[4]), Some(vec![3, 4]));
        assert_eq!(broadcast(&[5, 1, 4], &[3, 1]), Some(vec![5, 3, 4]));
        assert_eq!(broadcast(&[3], &[4]), None);
        assert_eq!(broadcast(&[2, 3], &[3]), Some(vec![2, 3]));
    }

    #[test]
    fn early_stopping_semantics() {
        let val = [0.9, 0.7, 0.75, 0.6, 0.65, 0.66, 0.7];
        assert_eq!(early_stopping(&val, 2), (Some(5), 3));
        assert_eq!(early_stopping(&val, 1), (Some(2), 1));
    }

    #[test]
    fn kmeans_iteration() {
        let points = [1, 2, 10, 11, 12];
        let labels = kmeans_assign(&points, &[0.0, 5.0]).unwrap();
        assert_eq!(labels, vec![0, 0, 1, 1, 1]);
        let next = kmeans_update(&points, &labels).unwrap();
        assert!((next[0] - 1.5).abs() < 1e-12 && (next[1] - 11.0).abs() < 1e-12);
        assert_eq!(kmeans_assign(&points, &[0.0, 2.0]), None);
    }
}
