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
    ("длина никнейма", 3, 5),
    ("срок вклада в месяцах", 1, 6),
    ("количество дней аренды", 1, 3),
    ("число участников команды", 2, 5),
    ("количество билетов в заказе", 1, 2),
    ("вес посылки в килограммах", 1, 3),
];

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

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
            let inside = [
                lo * 100,
                lo * 100 + 1,
                lo * 100 + 2,
                hi * 100 - 2,
                hi * 100 - 1,
                hi * 100,
            ]
            .iter()
            .map(|v| money(*v))
            .collect::<Vec<_>>()
            .join("; ");
            let third = match range(rng, 0, 2) {
                0 => three(10),
                1 => two(100),
                _ => inside,
            };
            let subject = *pick(
                rng,
                &[
                    "Сумма перевода",
                    "Сумма платежа",
                    "Сумма пополнения кошелька",
                    "Сумма вывода средств",
                    "Сумма заказа",
                    "Сумма возврата",
                ],
            );
            let precision = *pick(
                rng,
                &[
                    "ввод с точностью до копеек",
                    "вводится с точностью до копейки",
                    "допускаются копейки",
                ],
            );
            Draft::choice(
                format!(
                    "{subject} — от {lo}.00 до {hi}.00 ₽ включительно, {precision}. Какой набор соответствует трёхточечному анализу граничных значений (по три значения на каждую границу)?"
                ),
                three(1),
                vec![three(100), two(1), third],
            )
        }
    }
}

struct Tiering {
    text: String,
    valid: i64,
    invalid: i64,
}

/// Field name, the per-tier benefit, and whether the rate falls as the amount grows.
const TIERED_FIELDS: &[(&str, &str, bool)] = &[
    ("сумма заказа", "скидка", false),
    ("сумма покупки", "кешбэк", false),
    ("стоимость корзины", "скидка", false),
    ("сумма пополнения", "бонус", false),
    ("сумма перевода", "комиссия", true),
    ("сумма платежа", "комиссия", true),
];

/// Distinct rates keep every tier a separate class: equal neighbours would merge.
const TIER_RATES: &[i64] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 15];

fn tiers(
    rng: &mut Rng,
    count: usize,
    upper_bound: bool,
    benefit: &str,
    descending: bool,
) -> Tiering {
    let mut cuts = vec![1i64];
    for _ in 1..count {
        let last = cuts[cuts.len() - 1];
        cuts.push(last + range(rng, 5, 40) * 100);
    }
    let max = cuts[cuts.len() - 1] + range(rng, 10, 90) * 1000;
    let mut rates = pick_n(rng, TIER_RATES, count);
    rates.sort_unstable();
    if descending {
        rates.reverse();
    }
    let mut parts = Vec::new();
    for (i, rate) in rates.iter().enumerate() {
        let part = if i + 1 < count {
            format!(
                "от {} до {} ₽ — {benefit} {rate} %",
                cuts[i],
                cuts[i + 1] - 1
            )
        } else if upper_bound {
            format!("от {} до {max} ₽ — {benefit} {rate} %", cuts[i])
        } else {
            format!("от {} ₽ и выше — {benefit} {rate} %", cuts[i])
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
            let &(field, benefit, descending) = pick(rng, TIERED_FIELDS);
            let t = tiers(rng, count, upper, benefit, descending);
            let question = if chance(rng, 0.5) {
                "Сколько классов эквивалентности (валидных и невалидных) нужно выделить"
            } else {
                "Сколько всего классов эквивалентности — валидных и невалидных — нужно выделить"
            };
            Draft::integer(
                format!(
                    "{} — целое число рублей. Правила: {} \
                     {question} для поля «{field}»? Нечисловой ввод не рассматривайте.",
                    capitalize(field),
                    t.text
                ),
                t.valid + t.invalid,
            )
        }
        _ => {
            let (count, upper) = (range(rng, 2, 4) as usize, chance(rng, 0.5));
            let benefit = *pick(rng, &["скидка", "кешбэк"]);
            let t = tiers(rng, count, upper, benefit, false);
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

struct Lifecycle {
    /// Genitive: "жизненный цикл заказа".
    entity: &'static str,
    /// Prepositional phrases: "в интернет-магазине".
    places: &'static [&'static str],
    /// 0..=4 are the happy path, 5 is reached by event 4 (cancel), 6 by event 5 (reversal).
    states: [&'static str; 7],
    events: [&'static str; 6],
}

const LIFECYCLES: &[Lifecycle] = &[
    Lifecycle {
        entity: "заказа",
        places: &[
            "в интернет-магазине",
            "на маркетплейсе",
            "в магазине электроники",
            "в онлайн-гипермаркете",
        ],
        states: [
            "Новый",
            "Оплачен",
            "Собран",
            "Отправлен",
            "Доставлен",
            "Отменён",
            "Возврат",
        ],
        events: [
            "оплатить",
            "собрать",
            "отправить",
            "доставить",
            "отменить",
            "вернуть",
        ],
    },
    Lifecycle {
        entity: "дефекта",
        places: &[
            "в баг-трекере",
            "в трекере задач команды",
            "в системе учёта дефектов",
        ],
        states: [
            "Новый",
            "Подтверждён",
            "В работе",
            "Исправлен",
            "Закрыт",
            "Отклонён",
            "Переоткрыт",
        ],
        events: [
            "подтвердить",
            "взять в работу",
            "исправить",
            "проверить",
            "отклонить",
            "переоткрыть",
        ],
    },
    Lifecycle {
        entity: "обращения",
        places: &["в службе поддержки", "в сервис-деске", "в helpdesk-системе"],
        states: [
            "Открыто",
            "Назначено",
            "В работе",
            "Решено",
            "Закрыто",
            "Отклонено",
            "Переоткрыто",
        ],
        events: [
            "назначить",
            "взять в работу",
            "решить",
            "закрыть",
            "отклонить",
            "переоткрыть",
        ],
    },
    Lifecycle {
        entity: "документа",
        places: &[
            "в системе электронного документооборота",
            "в корпоративном портале",
            "в сервисе согласования договоров",
        ],
        states: [
            "Черновик",
            "На согласовании",
            "Согласован",
            "Подписан",
            "Опубликован",
            "Отозван",
            "Аннулирован",
        ],
        events: [
            "отправить на согласование",
            "согласовать",
            "подписать",
            "опубликовать",
            "отозвать",
            "аннулировать",
        ],
    },
    Lifecycle {
        entity: "посылки",
        places: &[
            "в службе доставки",
            "в почтовом сервисе",
            "в логистической системе",
        ],
        states: [
            "Принята",
            "На сортировке",
            "В пути",
            "В пункте выдачи",
            "Вручена",
            "Отменена",
            "Возвращена",
        ],
        events: [
            "передать на сортировку",
            "отправить",
            "доставить в пункт выдачи",
            "вручить",
            "отменить",
            "вернуть отправителю",
        ],
    },
    Lifecycle {
        entity: "платежа",
        places: &[
            "в платёжном сервисе",
            "в процессинге банка",
            "в платёжном шлюзе",
        ],
        states: [
            "Создан",
            "Авторизован",
            "Подтверждён",
            "Списан",
            "Зачислен",
            "Отменён",
            "Возвращён",
        ],
        events: [
            "авторизовать",
            "подтвердить",
            "списать",
            "зачислить",
            "отменить",
            "вернуть",
        ],
    },
    Lifecycle {
        entity: "релиза",
        places: &[
            "в CI/CD-системе",
            "в системе управления релизами",
            "в пайплайне доставки",
        ],
        states: [
            "Запланирован",
            "Собран",
            "Протестирован",
            "На стейджинге",
            "В продакшене",
            "Отменён",
            "Откачен",
        ],
        events: [
            "собрать",
            "протестировать",
            "выкатить на стейджинг",
            "выкатить в продакшен",
            "отменить",
            "откатить",
        ],
    },
    Lifecycle {
        entity: "бронирования",
        places: &[
            "в системе бронирования отеля",
            "в сервисе аренды жилья",
            "в гостиничной CRM",
        ],
        states: [
            "Создано",
            "Подтверждено",
            "Оплачено",
            "Активно",
            "Завершено",
            "Отменено",
            "Спор",
        ],
        events: [
            "подтвердить",
            "оплатить",
            "заселить",
            "выселить",
            "отменить",
            "открыть спор",
        ],
    },
    Lifecycle {
        entity: "отклика на вакансию",
        places: &[
            "на платформе найма",
            "в ATS-системе",
            "на карьерном портале",
        ],
        states: [
            "Новый",
            "Просмотрен",
            "Интервью",
            "Оффер",
            "Принят",
            "Отказ",
            "Отозван",
        ],
        events: [
            "просмотреть",
            "пригласить на интервью",
            "сделать оффер",
            "принять оффер",
            "отказать",
            "отозвать",
        ],
    },
    Lifecycle {
        entity: "заявления на выплату",
        places: &[
            "в страховой компании",
            "в системе урегулирования убытков",
            "в личном кабинете страхователя",
        ],
        states: [
            "Подано",
            "Принято",
            "Проверено",
            "Одобрено",
            "Выплачено",
            "Отклонено",
            "Оспорено",
        ],
        events: [
            "принять",
            "проверить",
            "одобрить",
            "выплатить",
            "отклонить",
            "оспорить",
        ],
    },
];

const TRANSITION_HEADERS: &[[&str; 3]] = &[
    ["Из состояния", "Событие", "В состояние"],
    ["Текущее состояние", "Событие", "Новое состояние"],
    ["Исходное состояние", "Событие", "Целевое состояние"],
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
        // Rows stay grouped by source state; only the order inside a group varies.
        let mut t = shuffled(rng, t);
        t.sort_by_key(|row| row.0);
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

    fn table(&self, lc: &Lifecycle, headers: &[&str; 3]) -> String {
        let rows: Vec<Vec<String>> = self
            .transitions
            .iter()
            .map(|(s, e, t)| {
                vec![
                    lc.states[*s].to_owned(),
                    lc.events[*e].to_owned(),
                    lc.states[*t].to_owned(),
                ]
            })
            .collect();
        md_table(headers, &rows)
    }

    fn used_states(&self) -> usize {
        let mut s: Vec<usize> = self.transitions.iter().flat_map(|t| [t.0, t.2]).collect();
        s.sort_unstable();
        s.dedup();
        s.len()
    }
}

fn state_transition(level: u8, rng: &mut Rng) -> Draft {
    let lc = pick(rng, LIFECYCLES);
    let place = *pick(rng, lc.places);
    let entity = lc.entity;
    let machine = Machine::random(rng);
    let table = machine.table(lc, pick(rng, TRANSITION_HEADERS));
    let intro = match range(rng, 0, 2) {
        0 => format!("Жизненный цикл {entity} {place} задан таблицей допустимых переходов"),
        1 => format!(
            "{} жизненный цикл {entity} описан таблицей допустимых переходов",
            capitalize(place)
        ),
        _ => format!("Допустимые переходы между состояниями {entity} {place} заданы таблицей"),
    };
    let start = lc.states[0];
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
            let events = lc.events.len() as i64;
            let render = |p: &[usize]| {
                p.iter()
                    .map(|e| lc.events[*e])
                    .collect::<Vec<_>>()
                    .join(" → ")
            };
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
                    1 => candidate.push(range(rng, 0, events - 1) as usize),
                    2 if candidate.len() > 1 => {
                        let i = range(rng, 0, candidate.len() as i64 - 2) as usize;
                        candidate.swap(i, i + 1);
                    }
                    _ => {
                        let i = range(rng, 0, candidate.len() as i64 - 1) as usize;
                        candidate[i] = range(rng, 0, events - 1) as usize;
                    }
                }
                if !machine.accepts(&candidate)
                    && !wrong.contains(&candidate)
                    && !candidate.is_empty()
                {
                    wrong.push(candidate);
                }
            }
            let question = if chance(rng, 0.5) {
                "Какая последовательность событий допустима"
            } else {
                "Какую последовательность событий система примет целиком"
            };
            Draft::choice(
                format!(
                    "{intro} (начальное состояние — «{start}»):\n\n{table}\n\
                     {question}, если начать с состояния «{start}»?"
                ),
                render(&path),
                wrong.iter().map(|p| render(p)),
            )
        }
        2 => {
            let states = machine.used_states() as i64;
            let invalid = states * lc.events.len() as i64 - machine.transitions.len() as i64;
            let events = shuffled(rng, lc.events.to_vec()).join(", ");
            let question = if chance(rng, 0.5) {
                "Сколько негативных тестов нужно, чтобы проверить каждое недопустимое сочетание ровно один раз?"
            } else {
                "Сколько негативных тестов понадобится, если каждое недопустимое сочетание проверяется ровно одним тестом?"
            };
            Draft::integer(
                format!(
                    "{intro}:\n\n{table}\n\
                     События: {events}. Любое сочетание (состояние из таблицы, событие), которого нет в таблице, недопустимо и должно отклоняться. \
                     {question}"
                ),
                invalid,
            )
        }
        _ => {
            let pairs: i64 = (0..lc.states.len())
                .map(|s| {
                    let incoming = machine.transitions.iter().filter(|t| t.2 == s).count() as i64;
                    let outgoing = machine.transitions.iter().filter(|t| t.0 == s).count() as i64;
                    incoming * outgoing
                })
                .sum();
            let question = if chance(rng, 0.5) {
                "Сколько существует различных последовательностей из двух подряд идущих допустимых переходов (именно их нужно покрыть для 1-switch coverage)?"
            } else {
                "Для 1-switch coverage нужно покрыть все различные последовательности из двух подряд идущих допустимых переходов. Сколько их?"
            };
            Draft::integer(format!("{intro}:\n\n{table}\n{question}"), pairs)
        }
    }
}

struct Cond {
    /// `#` is replaced by one of `values`.
    text: &'static str,
    values: &'static [i64],
    /// Plausible as a precondition whose failure alone decides the outcome.
    gate: bool,
}

const fn flag(text: &'static str) -> Cond {
    Cond {
        text,
        values: &[],
        gate: false,
    }
}

const fn gate(text: &'static str) -> Cond {
    Cond {
        text,
        values: &[],
        gate: true,
    }
}

const fn num(text: &'static str, values: &'static [i64]) -> Cond {
    Cond {
        text,
        values,
        gate: false,
    }
}

impl Cond {
    fn render(&self, rng: &mut Rng) -> String {
        if self.values.is_empty() {
            self.text.to_owned()
        } else {
            self.text.replace('#', &pick(rng, self.values).to_string())
        }
    }
}

struct Rulebook {
    /// Singular nominative subjects: "<subject> зависит от условий".
    subjects: &'static [&'static str],
    refusal: &'static str,
    conditions: &'static [Cond],
    /// Multi-valued conditions: label and the value pool (at least 4).
    multi: &'static [(&'static str, &'static [&'static str])],
}

const RULEBOOKS: &[Rulebook] = &[
    Rulebook {
        subjects: &[
            "Итоговая стоимость заказа",
            "Логика расчёта стоимости заказа",
            "Размер скидки на заказ",
            "Цена заказа в интернет-магазине",
        ],
        refusal: "отказ в оформлении заказа",
        conditions: &[
            gate("товар есть на складе"),
            gate("адрес доставки входит в зону обслуживания"),
            gate("клиент подтвердил номер телефона"),
            flag("у клиента есть промокод"),
            num(
                "сумма заказа больше # ₽",
                &[1000, 1500, 2000, 2500, 3000, 4000, 5000, 7000],
            ),
            flag("доставка в пределах города"),
            flag("клиент оформил подписку"),
            flag("оплата картой онлайн"),
            num("в заказе больше # товаров", &[2, 3, 4, 5, 7, 9]),
            flag("клиент делает первый заказ"),
            flag("заказ оформлен в выходной день"),
        ],
        multi: &[
            (
                "тип клиента",
                &[
                    "новый",
                    "постоянный",
                    "VIP",
                    "корпоративный",
                    "сотрудник компании",
                ],
            ),
            (
                "способ доставки",
                &["курьер", "самовывоз", "почта", "пункт выдачи", "постамат"],
            ),
        ],
    },
    Rulebook {
        subjects: &[
            "Решение о выдаче кредита",
            "Одобрение кредитной заявки",
            "Ставка по потребительскому кредиту",
            "Кредитный лимит клиента",
        ],
        refusal: "отказ в выдаче кредита",
        conditions: &[
            Cond {
                text: "клиенту исполнилось # лет",
                values: &[18, 19, 20, 25, 26, 27, 28, 30],
                gate: true,
            },
            gate("кредитная история клиента положительная"),
            gate("доход клиента подтверждён справкой"),
            num(
                "ежемесячный доход больше # тыс. ₽",
                &[30, 40, 50, 60, 80, 100, 120, 150],
            ),
            flag("есть поручитель"),
            num(
                "стаж на последнем месте работы больше # месяцев",
                &[3, 6, 9, 12],
            ),
            num("сумма кредита больше # тыс. ₽", &[100, 200, 300, 500, 700]),
            flag("есть залог"),
            flag("клиент получает зарплату на карту банка"),
            flag("у клиента уже есть кредит в этом банке"),
        ],
        multi: &[
            (
                "категория заёмщика",
                &[
                    "работник по найму",
                    "ИП",
                    "самозанятый",
                    "пенсионер",
                    "студент",
                ],
            ),
            (
                "цель кредита",
                &[
                    "ремонт",
                    "покупка автомобиля",
                    "образование",
                    "рефинансирование",
                    "покупка техники",
                ],
            ),
        ],
    },
    Rulebook {
        subjects: &[
            "Доступ к разделу «Отчёты»",
            "Доступ к разделу «Платежи»",
            "Доступ к панели администратора",
            "Доступ к выгрузке данных через API",
            "Доступ к настройкам проекта",
        ],
        refusal: "отказ в доступе",
        conditions: &[
            gate("пользователь авторизован"),
            gate("email пользователя подтверждён"),
            gate("подписка организации оплачена"),
            flag("у пользователя роль администратора"),
            flag("включена двухфакторная аутентификация"),
            flag("запрос пришёл из корпоративной сети"),
            num(
                "аккаунт создан больше # дней назад",
                &[5, 7, 10, 14, 30, 60, 90],
            ),
            flag("пользователь принял условия соглашения"),
            flag("пользователь входит в команду проекта"),
            num(
                "с последнего входа прошло меньше # минут",
                &[15, 30, 45, 60],
            ),
        ],
        multi: &[
            (
                "роль пользователя",
                &[
                    "гость",
                    "читатель",
                    "редактор",
                    "модератор",
                    "администратор",
                ],
            ),
            (
                "тариф",
                &["бесплатный", "базовый", "профессиональный", "корпоративный"],
            ),
        ],
    },
    Rulebook {
        subjects: &[
            "Стоимость доставки",
            "Тариф доставки",
            "Срок доставки заказа",
            "Выбор службы доставки",
        ],
        refusal: "отказ в доставке",
        conditions: &[
            gate("адрес входит в зону доставки"),
            gate("заказ оплачен"),
            gate("все товары есть на складе"),
            num("вес посылки больше # кг", &[2, 3, 5, 10, 15, 20, 25, 30]),
            flag("получатель выбрал экспресс-доставку"),
            flag("в заказе есть хрупкий товар"),
            flag("нужна доставка к точному времени"),
            flag("получатель — юридическое лицо"),
            num("длина посылки больше # см", &[50, 60, 80, 100, 120, 150]),
            num("сумма заказа больше # ₽", &[2000, 3000, 4000, 5000, 7000]),
            flag("доставка за город"),
        ],
        multi: &[
            (
                "способ доставки",
                &["курьер", "самовывоз", "почта", "пункт выдачи", "постамат"],
            ),
            ("размер посылки", &["S", "M", "L", "XL"]),
        ],
    },
    Rulebook {
        subjects: &[
            "Стоимость полиса автострахования",
            "Решение об оформлении страхового полиса",
            "Страховой тариф для водителя",
            "Коэффициент стоимости полиса",
        ],
        refusal: "отказ в оформлении полиса",
        conditions: &[
            gate("у водителя есть действующее удостоверение"),
            gate("автомобиль прошёл техосмотр"),
            Cond {
                text: "водительский стаж больше # лет",
                values: &[2, 3, 5],
                gate: true,
            },
            flag("за последний год были страховые случаи"),
            num("автомобиль старше # лет", &[3, 5, 7, 10, 15]),
            flag("автомобиль хранится в гараже"),
            flag("установлена противоугонная система"),
            num(
                "мощность двигателя больше # л. с.",
                &[100, 120, 150, 200, 250],
            ),
            num("водителю больше # лет", &[25, 30, 35, 40, 45, 50, 60]),
            flag("полис оформляется онлайн"),
        ],
        multi: &[
            (
                "категория транспорта",
                &[
                    "легковой автомобиль",
                    "грузовик",
                    "мотоцикл",
                    "автобус",
                    "прицеп",
                ],
            ),
            (
                "срок страхования",
                &["3 месяца", "полгода", "год", "два года"],
            ),
        ],
    },
    Rulebook {
        subjects: &[
            "Стоимость проживания",
            "Подтверждение бронирования",
            "Цена номера за ночь",
            "Размер предоплаты",
        ],
        refusal: "отказ в бронировании",
        conditions: &[
            gate("есть свободные номера на эти даты"),
            gate("гость внёс предоплату"),
            gate("данные карты гостя прошли проверку"),
            num(
                "бронирование сделано больше чем за # дней до заезда",
                &[7, 10, 14, 30, 60],
            ),
            flag("гость — участник программы лояльности"),
            num("проживание дольше # ночей", &[2, 3, 5, 7, 10, 14]),
            flag("нужен трансфер из аэропорта"),
            flag("заезд в выходные"),
            num("в номере больше # гостей", &[2, 3]),
            flag("гость бронирует напрямую через сайт отеля"),
        ],
        multi: &[
            (
                "тип номера",
                &["стандарт", "улучшенный", "люкс", "семейный", "апартаменты"],
            ),
            (
                "питание",
                &["без питания", "завтрак", "полупансион", "всё включено"],
            ),
        ],
    },
];

fn bullet_list(lines: &[String]) -> String {
    lines
        .iter()
        .map(|l| format!("- {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn decision_table(level: u8, rng: &mut Rng) -> Draft {
    let book = pick(rng, RULEBOOKS);
    let subject = *pick(rng, book.subjects);
    let all: Vec<&Cond> = book.conditions.iter().collect();
    match level {
        1 => {
            let n = range(rng, 2, 4) as usize;
            let conds: Vec<String> = pick_n(rng, &all, n).iter().map(|c| c.render(rng)).collect();
            let lead = if chance(rng, 0.5) {
                "зависит от булевых условий"
            } else {
                "определяется булевыми условиями"
            };
            let question = *pick(
                rng,
                &[
                    "Сколько правил (столбцов) будет в полной таблице решений?",
                    "Сколько столбцов-правил содержит полная (несокращённая) таблица решений для этих условий?",
                    "Сколько правил получится в полной таблице решений, если перебрать все комбинации условий?",
                ],
            );
            Draft::integer(
                format!("{subject} {lead}:\n\n{}\n\n{question}", bullet_list(&conds)),
                1 << n,
            )
        }
        2 => {
            let n = range(rng, 1, 3) as usize;
            let &(label, pool) = pick(rng, book.multi);
            let k = if chance(rng, 1.0 / 3.0) { 4 } else { 3 };
            let values = pick_n(rng, pool, k);
            let multi = format!(
                "{label}: {} или {}",
                values[..k - 1].join(", "),
                values[k - 1]
            );
            let mut lines: Vec<String> = pick_n(rng, &all, n)
                .iter()
                .map(|c| format!("{} (да/нет)", c.render(rng)))
                .collect();
            let at = range(rng, 0, n as i64) as usize;
            lines.insert(at, multi);
            let lead = if chance(rng, 0.5) {
                "зависит от условий"
            } else {
                "определяется условиями"
            };
            let question = if chance(rng, 0.5) {
                "Сколько правил будет в полной таблице решений?"
            } else {
                "Сколько столбцов (правил) содержит полная таблица решений?"
            };
            Draft::integer(
                format!("{subject} {lead}:\n\n{}\n\n{question}", bullet_list(&lines)),
                (k as i64) << n,
            )
        }
        _ => {
            let n = range(rng, 3, 5) as usize;
            let dominant = range(rng, 1, 2) as usize;
            let (gates, rest): (Vec<&Cond>, Vec<&Cond>) = all.iter().partition(|c| c.gate);
            let mut chosen: Vec<&Cond> = pick_n(rng, &gates, dominant);
            chosen.extend(pick_n(rng, &rest, n - dominant));
            let conds: Vec<String> = chosen.iter().map(|c| c.render(rng)).collect();
            let refusal = book.refusal;
            let rules: Vec<String> = conds[..dominant]
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    if i == 0 {
                        format!("если неверно, что {c}, — {refusal} независимо от остальных условий")
                    } else {
                        format!("если первое условие выполнено, но неверно, что {c}, — тоже {refusal} независимо от остальных")
                    }
                })
                .collect();
            let lead = if chance(rng, 0.5) {
                "задаётся таблицей решений"
            } else {
                "описывается таблицей решений"
            };
            let answer = dominant as i64 + (1i64 << (n - dominant));
            Draft::integer(
                format!(
                    "{subject} {lead} по {n} булевым условиям:\n\n{}\n\nПравила: {}. Остальные комбинации дают разные результаты. \
                     Сколько правил останется в таблице после схлопывания столбцов с безразличными («—») условиями?",
                    bullet_list(&conds),
                    rules.join("; ")
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

struct Flow {
    /// Nominative, lowercase unless it starts with an acronym.
    who: &'static str,
    /// Genitive plural of the unit of work.
    items: &'static str,
    /// Per-item time in nominative and prepositional case.
    time: (&'static str, &'static str),
}

const RESPONSE_TIME: (&str, &str) = ("время ответа", "времени ответа");

const FLOWS: &[Flow] = &[
    Flow {
        who: "сервис авторизации",
        items: "запросов",
        time: RESPONSE_TIME,
    },
    Flow {
        who: "API каталога",
        items: "запросов",
        time: RESPONSE_TIME,
    },
    Flow {
        who: "сервис поиска",
        items: "запросов",
        time: RESPONSE_TIME,
    },
    Flow {
        who: "API-шлюз",
        items: "запросов",
        time: RESPONSE_TIME,
    },
    Flow {
        who: "бэкенд мобильного приложения",
        items: "запросов",
        time: RESPONSE_TIME,
    },
    Flow {
        who: "сервис рекомендаций",
        items: "запросов",
        time: RESPONSE_TIME,
    },
    Flow {
        who: "микросервис корзины",
        items: "запросов",
        time: RESPONSE_TIME,
    },
    Flow {
        who: "сервис геокодинга",
        items: "запросов",
        time: RESPONSE_TIME,
    },
    Flow {
        who: "сервис профилей пользователей",
        items: "запросов",
        time: RESPONSE_TIME,
    },
    Flow {
        who: "платёжный шлюз",
        items: "транзакций",
        time: ("время обработки транзакции", "времени обработки транзакции"),
    },
    Flow {
        who: "консьюмер очереди заказов",
        items: "сообщений",
        time: ("время обработки сообщения", "времени обработки сообщения"),
    },
    Flow {
        who: "сервис уведомлений",
        items: "сообщений",
        time: ("время отправки сообщения", "времени отправки сообщения"),
    },
    Flow {
        who: "обработчик событий аналитики",
        items: "событий",
        time: ("время обработки события", "времени обработки события"),
    },
];

const LOAD_TOOLS: &[&str] = &["k6", "JMeter", "Locust", "Gatling"];

const LOAD_SCENARIOS: &[&str] = &[
    "Поиск товара",
    "Оформление заказа",
    "Вход в личный кабинет",
    "Просмотр каталога",
    "Добавление в корзину",
    "Оплата заказа",
    "Загрузка ленты",
    "Отправка сообщения",
];

fn littles_law(level: u8, rng: &mut Rng) -> Draft {
    let r: f64 = *pick(
        rng,
        &[
            0.1, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.75, 0.8, 1.2, 1.5, 2.0, 2.5,
        ],
    );
    let rt = fmt_number(r);
    match level {
        1 => {
            let x = range(rng, 2, 50) * 10;
            let f = pick(rng, FLOWS);
            let items = f.items;
            let lead = *pick(
                rng,
                &[
                    "",
                    "По данным мониторинга, ",
                    "В часы пик ",
                    "Во время нагрузочного теста ",
                    "В установившемся режиме ",
                ],
            );
            let verb = *pick(
                rng,
                &["стабильно обрабатывает", "обрабатывает", "обслуживает"],
            );
            let load = capitalize(&format!(
                "{lead}{} {verb} в среднем {x} {items} в секунду",
                f.who
            ));
            let timing = if chance(rng, 0.5) {
                format!(", среднее {} — {rt} с.", f.time.0)
            } else {
                format!(" при среднем {} {rt} с.", f.time.1)
            };
            let question = match range(rng, 0, 2) {
                0 => format!("Сколько {items} в среднем одновременно находятся в обработке?"),
                1 => format!("Сколько {items} в среднем обрабатываются одновременно?"),
                _ => format!("Какое среднее число {items} одновременно находится в системе?"),
            };
            Draft::number(format!("{load}{timing} {question}"), x as f64 * r, 0.01)
        }
        2 => {
            let x = range(rng, 2, 30) * 10;
            let z = range(rng, 1, 10) as f64;
            let zt = fmt_number(z);
            let tool = *pick(rng, LOAD_TOOLS);
            let scenario = *pick(rng, LOAD_SCENARIOS);
            let lead = match range(rng, 0, 2) {
                0 => format!("Нагрузочный тест в закрытой модели ({tool}, сценарий «{scenario}»)"),
                1 => {
                    format!("Сценарий «{scenario}» запускается в {tool} в закрытой модели нагрузки")
                }
                _ => format!("В {tool} настроен тест в закрытой модели для сценария «{scenario}»"),
            };
            let pause = if chance(rng, 0.5) {
                format!("затем делает паузу (think time) {zt} с")
            } else {
                format!("затем выдерживает think time {zt} с")
            };
            let question = match range(rng, 0, 2) {
                0 => format!(
                    "Сколько виртуальных пользователей нужно, чтобы получить {x} запросов в секунду?"
                ),
                1 => format!(
                    "Сколько виртуальных пользователей нужно запустить, чтобы тест создавал {x} запросов в секунду?"
                ),
                _ => format!(
                    "Какое число виртуальных пользователей обеспечит нагрузку {x} запросов в секунду?"
                ),
            };
            Draft::number(
                format!(
                    "{lead}: каждый виртуальный пользователь отправляет запрос, ждёт ответ (в среднем {rt} с), {pause} и повторяет. {question}"
                ),
                x as f64 * (r + z),
                0.01,
            )
        }
        _ => {
            let users = range(rng, 5, 60) * 10;
            let z = range(rng, 1, 10) as f64;
            let zt = fmt_number(z);
            let x = users as f64 / (r + z);
            let tool = *pick(rng, LOAD_TOOLS);
            let scenario = *pick(rng, LOAD_SCENARIOS);
            let setup = match range(rng, 0, 2) {
                0 => format!(
                    "Тест в {tool} для сценария «{scenario}» работает в закрытой модели: {users} виртуальных пользователей, среднее время ответа — {rt} с, think time — {zt} с."
                ),
                1 => format!(
                    "В закрытой модели нагрузки ({tool}, сценарий «{scenario}») {users} виртуальных пользователей; среднее время ответа — {rt} с, think time — {zt} с."
                ),
                _ => format!(
                    "Сценарий «{scenario}» в {tool}: {users} виртуальных пользователей в закрытой модели, среднее время ответа — {rt} с, пауза между итерациями (think time) — {zt} с."
                ),
            };
            let question = *pick(
                rng,
                &[
                    "Какую пропускную способность (запросов в секунду) создаст тест в установившемся режиме?",
                    "Сколько запросов в секунду создаст тест в установившемся режиме?",
                    "Какую нагрузку в запросах в секунду даст тест в установившемся режиме?",
                ],
            );
            Draft::number(
                format!("{setup} {question}"),
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
