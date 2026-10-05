//! Small dictionaries shared by the need and resume parsers.

use std::sync::LazyLock;

use catalog::{Grade, Industry, Specialization, WorkFormat};

use crate::text::{self, Matcher};

#[rustfmt::skip]
static CITIES: &[(&str, &[&str])] = &[
    ("Москва", &["москва", "мск", "moscow", "московск*"]),
    ("Санкт-Петербург", &["санкт-петербург", "петербург", "спб", "питер", "saint petersburg", "st. petersburg", "st petersburg", "spb"]),
    ("Новосибирск", &["новосибирск", "novosibirsk"]),
    ("Екатеринбург", &["екатеринбург", "екб", "yekaterinburg", "ekaterinburg"]),
    ("Казань", &["казань", "kazan"]),
    ("Нижний Новгород", &["нижний новгород", "nizhny novgorod"]),
    ("Челябинск", &["челябинск", "chelyabinsk"]),
    ("Самара", &["самара", "samara"]),
    ("Омск", &["омск", "omsk"]),
    ("Ростов-на-Дону", &["ростов-на-дону", "ростов", "rostov"]),
    ("Уфа", &["уфа", "уфе", "уфы", "ufa"]),
    ("Красноярск", &["красноярск", "krasnoyarsk"]),
    ("Воронеж", &["воронеж", "voronezh"]),
    ("Пермь", &["пермь", "perm"]),
    ("Волгоград", &["волгоград", "volgograd"]),
    ("Краснодар", &["краснодар", "krasnodar"]),
    ("Саратов", &["саратов", "saratov"]),
    ("Тюмень", &["тюмень", "tyumen"]),
    ("Томск", &["томск", "tomsk"]),
    ("Иркутск", &["иркутск", "irkutsk"]),
    ("Ярославль", &["ярославль", "yaroslavl"]),
    ("Владивосток", &["владивосток", "vladivostok"]),
    ("Калининград", &["калининград", "kaliningrad"]),
    ("Иннополис", &["иннополис", "innopolis"]),
    ("Сочи", &["сочи", "sochi"]),
    ("Тула", &["тула", "туле", "тулы", "tula"]),
    ("Ижевск", &["ижевск", "izhevsk"]),
    ("Хабаровск", &["хабаровск", "khabarovsk"]),
    ("Барнаул", &["барнаул", "barnaul"]),
];

static CITY_MATCHER: LazyLock<Matcher<u16>> = LazyLock::new(|| {
    Matcher::new(
        CITIES
            .iter()
            .enumerate()
            .flat_map(|(i, (_, aliases))| aliases.iter().map(move |a| (*a, i as u16))),
    )
});

/// First known city mentioned in the text, in canonical Russian spelling.
pub fn canonical_city(text: &str) -> Option<&'static str> {
    CITY_MATCHER
        .first(text)
        .and_then(|&i| CITIES.get(i as usize))
        .map(|(name, _)| *name)
}

/// Comparable city key: known cities collapse their aliases ("СПб" == "Питер"), unknown
/// ones hash their folded spelling; 0 means "not specified".
pub(crate) fn city_key(text: &str) -> u32 {
    if let Some(&i) = CITY_MATCHER.first(text) {
        return u32::from(i) + 1;
    }
    let k = text::key(
        text.trim()
            .trim_start_matches("г.")
            .trim_start_matches("г "),
    );
    if k.is_empty() {
        0
    } else {
        text::fnv1a(&k) | 0x8000_0000
    }
}

#[rustfmt::skip]
static INDUSTRY_PATTERNS: &[(&str, Option<Industry>)] = &[
    // Insurance and medical policies are perks in vacancy texts, not the domain.
    ("дмс", None), ("медицинск* страхов*", None), ("медицинск* полис*", None), ("страховк*", None),
    ("финтех*", Some(Industry::Fintech)), ("fintech", Some(Industry::Fintech)), ("банк", Some(Industry::Fintech)),
    ("банковск*", Some(Industry::Fintech)), ("bank*", Some(Industry::Fintech)), ("платеж*", Some(Industry::Fintech)),
    ("payments", Some(Industry::Fintech)), ("финансов*", Some(Industry::Fintech)), ("брокер*", Some(Industry::Fintech)),
    ("страхов* компани*", Some(Industry::Fintech)), ("кредитн*", Some(Industry::Fintech)), ("сбер*", Some(Industry::Fintech)),
    ("e-commerce", Some(Industry::Ecommerce)), ("ecommerce", Some(Industry::Ecommerce)), ("интернет-магазин*", Some(Industry::Ecommerce)),
    ("маркетплейс*", Some(Industry::Ecommerce)), ("marketplace*", Some(Industry::Ecommerce)), ("ритейл*", Some(Industry::Ecommerce)),
    ("retail", Some(Industry::Ecommerce)), ("wildberries", Some(Industry::Ecommerce)), ("ozon", Some(Industry::Ecommerce)),
    ("lamoda", Some(Industry::Ecommerce)),
    ("геймдев*", Some(Industry::Gamedev)), ("gamedev", Some(Industry::Gamedev)), ("game development", Some(Industry::Gamedev)),
    ("игров* студи*", Some(Industry::Gamedev)), ("игров* индустри*", Some(Industry::Gamedev)), ("game studio", Some(Industry::Gamedev)),
    ("unreal engine", Some(Industry::Gamedev)), ("unity3d", Some(Industry::Gamedev)), ("разработк* игр", Some(Industry::Gamedev)),
    ("госсектор*", Some(Industry::Govtech)), ("государственн* сектор*", Some(Industry::Govtech)), ("госуслуг*", Some(Industry::Govtech)),
    ("государственн* учреждени*", Some(Industry::Govtech)), ("государственн* информационн* систем*", Some(Industry::Govtech)),
    ("государственн* услуг*", Some(Industry::Govtech)), ("госкомпани*", Some(Industry::Govtech)),
    ("govtech", Some(Industry::Govtech)), ("госзакуп*", Some(Industry::Govtech)), ("министерств*", Some(Industry::Govtech)),
    ("госкорпорац*", Some(Industry::Govtech)), ("мфц", Some(Industry::Govtech)), ("public sector", Some(Industry::Govtech)),
    ("телеком*", Some(Industry::Telecom)), ("telecom*", Some(Industry::Telecom)), ("оператор* связи", Some(Industry::Telecom)),
    ("мтс", Some(Industry::Telecom)), ("билайн", Some(Industry::Telecom)), ("мегафон", Some(Industry::Telecom)),
    ("tele2", Some(Industry::Telecom)), ("ростелеком*", Some(Industry::Telecom)),
    ("edtech", Some(Industry::Edtech)), ("онлайн-образовани*", Some(Industry::Edtech)), ("онлайн-школ*", Some(Industry::Edtech)),
    ("образовательн* платформ*", Some(Industry::Edtech)), ("образовательн* сервис*", Some(Industry::Edtech)),
    ("образовательн* проект*", Some(Industry::Edtech)), ("lms", Some(Industry::Edtech)), ("skyeng", Some(Industry::Edtech)),
    ("skillbox", Some(Industry::Edtech)), ("нетологи*", Some(Industry::Edtech)),
    ("медицин*", Some(Industry::Healthtech)), ("healthtech", Some(Industry::Healthtech)), ("medtech", Some(Industry::Healthtech)),
    ("медтех*", Some(Industry::Healthtech)), ("здравоохранени*", Some(Industry::Healthtech)), ("клиник*", Some(Industry::Healthtech)),
    ("телемедицин*", Some(Industry::Healthtech)), ("фармацевт*", Some(Industry::Healthtech)), ("healthcare", Some(Industry::Healthtech)),
    ("медиа*", Some(Industry::Media)), ("media", Some(Industry::Media)), ("стриминг*", Some(Industry::Media)),
    ("streaming", Some(Industry::Media)), ("онлайн-кинотеатр*", Some(Industry::Media)), ("новостн*", Some(Industry::Media)),
    ("издательств*", Some(Industry::Media)), ("контент-платформ*", Some(Industry::Media)),
    ("промышленн* эксплуатаци*", None), ("промышлен*", Some(Industry::Manufacturing)), ("производств*", Some(Industry::Manufacturing)), ("завод*", Some(Industry::Manufacturing)),
    ("manufacturing", Some(Industry::Manufacturing)), ("industrial", Some(Industry::Manufacturing)), ("iiot", Some(Industry::Manufacturing)),
    ("нефтегаз*", Some(Industry::Manufacturing)), ("энергетик*", Some(Industry::Manufacturing)), ("металлург*", Some(Industry::Manufacturing)),
    ("асу тп", Some(Industry::Manufacturing)), ("scada", Some(Industry::Manufacturing)),
    ("логистик*", Some(Industry::Logistics)), ("logistics", Some(Industry::Logistics)), ("доставк*", Some(Industry::Logistics)),
    ("перевозк*", Some(Industry::Logistics)), ("грузоперевоз*", Some(Industry::Logistics)), ("склад", Some(Industry::Logistics)),
    ("складск*", Some(Industry::Logistics)), ("supply chain", Some(Industry::Logistics)), ("wms", Some(Industry::Logistics)),
    ("курьерск*", Some(Industry::Logistics)), ("транспортн* компани*", Some(Industry::Logistics)),
    ("аутсорс*", Some(Industry::Outsourcing)), ("outsourc*", Some(Industry::Outsourcing)), ("аутстаф*", Some(Industry::Outsourcing)),
    ("outstaff*", Some(Industry::Outsourcing)), ("заказн* разработк*", Some(Industry::Outsourcing)), ("веб-студи*", Some(Industry::Outsourcing)),
    ("digital-агентств*", Some(Industry::Outsourcing)), ("интегратор*", Some(Industry::Outsourcing)),
];

static INDUSTRY_MATCHER: LazyLock<Matcher<Option<Industry>>> =
    LazyLock::new(|| Matcher::new(INDUSTRY_PATTERNS.iter().map(|(p, i)| (*p, *i))));

pub(crate) fn industries(text: &str) -> Vec<Industry> {
    text::distinct(
        INDUSTRY_MATCHER
            .find_str(text)
            .into_iter()
            .filter_map(|v| *v),
    )
}

#[rustfmt::skip]
static SOFT_SKILLS: &[(&str, &[&str])] = &[
    ("Коммуникабельность", &["коммуникабельн*", "коммуникативн* навык*", "communication", "communication skills"]),
    ("Лидерство", &["лидерств*", "лидерск* качеств*", "leadership"]),
    ("Менторство", &["менторств*", "менторинг*", "наставничеств*", "mentoring", "mentorship", "наставник*"]),
    ("Работа в команде", &["работ* в команде", "командн* работ*", "командн* игрок*", "teamwork", "team player", "team work"]),
    ("Ответственность", &["ответственност*", "accountability"]),
    ("Самоорганизация", &["самоорганизац*", "самостоятельност*", "self-organized", "self-motivated", "self-management"]),
    ("Обучаемость", &["обучаемост*", "быстро обучаюсь", "быстро учусь", "легко обучаюсь", "fast learner", "quick learner"]),
    ("Стрессоустойчивость", &["стрессоустойчив*", "stress resistance", "stress tolerance"]),
    ("Аналитическое мышление", &["аналитическ* мышлени*", "аналитическ* склад*", "analytical thinking", "analytical skills", "analytical mindset"]),
    ("Критическое мышление", &["критическ* мышлени*", "critical thinking"]),
    ("Решение проблем", &["problem solving", "problem-solving", "решени* проблем"]),
    ("Тайм-менеджмент", &["тайм-менеджмент*", "time management", "управлени* временем"]),
    ("Публичные выступления", &["публичн* выступлени*", "public speaking", "выступал* на конференци*"]),
    ("Внимательность к деталям", &["внимательн* к детал*", "внимательност*", "attention to detail", "detail-oriented"]),
    ("Креативность", &["креативн*", "creativity", "creative thinking"]),
    ("Ориентация на результат", &["ориентаци* на результат", "результативност*", "result-oriented", "results-oriented"]),
    ("Проактивность", &["проактивн*", "инициативн*", "proactive", "proactivity"]),
    ("Эмпатия", &["эмпати*", "empathy"]),
    ("Адаптивность", &["адаптивност*", "гибкость мышления", "adaptability"]),
];

static SOFT_MATCHER: LazyLock<Matcher<u8>> = LazyLock::new(|| {
    Matcher::new(
        SOFT_SKILLS
            .iter()
            .enumerate()
            .flat_map(|(i, (_, aliases))| aliases.iter().map(move |a| (*a, i as u8))),
    )
});

pub(crate) fn soft_skills(text: &str) -> Vec<String> {
    text::distinct(SOFT_MATCHER.find_str(text).into_iter().copied())
        .into_iter()
        .filter_map(|i| {
            SOFT_SKILLS
                .get(i as usize)
                .map(|(name, _)| (*name).to_owned())
        })
        .collect()
}

#[rustfmt::skip]
static SPEC_PATTERNS: &[(&str, Option<(Specialization, f32)>)] = &[
    ("backend", Some((Specialization::Backend, 3.0))), ("back-end", Some((Specialization::Backend, 3.0))),
    ("бэкенд*", Some((Specialization::Backend, 3.0))), ("бекенд*", Some((Specialization::Backend, 3.0))),
    ("бэкэнд*", Some((Specialization::Backend, 3.0))), ("бэк~", Some((Specialization::Backend, 2.0))),
    ("серверн* разработк*", Some((Specialization::Backend, 3.0))), ("server-side", Some((Specialization::Backend, 2.0))),
    ("frontend", Some((Specialization::Frontend, 3.0))), ("front-end", Some((Specialization::Frontend, 3.0))),
    ("фронтенд*", Some((Specialization::Frontend, 3.0))), ("фронтэнд*", Some((Specialization::Frontend, 3.0))),
    ("фронт работ", None), ("фронт~", Some((Specialization::Frontend, 2.0))),
    ("верстальщик*", Some((Specialization::Frontend, 2.0))), ("ui-разработчик*", Some((Specialization::Frontend, 2.0))),
    ("devops", Some((Specialization::Devops, 3.0))), ("dev ops", Some((Specialization::Devops, 3.0))),
    ("девопс*", Some((Specialization::Devops, 3.0))), ("sre", Some((Specialization::Devops, 3.0))),
    ("site reliability", Some((Specialization::Devops, 3.0))), ("platform engineer*", Some((Specialization::Devops, 2.0))),
    ("инфраструктур*", Some((Specialization::Devops, 1.5))), ("системн* администратор*", Some((Specialization::Devops, 2.0))),
    ("сисадмин*", Some((Specialization::Devops, 2.0))),
    ("qa", Some((Specialization::Qa, 3.0))), ("тестировщик*", Some((Specialization::Qa, 3.0))),
    ("тестировщиц*", Some((Specialization::Qa, 3.0))), ("инженер* по тестированию", Some((Specialization::Qa, 3.0))),
    ("quality assurance", Some((Specialization::Qa, 3.0))), ("sdet", Some((Specialization::Qa, 3.0))),
    ("aqa", Some((Specialization::Qa, 3.0))), ("автоматизатор*", Some((Specialization::Qa, 3.0))),
    ("тестировани*", Some((Specialization::Qa, 1.0))), ("автотест*", Some((Specialization::Qa, 1.0))),
    ("tester", Some((Specialization::Qa, 3.0))),
    ("ml", Some((Specialization::Ml, 3.0))), ("machine learning", Some((Specialization::Ml, 3.0))),
    ("data scien*", Some((Specialization::Ml, 3.0))), ("дата сайентист*", Some((Specialization::Ml, 3.0))),
    ("датасайентист*", Some((Specialization::Ml, 3.0))), ("машинн* обучени*", Some((Specialization::Ml, 3.0))),
    ("cv-инженер*", Some((Specialization::Ml, 3.0))), ("nlp-инженер*", Some((Specialization::Ml, 3.0))),
    ("аналитик* данных", Some((Specialization::Ml, 1.5))), ("data analyst", Some((Specialization::Ml, 1.5))),
    ("data engineer", Some((Specialization::Ml, 1.5))),
    ("информационн* безопасност*", Some((Specialization::Security, 3.0))), ("кибербезопасност*", Some((Specialization::Security, 3.0))),
    ("information security", Some((Specialization::Security, 3.0))), ("cybersecurity", Some((Specialization::Security, 3.0))),
    ("infosec", Some((Specialization::Security, 3.0))), ("иб", Some((Specialization::Security, 2.0))),
    ("security", Some((Specialization::Security, 2.0))), ("безопасност*", Some((Specialization::Security, 1.0))),
    ("пентестер*", Some((Specialization::Security, 3.0))), ("soc-аналитик*", Some((Specialization::Security, 3.0))),
    ("spring security", None),
];

pub(crate) static SPEC_MATCHER: LazyLock<Matcher<Option<(Specialization, f32)>>> =
    LazyLock::new(|| Matcher::new(SPEC_PATTERNS.iter().map(|(p, v)| (*p, *v))));

#[rustfmt::skip]
static GRADE_PATTERNS: &[(&str, Grade)] = &[
    ("junior", Grade::Junior), ("джун~", Grade::Junior), ("джуниор*", Grade::Junior), ("младш*", Grade::Junior),
    ("начинающ*", Grade::Junior), ("стажер*", Grade::Junior), ("intern", Grade::Junior), ("trainee", Grade::Junior),
    ("entry level", Grade::Junior), ("без опыта", Grade::Junior),
    ("middle", Grade::Middle), ("мидл~", Grade::Middle), ("миддл~", Grade::Middle), ("mid-level", Grade::Middle),
    ("midlevel", Grade::Middle),
    ("senior", Grade::Senior), ("синьор~", Grade::Senior), ("сеньор~", Grade::Senior), ("сениор~", Grade::Senior),
    ("синиор~", Grade::Senior), ("старш*", Grade::Senior), ("ведущ* разработчик*", Grade::Senior),
    ("ведущ* инженер*", Grade::Senior), ("ведущ* специалист*", Grade::Senior), ("ведущ* программист*", Grade::Senior),
    ("ведущ* тестировщик*", Grade::Senior), ("ведущ* аналитик*", Grade::Senior), ("тимлид*", Grade::Senior),
    ("техлид*", Grade::Senior), ("team lead", Grade::Senior), ("teamlead", Grade::Senior), ("tech lead", Grade::Senior),
    ("techlead", Grade::Senior), ("lead", Grade::Senior), ("principal", Grade::Senior),
];

pub(crate) static GRADE_MATCHER: LazyLock<Matcher<Grade>> =
    LazyLock::new(|| Matcher::new(GRADE_PATTERNS.iter().map(|(p, g)| (*p, *g))));

pub(crate) fn grades(text: &str) -> Vec<Grade> {
    text::distinct(GRADE_MATCHER.find_str(text).into_iter().copied())
}

#[rustfmt::skip]
static FORMAT_PATTERNS: &[(&str, WorkFormat)] = &[
    ("удален*", WorkFormat::Remote), ("remote*", WorkFormat::Remote), ("дистанционн*", WorkFormat::Remote),
    ("work from home", WorkFormat::Remote), ("wfh", WorkFormat::Remote), ("из дома", WorkFormat::Remote),
    ("гибрид*", WorkFormat::Hybrid), ("hybrid", WorkFormat::Hybrid),
    ("офис*", WorkFormat::Office), ("office", WorkFormat::Office), ("on-site", WorkFormat::Office),
    ("onsite", WorkFormat::Office),
];

static FORMAT_MATCHER: LazyLock<Matcher<WorkFormat>> =
    LazyLock::new(|| Matcher::new(FORMAT_PATTERNS.iter().map(|(p, f)| (*p, *f))));

pub(crate) fn work_formats(text: &str) -> Vec<WorkFormat> {
    text::distinct(FORMAT_MATCHER.find_str(text).into_iter().copied())
}

#[rustfmt::skip]
static NICE_PATTERNS: &[&str] = &[
    "будет плюсом", "плюсом будет", "будет преимуществом", "преимуществом будет", "дополнительным плюсом",
    "желательно", "желателен", "желательна", "желательны", "приветствуется", "приветствуются",
    "как плюс", "не обязательно", "необязательно", "nice to have", "is a plus", "will be a plus",
    "would be a plus", "a plus", "preferred", "bonus points", "optional",
];

pub(crate) static NICE_MATCHER: LazyLock<Matcher<()>> =
    LazyLock::new(|| Matcher::new(NICE_PATTERNS.iter().map(|p| (*p, ()))));

#[rustfmt::skip]
static MUST_PATTERNS: &[&str] = &[
    "обязательно", "обязателен", "обязательна", "обязательны", "требуется", "требуются", "нужен", "нужна",
    "нужно", "необходим", "необходимо", "must", "required",
];

pub(crate) static MUST_MATCHER: LazyLock<Matcher<()>> =
    LazyLock::new(|| Matcher::new(MUST_PATTERNS.iter().map(|p| (*p, ()))));

#[rustfmt::skip]
static RELOCATION_PATTERNS: &[(&str, bool)] = &[
    ("не готов* к переезду", false), ("готов* к переезду", true), ("переезд не рассматрива*", false),
    ("open to relocation", true), ("ready to relocate", true), ("willing to relocate", true),
    ("not willing to relocate", false), ("relocation", true), ("релокаци*", true),
];

static RELOCATION_MATCHER: LazyLock<Matcher<bool>> =
    LazyLock::new(|| Matcher::new(RELOCATION_PATTERNS.iter().map(|(p, v)| (*p, *v))));

pub(crate) fn relocation(text: &str) -> Option<bool> {
    RELOCATION_MATCHER.first(text).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cities_with_inflection() {
        assert_eq!(canonical_city("офис в Москве"), Some("Москва"));
        assert_eq!(
            canonical_city("в Санкт-Петербурге"),
            Some("Санкт-Петербург")
        );
        assert_eq!(canonical_city("г. Казань"), Some("Казань"));
        assert_eq!(canonical_city("из Перми"), Some("Пермь"));
        assert_eq!(canonical_city("Kazan, Russia"), Some("Казань"));
        assert_eq!(canonical_city("Томск"), Some("Томск"));
        assert_eq!(canonical_city("по РФ"), None);
        assert_eq!(city_key("СПб"), city_key("Питер"));
        assert_eq!(city_key("г. Мухосранск"), city_key("мухосранск"));
        assert_ne!(city_key("Москва"), city_key("Казань"));
        assert_eq!(city_key("  "), 0);
    }

    #[test]
    fn industries_ignore_perks() {
        assert_eq!(
            industries("Финтех-компания, платёжная платформа"),
            vec![Industry::Fintech]
        );
        assert!(industries("ДМС, медицинская страховка").is_empty());
        assert_eq!(
            industries("маркетплейс и доставка"),
            vec![Industry::Ecommerce, Industry::Logistics]
        );
    }

    #[test]
    fn soft_skill_dictionary() {
        assert_eq!(
            soft_skills("Коммуникабельность, умение работать в команде; teamwork; менторство"),
            vec!["Коммуникабельность", "Работа в команде", "Менторство"]
        );
    }

    #[test]
    fn grade_words() {
        assert_eq!(grades("ищем мидла"), vec![Grade::Middle]);
        assert_eq!(grades("Джун или стажёр"), vec![Grade::Junior]);
        assert_eq!(grades("Senior/Lead"), vec![Grade::Senior]);
        assert!(grades("ведущий банк страны").is_empty());
    }

    #[test]
    fn formats() {
        assert_eq!(
            work_formats("Удалёнка или гибрид"),
            vec![WorkFormat::Remote, WorkFormat::Hybrid]
        );
        assert_eq!(work_formats("офис в центре"), vec![WorkFormat::Office]);
    }

    #[test]
    fn relocation_flags() {
        assert_eq!(relocation("Не готов к переезду"), Some(false));
        assert_eq!(relocation("готова к переезду"), Some(true));
        assert_eq!(relocation("ничего"), None);
    }
}
