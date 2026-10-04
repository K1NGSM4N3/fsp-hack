//! QA templates: test design techniques, API test assertions, load modelling.

use catalog::Specialization::{Backend, Devops, Qa};

use super::util::{chance, fmt_number, join, md_table, pick, pick_n, range, shuffled};
use super::{C_CHOICE, C_OPEN, Draft, Level, Rng, Source, Template, Topic};

pub(crate) const TEMPLATES: &[Template] = &[
    Template {
        id: "qa.boundary_values",
        title: "Анализ граничных значений",
        topic: Topic::TestDesign,
        specializations: &[Qa],
        skills: &["test_design", "manual_testing"],
        levels: &[
            Level::new(1, 1.0, -1.2, C_CHOICE, 90),
            Level::new(2, 1.2, -0.3, C_CHOICE, 90),
            Level::new(3, 1.4, 0.6, C_CHOICE, 120),
        ],
        source: Source::Computed(boundary_values),
    },
    Template {
        id: "qa.equivalence_classes",
        title: "Классы эквивалентности",
        topic: Topic::TestDesign,
        specializations: &[Qa],
        skills: &["test_design", "manual_testing"],
        levels: &[
            Level::new(1, 1.0, -0.8, C_OPEN, 90),
            Level::new(2, 1.2, 0.1, C_OPEN, 120),
            Level::new(3, 1.4, 0.9, C_OPEN, 150),
        ],
        source: Source::Computed(equivalence_classes),
    },
    Template {
        id: "qa.pairwise",
        title: "Комбинаторика и pairwise",
        topic: Topic::TestDesign,
        specializations: &[Qa],
        skills: &["test_design"],
        levels: &[
            Level::new(1, 0.9, -1.1, C_OPEN, 60),
            Level::new(2, 1.2, 0.2, C_OPEN, 90),
            Level::new(3, 1.4, 0.8, C_OPEN, 120),
        ],
        source: Source::Computed(pairwise),
    },
    Template {
        id: "qa.state_transition",
        title: "Тестирование переходов состояний",
        topic: Topic::TestDesign,
        specializations: &[Qa],
        skills: &["test_design"],
        levels: &[
            Level::new(1, 1.0, -0.5, C_CHOICE, 120),
            Level::new(2, 1.2, 0.3, C_OPEN, 150),
            Level::new(3, 1.5, 1.1, C_OPEN, 180),
        ],
        source: Source::Computed(state_transition),
    },
    Template {
        id: "qa.decision_table",
        title: "Таблицы решений",
        topic: Topic::TestDesign,
        specializations: &[Qa],
        skills: &["test_design"],
        levels: &[
            Level::new(1, 0.9, -1.0, C_OPEN, 60),
            Level::new(2, 1.1, -0.2, C_OPEN, 90),
            Level::new(3, 1.3, 0.6, C_OPEN, 120),
        ],
        source: Source::Computed(decision_table),
    },
    Template {
        id: "qa.api_assertions",
        title: "Проверки в API-тесте",
        topic: Topic::TestAutomation,
        specializations: &[Qa, Backend],
        skills: &["api_testing", "pytest", "python"],
        levels: &[
            Level::new(1, 1.0, -0.6, C_CHOICE, 120),
            Level::new(2, 1.2, 0.2, C_CHOICE, 150),
            Level::new(3, 1.4, 1.0, C_CHOICE, 150),
        ],
        source: Source::Computed(api_assertions),
    },
    Template {
        id: "perf.littles_law",
        title: "Нагрузочное тестирование: закон Литтла",
        topic: Topic::TestAutomation,
        specializations: &[Qa, Devops, Backend],
        skills: &["load_testing"],
        levels: &[
            Level::new(1, 1.0, -0.1, C_OPEN, 90),
            Level::new(2, 1.3, 0.6, C_OPEN, 120),
            Level::new(3, 1.4, 1.2, C_OPEN, 150),
        ],
        source: Source::Computed(littles_law),
    },
];

const FIELDS: &[(&str, i64, i64)] = &[
    ("возраст клиента", 14, 18),
    ("количество товаров в корзине", 1, 5),
    ("длина пароля", 6, 12),
    ("число гостей в бронировании", 1, 4),
    ("срок кредита в месяцах", 3, 12),
    ("процент скидки", 1, 5),
];

fn set(values: &[i64]) -> String {
    join(values, ", ")
}

fn boundary_values(level: u8, rng: &mut Rng) -> Draft {
    let &(name, lo_min, lo_max) = pick(rng, FIELDS);
    let a = range(rng, lo_min, lo_max);
    let b = a + range(rng, 10, 80);
    let mid = (a + b) / 2;
    match level {
        1 => {
            let answer = set(&[a - 1, a, b, b + 1]);
            let distractors = vec![
                set(&[a, a + 1, b - 1, b]),
                set(&[a - 1, a + 1, b - 1, b + 1]),
                set(&[a - 1, mid, b + 1]),
                set(&[a, mid, b]),
                set(&[a - 10, a, b, b + 10]),
            ];
            Draft::choice(
                format!(
                    "Поле «{name}» принимает целые значения от {a} до {b} включительно. Какой набор значений соответствует анализу граничных значений (по два значения на каждой границе: на границе и за ней)?"
                ),
                answer,
                distractors,
            )
        }
        2 => {
            let (text, valid_lo, valid_hi) = match range(rng, 0, 2) {
                0 => (format!("больше {a} и меньше {b}"), a + 1, b - 1),
                1 => (format!("от {a} включительно до {b} не включая"), a, b - 1),
                _ => (format!("больше {a} и не больше {b}"), a + 1, b),
            };
            let answer = set(&[valid_lo - 1, valid_lo, valid_hi, valid_hi + 1]);
            let distractors = vec![
                set(&[a - 1, a, b, b + 1]),
                set(&[a, a + 1, b - 1, b]),
                set(&[valid_lo, valid_lo + 1, valid_hi - 1, valid_hi]),
                set(&[a + 1, a + 2, b - 2, b - 1]),
                set(&[valid_lo - 1, valid_lo, valid_hi - 1, valid_hi]),
            ];
            Draft::choice(
                format!(
                    "Поле «{name}» принимает целые значения: {text}. Какой набор значений соответствует анализу граничных значений (по два значения на каждой границе допустимого диапазона: крайнее допустимое и ближайшее недопустимое)?"
                ),
                answer,
                distractors,
            )
        }
        _ => {
            let lo = range(rng, 1, 20) * 50;
            let hi = lo * range(rng, 10, 100);
            let money = |v: i64| format!("{}.{:02}", v.div_euclid(100), v.rem_euclid(100));
            let three = |step: i64| {
                [
                    lo * 100 - step,
                    lo * 100,
                    lo * 100 + step,
                    hi * 100 - step,
                    hi * 100,
                    hi * 100 + step,
                ]
                .iter()
                .map(|v| money(*v))
                .collect::<Vec<_>>()
                .join("; ")
            };
            let two = |step: i64| {
                [lo * 100 - step, lo * 100, hi * 100, hi * 100 + step]
                    .iter()
                    .map(|v| money(*v))
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            Draft::choice(
                format!(
                    "Сумма перевода — от {lo}.00 до {hi}.00 ₽ включительно, ввод с точностью до копеек. Какой набор соответствует трёхточечному анализу граничных значений (по три значения на каждую границу)?"
                ),
                three(1),
                vec![three(100), two(1), three(10), two(100)],
            )
        }
    }
}

struct Tiering {
    text: String,
    valid: i64,
    invalid: i64,
}

fn tiers(rng: &mut Rng, count: usize, upper_bound: bool) -> Tiering {
    let mut cuts = vec![1i64];
    for _ in 1..count {
        let last = cuts[cuts.len() - 1];
        cuts.push(last + range(rng, 5, 40) * 100);
    }
    let max = cuts[cuts.len() - 1] + range(rng, 10, 90) * 1000;
    let rates = [0, 3, 5, 7, 10, 15];
    let mut parts = Vec::new();
    for i in 0..count {
        let rate = rates[i.min(rates.len() - 1)];
        let part = if i + 1 < count {
            format!("от {} до {} ₽ — скидка {rate} %", cuts[i], cuts[i + 1] - 1)
        } else if upper_bound {
            format!("от {} до {max} ₽ — скидка {rate} %", cuts[i])
        } else {
            format!("от {} ₽ и выше — скидка {rate} %", cuts[i])
        };
        parts.push(part);
    }
    let limits = if upper_bound {
        format!("Суммы меньше 1 и больше {max} отклоняются с ошибкой.")
    } else {
        "Суммы меньше 1 отклоняются с ошибкой, верхней границы нет.".to_owned()
    };
    Tiering {
        text: format!("{}. {limits}", parts.join("; ")),
        valid: count as i64,
        invalid: if upper_bound { 2 } else { 1 },
    }
}

fn equivalence_classes(level: u8, rng: &mut Rng) -> Draft {
    match level {
        1 | 2 => {
            let count = if level == 1 {
                range(rng, 2, 3)
            } else {
                range(rng, 3, 4)
            } as usize;
            let upper = level == 1 || chance(rng, 0.3);
            let t = tiers(rng, count, upper);
            Draft::integer(
                format!(
                    "Сумма заказа — целое число рублей. Правила: {} \
                     Сколько классов эквивалентности (валидных и невалидных) нужно выделить для поля «сумма заказа»? Нечисловой ввод не рассматривайте.",
                    t.text
                ),
                t.valid + t.invalid,
            )
        }
        _ => {
            let (count, upper) = (range(rng, 2, 4) as usize, chance(rng, 0.5));
            let t = tiers(rng, count, upper);
            let qty_hi = range(rng, 10, 99);
            let (qty_text, qty_valid, qty_invalid) = if chance(rng, 0.5) {
                (
                    format!(
                        "целое от 1 до {qty_hi}, обработка одинаковая; другие значения отклоняются"
                    ),
                    1,
                    2,
                )
            } else {
                (
                    format!(
                        "целое от 1 до {qty_hi}; от 1 до 9 — обычная доставка, от 10 до {qty_hi} — грузовая; другие значения отклоняются"
                    ),
                    2,
                    2,
                )
            };
            let answer = t.valid.max(qty_valid) + t.invalid + qty_invalid;
            Draft::integer(
                format!(
                    "Форма заказа содержит два независимых поля.\n\n\
                     - «Сумма» (целое число рублей): {}\n- «Количество»: {qty_text}.\n\n\
                     Тест задаёт оба поля. Сколько тестов минимально нужно, чтобы покрыть все классы эквивалентности обоих полей, \
                     если в одном тесте допускается не больше одного невалидного значения (остальные поля — валидные)?",
                    t.text
                ),
                answer,
            )
        }
    }
}

const PARAMETERS: &[(&str, &[&str])] = &[
    (
        "Браузер",
        &["Chrome", "Firefox", "Safari", "Edge", "Яндекс Браузер"],
    ),
    ("ОС", &["Windows", "macOS", "Linux", "Android", "iOS"]),
    ("Язык интерфейса", &["ru", "en", "kk", "uz"]),
    ("Способ оплаты", &["карта", "СБП", "наличные", "рассрочка"]),
    ("Тип пользователя", &["гость", "клиент", "VIP"]),
    (
        "Доставка",
        &["курьер", "самовывоз", "почта", "пункт выдачи"],
    ),
];

fn pairwise(level: u8, rng: &mut Rng) -> Draft {
    let n = if level == 1 {
        range(rng, 3, 4)
    } else {
        range(rng, 3, 5)
    } as usize;
    let params = pick_n(rng, PARAMETERS, n);
    let mut counts = Vec::new();
    let mut lines = Vec::new();
    for (name, values) in &params {
        let k = range(rng, 2, values.len() as i64) as usize;
        let chosen = pick_n(rng, values, k);
        counts.push(k as i64);
        lines.push(format!("- {name}: {}", chosen.join(", ")));
    }
    let list = lines.join("\n");
    match level {
        1 => Draft::integer(
            format!(
                "Параметры конфигурации для тестирования формы оплаты:\n\n{list}\n\nСколько тестов понадобится для полного перебора всех комбинаций?"
            ),
            counts.iter().product(),
        ),
        2 => {
            let mut sorted = counts.clone();
            sorted.sort_unstable_by(|a, b| b.cmp(a));
            Draft::integer(
                format!(
                    "Параметры конфигурации:\n\n{list}\n\nКакова нижняя граница числа тестов для попарного (pairwise) покрытия — меньше тестов не получится ни при каком наборе?"
                ),
                sorted[0] * sorted[1],
            )
        }
        _ => {
            let mut pairs = 0;
            for i in 0..counts.len() {
                for j in i + 1..counts.len() {
                    pairs += counts[i] * counts[j];
                }
            }
            Draft::integer(
                format!(
                    "Параметры конфигурации:\n\n{list}\n\nСколько различных пар значений (по всем парам параметров) должен покрыть набор тестов для полного pairwise-покрытия?"
                ),
                pairs,
            )
        }
    }
}

const STATES: &[&str] = &[
    "Новый",
    "Оплачен",
    "Собран",
    "Отправлен",
    "Доставлен",
    "Отменён",
    "Возврат",
];
const EVENTS: &[&str] = &[
    "оплатить",
    "собрать",
    "отправить",
    "доставить",
    "отменить",
    "вернуть",
];

struct Machine {
    transitions: Vec<(usize, usize, usize)>,
}

impl Machine {
    fn random(rng: &mut Rng) -> Self {
        let mut t = vec![(0, 0, 1), (1, 1, 2), (2, 2, 3), (3, 3, 4), (0, 4, 5)];
        let optional = [(1, 4, 5), (2, 4, 5), (3, 4, 5), (4, 5, 6), (3, 5, 6)];
        for o in optional {
            if chance(rng, 0.5) {
                t.push(o);
            }
        }
        t.sort_unstable();
        Self { transitions: t }
    }

    fn step(&self, state: usize, event: usize) -> Option<usize> {
        self.transitions
            .iter()
            .find(|(s, e, _)| *s == state && *e == event)
            .map(|t| t.2)
    }

    fn accepts(&self, events: &[usize]) -> bool {
        let mut state = 0;
        for &e in events {
            match self.step(state, e) {
                Some(next) => state = next,
                None => return false,
            }
        }
        true
    }

    fn table(&self) -> String {
        let rows: Vec<Vec<String>> = self
            .transitions
            .iter()
            .map(|(s, e, t)| {
                vec![
                    STATES[*s].to_owned(),
                    EVENTS[*e].to_owned(),
                    STATES[*t].to_owned(),
                ]
            })
            .collect();
        md_table(&["Из состояния", "Событие", "В состояние"], &rows)
    }

    fn used_states(&self) -> usize {
        let mut s: Vec<usize> = self.transitions.iter().flat_map(|t| [t.0, t.2]).collect();
        s.sort_unstable();
        s.dedup();
        s.len()
    }
}

fn state_transition(level: u8, rng: &mut Rng) -> Draft {
    let machine = Machine::random(rng);
    let table = machine.table();
    match level {
        1 => {
            let mut path = Vec::new();
            let mut state = 0;
            let len = range(rng, 2, 4);
            for _ in 0..len {
                let options: Vec<&(usize, usize, usize)> = machine
                    .transitions
                    .iter()
                    .filter(|t| t.0 == state)
                    .collect();
                if options.is_empty() {
                    break;
                }
                let t = *pick(rng, &options);
                path.push(t.1);
                state = t.2;
            }
            let render = |p: &[usize]| p.iter().map(|e| EVENTS[*e]).collect::<Vec<_>>().join(" → ");
            let mut wrong: Vec<Vec<usize>> = Vec::new();
            for _ in 0..200 {
                if wrong.len() >= 6 {
                    break;
                }
                let mut candidate = path.clone();
                match range(rng, 0, 3) {
                    0 if candidate.len() > 1 => {
                        let i = range(rng, 0, candidate.len() as i64 - 1) as usize;
                        candidate.remove(i);
                    }
                    1 => candidate.push(range(rng, 0, EVENTS.len() as i64 - 1) as usize),
                    2 if candidate.len() > 1 => {
                        let i = range(rng, 0, candidate.len() as i64 - 2) as usize;
                        candidate.swap(i, i + 1);
                    }
                    _ => {
                        let i = range(rng, 0, candidate.len() as i64 - 1) as usize;
                        candidate[i] = range(rng, 0, EVENTS.len() as i64 - 1) as usize;
                    }
                }
                if !machine.accepts(&candidate)
                    && !wrong.contains(&candidate)
                    && !candidate.is_empty()
                {
                    wrong.push(candidate);
                }
            }
            Draft::choice(
                format!(
                    "Жизненный цикл заказа задан таблицей допустимых переходов (начальное состояние — «Новый»):\n\n{table}\n\
                     Какая последовательность событий допустима, если начать с состояния «Новый»?"
                ),
                render(&path),
                wrong.iter().map(|p| render(p)),
            )
        }
        2 => {
            let states = machine.used_states() as i64;
            let invalid = states * EVENTS.len() as i64 - machine.transitions.len() as i64;
            Draft::integer(
                format!(
                    "Жизненный цикл заказа задан таблицей допустимых переходов:\n\n{table}\n\
                     События: {}. Любое сочетание (состояние из таблицы, событие), которого нет в таблице, недопустимо и должно отклоняться. \
                     Сколько негативных тестов нужно, чтобы проверить каждое недопустимое сочетание ровно один раз?",
                    EVENTS.join(", ")
                ),
                invalid,
            )
        }
        _ => {
            let pairs: i64 = (0..STATES.len())
                .map(|s| {
                    let incoming = machine.transitions.iter().filter(|t| t.2 == s).count() as i64;
                    let outgoing = machine.transitions.iter().filter(|t| t.0 == s).count() as i64;
                    incoming * outgoing
                })
                .sum();
            Draft::integer(
                format!(
                    "Жизненный цикл заказа задан таблицей допустимых переходов:\n\n{table}\n\
                     Сколько существует различных последовательностей из двух подряд идущих допустимых переходов (именно их нужно покрыть для 1-switch coverage)?"
                ),
                pairs,
            )
        }
    }
}

const CONDITIONS: &[&str] = &[
    "товар есть на складе",
    "у клиента есть промокод",
    "сумма заказа больше 3000 ₽",
    "доставка в пределах города",
    "клиент оформил подписку",
    "оплата картой онлайн",
];

fn decision_table(level: u8, rng: &mut Rng) -> Draft {
    match level {
        1 => {
            let n = range(rng, 2, 4) as usize;
            let conds = pick_n(rng, CONDITIONS, n);
            Draft::integer(
                format!(
                    "Логика расчёта стоимости зависит от булевых условий:\n\n{}\n\nСколько правил (столбцов) будет в полной таблице решений?",
                    conds
                        .iter()
                        .map(|c| format!("- {c}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                1 << n,
            )
        }
        2 => {
            let n = range(rng, 1, 3) as usize;
            let conds = pick_n(rng, CONDITIONS, n);
            let (multi, k) = *pick(
                rng,
                &[
                    ("тип клиента: новый, постоянный или VIP", 3i64),
                    (
                        "регион: Москва, Санкт-Петербург, другой город или другая страна",
                        4,
                    ),
                    ("способ доставки: курьер, самовывоз или почта", 3),
                ],
            );
            Draft::integer(
                format!(
                    "Логика расчёта стоимости зависит от условий:\n\n- {multi}\n{}\n\nСколько правил будет в полной таблице решений?",
                    conds
                        .iter()
                        .map(|c| format!("- {c} (да/нет)"))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                k << n,
            )
        }
        _ => {
            let n = range(rng, 3, 5) as usize;
            let dominant = range(rng, 1, 2) as usize;
            let conds = pick_n(rng, CONDITIONS, n);
            let gates: Vec<String> = conds[..dominant]
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    if i == 0 {
                        format!("если неверно, что {c}, — отказ независимо от остальных условий")
                    } else {
                        format!("если первое условие выполнено, но неверно, что {c}, — тоже отказ независимо от остальных")
                    }
                })
                .collect();
            let answer = dominant as i64 + (1i64 << (n - dominant));
            Draft::integer(
                format!(
                    "Таблица решений строится по {n} булевым условиям:\n\n{}\n\nПравила: {}. Остальные комбинации дают разные результаты. \
                     Сколько правил останется в таблице после схлопывания столбцов с безразличными («—») условиями?",
                    conds
                        .iter()
                        .map(|c| format!("- {c}"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    gates.join("; ")
                ),
                answer,
            )
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
enum PyVal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    None,
    Missing,
}

impl PyVal {
    fn json(&self) -> String {
        match self {
            PyVal::Int(v) => v.to_string(),
            PyVal::Float(v) => format!("{v:.1}"),
            PyVal::Str(s) => format!("\"{s}\""),
            PyVal::Bool(b) => b.to_string(),
            PyVal::None | PyVal::Missing => "null".to_owned(),
        }
    }

    /// Python `==` between JSON-decoded values: ints and floats compare numerically and
    /// `bool` is a subclass of `int`, so `True == 1`.
    fn py_eq(&self, other: &PyVal) -> bool {
        let num = |v: &PyVal| match v {
            PyVal::Int(i) => Some(*i as f64),
            PyVal::Float(f) => Some(*f),
            PyVal::Bool(b) => Some(f64::from(u8::from(*b))),
            _ => None,
        };
        match (self, other) {
            (PyVal::Str(a), PyVal::Str(b)) => a == b,
            (PyVal::None, PyVal::None) => true,
            _ => matches!((num(self), num(other)), (Some(a), Some(b)) if a == b),
        }
    }

    fn literal(&self) -> String {
        match self {
            PyVal::Bool(true) => "True".to_owned(),
            PyVal::Bool(false) => "False".to_owned(),
            PyVal::None | PyVal::Missing => "None".to_owned(),
            PyVal::Str(s) => format!("\"{s}\""),
            other => other.json(),
        }
    }
}

struct Check {
    line: String,
    passes: bool,
}

fn api_assertions(level: u8, rng: &mut Rng) -> Draft {
    let items_n = range(rng, 1, 4);
    let price = range(rng, 2, 30) * 50;
    let total = price * items_n;
    let status_name = *pick(rng, &["new", "created", "pending"]);
    let expected_code = *pick(rng, &[200i64, 201]);
    let resource = *pick(rng, &["orders", "carts", "invoices"]);

    let fail_count = match level {
        2 => 2,
        _ => 1,
    };
    let order: Vec<usize> = shuffled(rng, (0..5).collect());
    let failing: Vec<usize> = pick_n(rng, &order[..4], fail_count);

    let mut checks: Vec<Check> = Vec::new();
    let mut fields: Vec<(String, PyVal)> =
        vec![("id".to_owned(), PyVal::Int(range(rng, 100, 999)))];
    let mut actual_code = expected_code;
    for &kind in &order {
        let fail = failing.contains(&kind);
        match kind {
            0 => {
                actual_code = if fail {
                    if expected_code == 201 { 200 } else { 201 }
                } else {
                    expected_code
                };
                checks.push(Check {
                    line: format!("assert resp.status_code == {expected_code}"),
                    passes: !fail,
                });
            }
            1 => {
                let actual = if fail {
                    *pick(rng, &["draft", "processing", "New"])
                } else {
                    status_name
                };
                fields.push(("status".into(), PyVal::Str(actual.into())));
                checks.push(Check {
                    line: format!("assert body[\"status\"] == \"{status_name}\""),
                    passes: !fail,
                });
            }
            2 => {
                let actual = if fail { items_n + 1 } else { items_n };
                fields.push(("items_count".into(), PyVal::Int(actual)));
                checks.push(Check {
                    line: format!("assert body[\"items_count\"] == {items_n}"),
                    passes: !fail,
                });
            }
            3 => {
                let actual = match (level, fail) {
                    (3, true) => PyVal::Str(total.to_string()),
                    (3, false) => PyVal::Float(total as f64),
                    (_, true) => PyVal::Int(total + price / 2),
                    (_, false) => PyVal::Int(total),
                };
                let expected = PyVal::Int(total);
                fields.push(("total".into(), actual.clone()));
                checks.push(Check {
                    line: format!("assert body[\"total\"] == {}", expected.literal()),
                    passes: actual.py_eq(&expected),
                });
            }
            _ => {
                let (key, actual, expected): (&str, PyVal, PyVal) = if level == 3 {
                    if fail {
                        ("discount", PyVal::Missing, PyVal::None)
                    } else if chance(rng, 0.5) {
                        ("paid", PyVal::Bool(true), PyVal::Int(1))
                    } else {
                        ("discount", PyVal::None, PyVal::None)
                    }
                } else if fail {
                    (
                        "currency",
                        PyVal::Str("USD".into()),
                        PyVal::Str("RUB".into()),
                    )
                } else {
                    (
                        "currency",
                        PyVal::Str("RUB".into()),
                        PyVal::Str("RUB".into()),
                    )
                };
                if actual != PyVal::Missing {
                    fields.push((key.into(), actual.clone()));
                }
                let line = if expected == PyVal::None {
                    format!("assert body[\"{key}\"] is None")
                } else {
                    format!("assert body[\"{key}\"] == {}", expected.literal())
                };
                let passes = actual != PyVal::Missing && actual.py_eq(&expected);
                checks.push(Check { line, passes });
            }
        }
    }
    let Some(first_failing) = checks.iter().position(|c| !c.passes) else {
        return Draft::choice(
            "Какая проверка упадёт первой?",
            "assert resp.status_code == 201",
            [
                "assert body[\"status\"] == \"new\"",
                "assert body[\"total\"] == 100",
                "assert body[\"id\"] > 0",
            ]
            .map(str::to_owned),
        );
    };
    let body = fields
        .iter()
        .map(|(k, v)| format!("  \"{k}\": {}", v.json()))
        .collect::<Vec<_>>()
        .join(",\n");
    let reason = if actual_code == 201 { "Created" } else { "OK" };
    let response = format!(
        "HTTP/1.1 {actual_code} {reason}\nContent-Type: application/json\n\n{{\n{body}\n}}"
    );
    let asserts: String = checks.iter().map(|c| format!("    {}\n", c.line)).collect();
    let code = format!(
        "def test_create_{resource}(client):\n    resp = client.post(\"/api/{resource}\", json=PAYLOAD)\n    body = resp.json()\n{asserts}"
    );
    let answer = checks[first_failing].line.clone();
    let distractors = checks
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != first_failing)
        .map(|(_, c)| c.line.clone());
    Draft::choice(
        format!(
            "Сервер вернул ответ:\n\n```http\n{response}\n```\n\nНа какой строке тест упадёт? Тест останавливается на первой неуспешной проверке."
        ),
        answer,
        distractors,
    )
    .code("python", code)
}

fn littles_law(level: u8, rng: &mut Rng) -> Draft {
    let r: f64 = *pick(rng, &[0.2, 0.25, 0.4, 0.5, 0.8, 1.2, 1.5, 2.0]);
    match level {
        1 => {
            let x = range(rng, 2, 40) * 10;
            Draft::number(
                format!(
                    "Сервис стабильно обрабатывает в среднем {x} запросов в секунду, среднее время ответа — {} с. Сколько запросов в среднем одновременно находятся в обработке?",
                    fmt_number(r)
                ),
                x as f64 * r,
                0.01,
            )
            .hint("Число")
        }
        2 => {
            let x = range(rng, 2, 30) * 10;
            let z = range(rng, 1, 10) as f64;
            Draft::number(
                format!(
                    "Нагрузочный тест в закрытой модели: каждый виртуальный пользователь отправляет запрос, ждёт ответ (в среднем {} с), затем делает паузу (think time) {} с и повторяет. \
                     Сколько виртуальных пользователей нужно, чтобы получить {x} запросов в секунду?",
                    fmt_number(r),
                    fmt_number(z)
                ),
                x as f64 * (r + z),
                0.01,
            )
            .hint("Число")
        }
        _ => {
            let users = range(rng, 5, 60) * 10;
            let z = range(rng, 1, 10) as f64;
            let x = users as f64 / (r + z);
            Draft::number(
                format!(
                    "В закрытой модели нагрузки {users} виртуальных пользователей; среднее время ответа — {} с, think time — {} с. \
                     Какую пропускную способность (запросов в секунду) создаст тест в установившемся режиме?",
                    fmt_number(r),
                    fmt_number(z)
                ),
                (x * 10.0).round() / 10.0,
                0.051,
            )
            .hint("Число, округлите до десятых")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_equality_semantics() {
        assert!(PyVal::Float(900.0).py_eq(&PyVal::Int(900)));
        assert!(PyVal::Bool(true).py_eq(&PyVal::Int(1)));
        assert!(!PyVal::Str("900".into()).py_eq(&PyVal::Int(900)));
        assert!(PyVal::None.py_eq(&PyVal::None));
    }

    #[test]
    fn state_machine_walks() {
        let m = Machine {
            transitions: vec![(0, 0, 1), (1, 1, 2), (0, 4, 5)],
        };
        assert!(m.accepts(&[0, 1]));
        assert!(!m.accepts(&[1]));
        assert!(!m.accepts(&[4, 0]));
    }
}
