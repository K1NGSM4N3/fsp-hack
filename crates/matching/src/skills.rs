//! Skill dictionary: canonical slugs, Russian/English aliases, extraction from free text
//! and a similarity graph used for partial credit in scoring.

use std::sync::LazyLock;

use catalog::Specialization;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::text::{self, Matcher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum SkillGroup {
    Language,
    Database,
    Messaging,
    Api,
    Backend,
    Frontend,
    Infrastructure,
    Testing,
    DataMl,
    Security,
    Fundamentals,
}

impl SkillGroup {
    pub const fn name_ru(self) -> &'static str {
        match self {
            SkillGroup::Language => "Языки программирования",
            SkillGroup::Database => "Базы данных",
            SkillGroup::Messaging => "Очереди и брокеры",
            SkillGroup::Api => "API и протоколы",
            SkillGroup::Backend => "Бэкенд-фреймворки",
            SkillGroup::Frontend => "Фронтенд",
            SkillGroup::Infrastructure => "Инфраструктура",
            SkillGroup::Testing => "Тестирование",
            SkillGroup::DataMl => "Данные и ML",
            SkillGroup::Security => "Безопасность",
            SkillGroup::Fundamentals => "Основы и практики",
        }
    }
}

/// Dictionary entry. Aliases are matched case-insensitively with token boundaries and
/// Russian inflection (see `text`); the slug is the stable id stored in the database.
#[derive(Debug, Serialize)]
pub struct SkillDef {
    pub slug: &'static str,
    pub name: &'static str,
    pub group: SkillGroup,
    pub aliases: &'static [&'static str],
}

/// Owned API representation of a [`SkillDef`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SkillView {
    pub slug: String,
    pub name: String,
    pub group: SkillGroup,
    pub group_name: String,
    pub aliases: Vec<String>,
}

impl From<&SkillDef> for SkillView {
    fn from(d: &SkillDef) -> Self {
        SkillView {
            slug: d.slug.to_owned(),
            name: d.name.to_owned(),
            group: d.group,
            group_name: d.group.name_ru().to_owned(),
            aliases: d.aliases.iter().map(|a| (*a).to_owned()).collect(),
        }
    }
}

const fn sk(
    slug: &'static str,
    name: &'static str,
    group: SkillGroup,
    aliases: &'static [&'static str],
) -> SkillDef {
    SkillDef {
        slug,
        name,
        group,
        aliases,
    }
}

use SkillGroup::*;

#[rustfmt::skip]
pub static SKILLS: &[SkillDef] = &[
    sk("python", "Python", Language, &["python", "python3", "питон", "пайтон", "питонист"]),
    sk("javascript", "JavaScript", Language, &["javascript", "js", "java script", "ecmascript", "es6", "джаваскрипт", "жс", "джс"]),
    sk("typescript", "TypeScript", Language, &["typescript", "ts", "тайпскрипт"]),
    sk("go", "Go", Language, &["go", "golang", "голанг"]),
    sk("java", "Java", Language, &["java", "jvm", "джава", "джавист"]),
    sk("kotlin", "Kotlin", Language, &["kotlin", "котлин"]),
    sk("rust", "Rust", Language, &["rust", "rustlang"]),
    sk("cpp", "C++", Language, &["c++", "с++", "cpp", "си++"]),
    sk("csharp", "C#", Language, &["c#", "с#", "csharp", "c sharp", ".net", "asp.net", "dotnet", "дотнет", "шарп"]),
    sk("php", "PHP", Language, &["php", "пхп", "laravel", "symfony", "yii"]),
    sk("sql", "SQL", Language, &["sql", "t-sql", "pl/sql", "plsql", "mssql", "ms sql", "oracle"]),
    sk("bash", "Bash", Language, &["bash", "shell", "shell scripting", "zsh", "баш"]),
    sk("postgresql", "PostgreSQL", Database, &["postgresql", "postgres", "postgre", "pg", "psql", "pgsql", "plpgsql", "pl/pgsql", "постгрес", "постгре", "постгрескуэль"]),
    sk("mysql", "MySQL", Database, &["mysql", "mariadb", "мускул"]),
    sk("mongodb", "MongoDB", Database, &["mongodb", "mongo", "монго", "монгодб"]),
    sk("redis", "Redis", Database, &["redis", "редис", "valkey", "keydb"]),
    sk("clickhouse", "ClickHouse", Database, &["clickhouse", "click house", "кликхаус", "кликхауз"]),
    sk("elasticsearch", "Elasticsearch", Database, &["elasticsearch", "elastic search", "elastic", "elk", "opensearch", "эластик", "эластиксерч"]),
    sk("kafka", "Apache Kafka", Messaging, &["kafka", "apache kafka", "кафка"]),
    sk("rabbitmq", "RabbitMQ", Messaging, &["rabbitmq", "rabbit mq", "rabbit", "amqp", "рэббит", "реббит"]),
    sk("grpc", "gRPC", Api, &["grpc", "protobuf", "protocol buffers"]),
    sk("rest", "REST API", Api, &["rest", "rest api", "restful", "рест", "рест апи"]),
    sk("graphql", "GraphQL", Api, &["graphql", "graph ql"]),
    sk("http", "HTTP", Api, &["http", "https", "http/2", "http2", "websocket", "websockets", "вебсокет"]),
    sk("django", "Django", Backend, &["django", "джанго", "drf", "django rest framework"]),
    sk("fastapi", "FastAPI", Backend, &["fastapi", "fast api", "фастапи"]),
    sk("flask", "Flask", Backend, &["flask", "фласк"]),
    sk("spring", "Spring", Backend, &["spring", "spring boot", "springboot", "spring framework", "spring cloud", "спринг", "спрингбут"]),
    sk("nodejs", "Node.js", Backend, &["node.js", "nodejs", "node", "express.js", "expressjs", "nestjs", "nest.js"]),
    sk("react", "React", Frontend, &["react", "react.js", "reactjs", "react native", "react hooks", "реакт"]),
    sk("vue", "Vue.js", Frontend, &["vue", "vue.js", "vuejs", "vue3", "nuxt", "nuxt.js", "vuex", "pinia", "вью"]),
    sk("angular", "Angular", Frontend, &["angular", "angularjs", "angular.js", "rxjs", "ангуляр"]),
    sk("nextjs", "Next.js", Frontend, &["next.js", "nextjs", "next js", "некст"]),
    sk("html", "HTML", Frontend, &["html", "html5", "верстка", "адаптивная верстка"]),
    sk("css", "CSS", Frontend, &["css", "css3", "scss", "sass", "tailwind", "tailwind css", "styled components", "flexbox"]),
    sk("redux", "Redux", Frontend, &["redux", "redux toolkit", "rtk query", "редакс"]),
    sk("docker", "Docker", Infrastructure, &["docker", "docker compose", "dockerfile", "podman", "докер", "контейнеризация"]),
    sk("kubernetes", "Kubernetes", Infrastructure, &["kubernetes", "k8s", "k3s", "kubectl", "openshift", "кубер", "кубернетес", "кубернетис"]),
    sk("linux", "Linux", Infrastructure, &["linux", "unix", "ubuntu", "debian", "centos", "rhel", "astra linux", "линукс", "юникс"]),
    sk("terraform", "Terraform", Infrastructure, &["terraform", "terragrunt", "opentofu", "терраформ"]),
    sk("ansible", "Ansible", Infrastructure, &["ansible", "ансибл"]),
    sk("ci_cd", "CI/CD", Infrastructure, &["ci/cd", "ci cd", "cicd", "gitlab ci", "github actions", "jenkins", "teamcity", "argocd", "argo cd", "continuous integration", "continuous delivery", "continuous deployment", "дженкинс"]),
    sk("prometheus", "Prometheus", Infrastructure, &["prometheus", "promql", "alertmanager", "victoriametrics", "прометеус"]),
    sk("grafana", "Grafana", Infrastructure, &["grafana", "графана"]),
    sk("nginx", "Nginx", Infrastructure, &["nginx", "энджинкс", "нджинкс"]),
    sk("networking", "Сети (TCP/IP)", Infrastructure, &["tcp/ip", "tcp", "udp", "dns", "dhcp", "vlan", "bgp", "ospf", "networking", "компьютерные сети", "сетевые технологии", "сетевое администрирование", "маршрутизация"]),
    sk("helm", "Helm", Infrastructure, &["helm", "helm charts", "helmfile", "хелм"]),
    sk("manual_testing", "Ручное тестирование", Testing, &["ручное тестирование", "manual testing", "manual qa", "функциональное тестирование", "регрессионное тестирование", "regression testing", "smoke testing", "смоук тестирование", "exploratory testing", "исследовательское тестирование", "баг-репорт", "bug reports"]),
    sk("test_design", "Тест-дизайн", Testing, &["тест-дизайн", "test design", "тест-кейс", "test case", "test cases", "чек-лист", "checklist", "checklists", "тестовая документация", "классы эквивалентности", "граничные значения", "equivalence partitioning", "boundary values", "pairwise"]),
    sk("selenium", "Selenium", Testing, &["selenium", "selenide", "webdriver", "селениум"]),
    sk("playwright", "Playwright", Testing, &["playwright", "плейрайт"]),
    sk("cypress", "Cypress", Testing, &["cypress"]),
    sk("pytest", "pytest", Testing, &["pytest", "py.test"]),
    sk("postman", "Postman", Testing, &["postman", "постман", "insomnia"]),
    sk("load_testing", "Нагрузочное тестирование", Testing, &["нагрузочное тестирование", "нагрузочные тесты", "load testing", "performance testing", "jmeter", "gatling", "locust", "k6", "яндекс.танк", "yandex tank"]),
    sk("api_testing", "Тестирование API", Testing, &["тестирование api", "тестирование rest api", "api testing", "api-тестирование", "rest assured", "soapui"]),
    sk("unit_testing", "Юнит-тестирование", Testing, &["unit tests", "unit test", "unit testing", "unit-тест", "юнит-тест", "юнит-тестирование", "модульное тестирование", "модульные тесты", "junit", "jest", "mocha", "unittest", "xunit", "nunit", "testng", "tdd"]),
    sk("pandas", "pandas", DataMl, &["pandas", "пандас"]),
    sk("numpy", "NumPy", DataMl, &["numpy", "нампай"]),
    sk("scikit_learn", "scikit-learn", DataMl, &["scikit-learn", "sklearn", "склерн"]),
    sk("pytorch", "PyTorch", DataMl, &["pytorch", "torch", "pytorch lightning", "пайторч"]),
    sk("tensorflow", "TensorFlow", DataMl, &["tensorflow", "keras", "тензорфлоу"]),
    sk("machine_learning", "Машинное обучение", DataMl, &["machine learning", "машинное обучение", "ml", "catboost", "xgboost", "lightgbm", "градиентный бустинг", "gradient boosting"]),
    sk("deep_learning", "Глубокое обучение", DataMl, &["deep learning", "глубокое обучение", "нейронные сети", "нейросети", "neural networks"]),
    sk("statistics", "Статистика", DataMl, &["statistics", "статистика", "статистический анализ", "теория вероятностей", "probability theory", "a/b тест", "a/b тестирование", "a/b testing", "ab тест", "проверка гипотез", "hypothesis testing"]),
    sk("ml_metrics", "Метрики качества ML", DataMl, &["ml metrics", "ml-метрики", "метрики качества моделей", "roc-auc", "auc", "f1-score", "f1-мера", "precision", "recall", "кросс-валидация", "cross-validation", "валидация моделей"]),
    sk("nlp", "NLP", DataMl, &["nlp", "natural language processing", "обработка естественного языка", "обработка текстов", "transformers", "bert", "hugging face", "huggingface", "spacy", "nltk", "word2vec"]),
    sk("llm", "LLM", DataMl, &["llm", "llms", "large language models", "rag", "langchain", "prompt engineering", "промпт-инжиниринг", "gpt"]),
    sk("computer_vision", "Компьютерное зрение", DataMl, &["computer vision", "компьютерное зрение", "opencv", "yolo", "object detection", "детекция объектов", "сегментация изображений", "image segmentation", "распознавание изображений"]),
    sk("spark", "Apache Spark", DataMl, &["spark", "apache spark", "pyspark", "spark sql", "спарк"]),
    sk("airflow", "Apache Airflow", DataMl, &["airflow", "apache airflow"]),
    sk("owasp", "OWASP Top 10", Security, &["owasp", "owasp top 10"]),
    sk("pentest", "Пентест", Security, &["pentest", "pentesting", "pentester", "penetration testing", "пентест", "пентестер", "тестирование на проникновение", "red team", "burp suite", "burp", "metasploit", "nmap", "kali linux"]),
    sk("cryptography", "Криптография", Security, &["cryptography", "криптография", "шифрование", "encryption", "pki", "tls", "ssl", "openssl"]),
    sk("network_security", "Сетевая безопасность", Security, &["network security", "сетевая безопасность", "firewall", "межсетевой экран", "межсетевые экраны", "ids/ips", "ngfw", "suricata", "snort", "wireshark"]),
    sk("appsec", "Безопасность приложений (AppSec)", Security, &["appsec", "application security", "безопасность приложений", "безопасная разработка", "devsecops", "sast", "dast", "ssdlc", "secure sdlc"]),
    sk("siem", "SIEM / SOC", Security, &["siem", "soc", "splunk", "maxpatrol", "incident response", "реагирование на инциденты", "мониторинг инцидентов"]),
    sk("git", "Git", Fundamentals, &["git", "github", "gitlab", "bitbucket", "gitflow", "git flow", "гит"]),
    sk("algorithms", "Алгоритмы", Fundamentals, &["algorithms", "алгоритмы", "алгоритмические задачи", "спортивное программирование", "олимпиадное программирование", "competitive programming", "leetcode"]),
    sk("data_structures", "Структуры данных", Fundamentals, &["data structures", "структуры данных"]),
    sk("system_design", "Проектирование систем", Fundamentals, &["system design", "проектирование систем", "архитектура систем", "software architecture", "highload", "high load", "хайлоад", "высоконагруженные", "ddd", "domain-driven design"]),
    sk("microservices", "Микросервисы", Fundamentals, &["microservices", "microservice", "микросервисы", "микросервисная архитектура"]),
    sk("concurrency", "Многопоточность и асинхронность", Fundamentals, &["concurrency", "multithreading", "многопоточность", "асинхронное программирование", "параллельное программирование", "asyncio", "async/await", "goroutines", "горутины"]),
];

/// Aliases that are also common English words: matched only with tech context.
#[rustfmt::skip]
const AMBIGUOUS: &[&str] = &[
    "go", "rest", "react", "spring", "ts", "spark", "node", "ml", "soc", "shell", "precision",
    "recall", "rag", "elastic", "rabbit", "torch", "oracle", "unix",
];

/// Symmetric similarity edges. Values express how much of the required skill a holder of
/// the other one can be expected to cover on day one (framework ↔ its language ≈ 0.5,
/// interchangeable tools ≈ 0.7). Anything not listed is 0 — explicit beats clever here,
/// because every partial credit is shown to the employer.
#[rustfmt::skip]
const EDGES: &[(&str, &str, f32)] = &[
    ("javascript", "typescript", 0.8), ("java", "kotlin", 0.7), ("java", "csharp", 0.5),
    ("cpp", "rust", 0.4), ("cpp", "csharp", 0.3),
    ("python", "django", 0.5), ("python", "fastapi", 0.5), ("python", "flask", 0.5),
    ("django", "fastapi", 0.5), ("django", "flask", 0.6), ("fastapi", "flask", 0.7),
    ("java", "spring", 0.6), ("kotlin", "spring", 0.5),
    ("javascript", "nodejs", 0.6), ("typescript", "nodejs", 0.5),
    ("python", "pandas", 0.4), ("python", "pytest", 0.3),
    ("postgresql", "mysql", 0.5), ("postgresql", "sql", 0.6), ("mysql", "sql", 0.6),
    ("clickhouse", "sql", 0.4), ("clickhouse", "postgresql", 0.3), ("mongodb", "postgresql", 0.2),
    ("kafka", "rabbitmq", 0.5),
    ("rest", "http", 0.6), ("rest", "grpc", 0.4), ("rest", "graphql", 0.4), ("http", "networking", 0.4),
    ("microservices", "system_design", 0.5),
    ("react", "vue", 0.4), ("react", "angular", 0.3), ("vue", "angular", 0.4),
    ("react", "nextjs", 0.7), ("react", "redux", 0.6), ("javascript", "react", 0.3), ("html", "css", 0.5),
    ("docker", "kubernetes", 0.5), ("kubernetes", "helm", 0.6), ("terraform", "ansible", 0.5),
    ("linux", "bash", 0.6), ("linux", "nginx", 0.3), ("linux", "networking", 0.3),
    ("prometheus", "grafana", 0.7), ("ci_cd", "docker", 0.3), ("ci_cd", "git", 0.3),
    ("pytest", "unit_testing", 0.6), ("selenium", "playwright", 0.7), ("selenium", "cypress", 0.6),
    ("playwright", "cypress", 0.7), ("manual_testing", "test_design", 0.6), ("postman", "api_testing", 0.7),
    ("api_testing", "rest", 0.4), ("load_testing", "api_testing", 0.3), ("unit_testing", "test_design", 0.3),
    ("pandas", "numpy", 0.6), ("pandas", "spark", 0.4),
    ("scikit_learn", "machine_learning", 0.7), ("scikit_learn", "ml_metrics", 0.5),
    ("machine_learning", "ml_metrics", 0.6), ("machine_learning", "statistics", 0.4), ("statistics", "ml_metrics", 0.5),
    ("pytorch", "tensorflow", 0.7), ("pytorch", "deep_learning", 0.7), ("tensorflow", "deep_learning", 0.7),
    ("deep_learning", "machine_learning", 0.6), ("nlp", "llm", 0.6), ("nlp", "deep_learning", 0.4),
    ("computer_vision", "deep_learning", 0.4), ("spark", "airflow", 0.3),
    ("owasp", "appsec", 0.8), ("pentest", "appsec", 0.6), ("pentest", "owasp", 0.6),
    ("pentest", "network_security", 0.4), ("network_security", "networking", 0.5),
    ("network_security", "siem", 0.5), ("cryptography", "appsec", 0.3),
    ("algorithms", "data_structures", 0.8),
];

#[rustfmt::skip]
const TYPICAL_BACKEND: &[&str] = &["python", "go", "java", "postgresql", "sql", "redis", "kafka", "docker", "rest", "microservices", "git", "linux"];
#[rustfmt::skip]
const TYPICAL_FRONTEND: &[&str] = &["javascript", "typescript", "react", "vue", "html", "css", "redux", "nextjs", "rest", "git"];
#[rustfmt::skip]
const TYPICAL_DEVOPS: &[&str] = &["linux", "docker", "kubernetes", "terraform", "ansible", "ci_cd", "prometheus", "grafana", "helm", "nginx", "bash", "networking"];
#[rustfmt::skip]
const TYPICAL_QA: &[&str] = &["manual_testing", "test_design", "api_testing", "postman", "sql", "selenium", "playwright", "pytest", "load_testing", "git"];
#[rustfmt::skip]
const TYPICAL_ML: &[&str] = &["python", "pandas", "numpy", "scikit_learn", "pytorch", "machine_learning", "statistics", "ml_metrics", "sql", "nlp", "computer_vision"];
#[rustfmt::skip]
const TYPICAL_SECURITY: &[&str] = &["owasp", "pentest", "appsec", "network_security", "cryptography", "siem", "linux", "networking", "python"];

/// Default stack of a specialization, for autocomplete and need drafts.
pub fn typical_skills(spec: Specialization) -> &'static [&'static str] {
    match spec {
        Specialization::Backend => TYPICAL_BACKEND,
        Specialization::Frontend => TYPICAL_FRONTEND,
        Specialization::Devops => TYPICAL_DEVOPS,
        Specialization::Qa => TYPICAL_QA,
        Specialization::Ml => TYPICAL_ML,
        Specialization::Security => TYPICAL_SECURITY,
    }
}

/// Compact skill id for hot scoring paths. Dictionary skills use their index; skills
/// outside the dictionary get a stable 15-bit hash with the top bit set, so they still
/// match themselves without any shared interner state. Ids are process-local: persist slugs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SkillId(u16);

const UNKNOWN_BIT: u16 = 0x8000;

impl SkillId {
    pub fn of_slug(slug: &str) -> Option<SkillId> {
        INDEX.by_slug.get(slug).map(|&i| SkillId(i))
    }

    /// Slug, alias or free-form skill name → id.
    pub fn intern(text: &str) -> SkillId {
        if let Some(id) = Self::of_slug(text) {
            return id;
        }
        let k = text::key(text);
        match INDEX.by_key.get(&k) {
            Some(&i) => SkillId(i),
            None => SkillId(UNKNOWN_BIT | (text::fnv1a(&k) as u16 & !UNKNOWN_BIT)),
        }
    }

    pub fn def(self) -> Option<&'static SkillDef> {
        if self.is_known() {
            SKILLS.get(self.0 as usize)
        } else {
            None
        }
    }

    pub const fn is_known(self) -> bool {
        self.0 & UNKNOWN_BIT == 0
    }

    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }

    pub fn similarity(self, other: SkillId) -> f32 {
        if self == other {
            return 1.0;
        }
        if self.is_known() && other.is_known() {
            SIM.get(self.index() * SKILLS.len() + other.index())
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Row of the dense similarity table, for scanning many candidate skills against one
    /// required skill without per-pair lookups.
    pub(crate) fn sim_row(self) -> Option<&'static [f32]> {
        if !self.is_known() {
            return None;
        }
        let n = SKILLS.len();
        SIM.get(self.index() * n..(self.index() + 1) * n)
    }
}

/// Closed, compile-time key sets (slugs, folded aliases) → dense skill index.
struct Index {
    by_slug: FxHashMap<&'static str, u16>,
    by_key: FxHashMap<String, u16>,
}

static INDEX: LazyLock<Index> = LazyLock::new(|| {
    let mut by_slug = FxHashMap::with_capacity_and_hasher(SKILLS.len(), Default::default());
    let mut by_key = FxHashMap::default();
    for (i, d) in SKILLS.iter().enumerate() {
        let i = i as u16;
        by_slug.insert(d.slug, i);
        for k in [d.slug, d.name]
            .into_iter()
            .chain(d.aliases.iter().copied())
        {
            by_key.entry(text::key(k)).or_insert(i);
        }
    }
    Index { by_slug, by_key }
});

static MATCHER: LazyLock<Matcher<u16>> = LazyLock::new(|| {
    let mut specs: Vec<(String, u16)> = Vec::new();
    for (i, d) in SKILLS.iter().enumerate() {
        for a in d.aliases {
            let spec = if AMBIGUOUS.contains(a) {
                format!("?{a}")
            } else {
                (*a).to_owned()
            };
            specs.push((spec, i as u16));
        }
    }
    Matcher::new(specs.iter().map(|(s, i)| (s.as_str(), *i)))
});

static SIM: LazyLock<Box<[f32]>> = LazyLock::new(|| {
    let n = SKILLS.len();
    let mut table = vec![0.0f32; n * n];
    for i in 0..n {
        table[i * n + i] = 1.0;
    }
    for &(a, b, w) in EDGES {
        if let (Some(a), Some(b)) = (SkillId::of_slug(a), SkillId::of_slug(b)) {
            table[a.index() * n + b.index()] = w;
            table[b.index() * n + a.index()] = w;
        }
    }
    table.into_boxed_slice()
});

pub fn lookup(slug: &str) -> Option<&'static SkillDef> {
    SkillId::of_slug(slug).and_then(SkillId::def)
}

/// Single skill name as typed by a user ("Postgres", "k8s", "питоне") → dictionary entry.
pub fn normalize(text: &str) -> Option<&'static SkillDef> {
    let k = text::key(text);
    if k.is_empty() {
        return None;
    }
    if let Some(&i) = INDEX.by_key.get(&k) {
        return SKILLS.get(i as usize);
    }
    if k.split(' ').count() > 4 {
        return None;
    }
    extract_ids(text).first().and_then(|id| id.def())
}

pub(crate) fn extract_ids(text: &str) -> Vec<SkillId> {
    text::distinct(MATCHER.find_str(text).into_iter().map(|&i| SkillId(i)))
}

/// Canonical slugs mentioned in free text, in order of first mention.
pub fn extract_skills(text: &str) -> Vec<&'static str> {
    extract_ids(text)
        .into_iter()
        .filter_map(|id| id.def().map(|d| d.slug))
        .collect()
}

/// Similarity of two skills given as slugs or aliases: 1.0 for the same skill, the
/// graph weight for related ones, otherwise 0.
pub fn similarity(a: &str, b: &str) -> f32 {
    SkillId::intern(a).similarity(SkillId::intern(b))
}

/// Autocomplete: exact matches first, then name prefixes, alias prefixes, substrings.
pub fn search(prefix: &str) -> Vec<&'static SkillDef> {
    let q = text::key(prefix);
    if q.is_empty() {
        return Vec::new();
    }
    let mut ranked: Vec<(u8, &'static SkillDef)> = SKILLS
        .iter()
        .filter_map(|d| {
            let name = text::key(d.name);
            let names = || {
                std::iter::once(d.slug)
                    .chain(d.aliases.iter().copied())
                    .map(text::key)
            };
            let rank = if name == q || names().any(|a| a == q) {
                0
            } else if name.starts_with(&q) || name.split(' ').any(|w| w.starts_with(&q)) {
                1
            } else if names().any(|a| a.starts_with(&q)) {
                2
            } else if q.chars().count() >= 2
                && (name.contains(&q) || names().any(|a| a.contains(&q)))
            {
                3
            } else {
                return None;
            };
            Some((rank, d))
        })
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.name.cmp(b.1.name)));
    ranked.into_iter().map(|(_, d)| d).collect()
}

/// How strongly a skill points at each specialization (sums to 1 for skills that point
/// anywhere). Typical stacks split the vote; other skills vote by their group.
pub(crate) fn specialization_affinity(id: SkillId) -> [f32; 6] {
    AFFINITY.get(id.index()).copied().unwrap_or([0.0; 6])
}

static AFFINITY: LazyLock<Vec<[f32; 6]>> = LazyLock::new(|| {
    SKILLS
        .iter()
        .map(|d| {
            let mut v = [0.0f32; 6];
            let owners: Vec<Specialization> = Specialization::ALL
                .iter()
                .copied()
                .filter(|s| typical_skills(*s).contains(&d.slug))
                .collect();
            if owners.is_empty() {
                let spec = match d.group {
                    Language | Database | Messaging | Api | Backend => {
                        Some(Specialization::Backend)
                    }
                    Frontend => Some(Specialization::Frontend),
                    Infrastructure => Some(Specialization::Devops),
                    Testing => Some(Specialization::Qa),
                    DataMl => Some(Specialization::Ml),
                    Security => Some(Specialization::Security),
                    Fundamentals => None,
                };
                if let Some(s) = spec {
                    v[s as usize] = 1.0;
                }
            } else {
                let share = 1.0 / owners.len() as f32;
                for s in owners {
                    v[s as usize] = share;
                }
            }
            v
        })
        .collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_is_consistent() {
        assert!(SKILLS.len() >= 75);
        assert!(SKILLS.len() < UNKNOWN_BIT as usize);
        let mut seen: FxHashMap<String, &str> = FxHashMap::default();
        for d in SKILLS {
            assert!(!d.aliases.is_empty(), "{}", d.slug);
            for a in d.aliases {
                if let Some(prev) = seen.insert(text::key(a), d.slug) {
                    panic!("alias `{a}` used by `{prev}` and `{}`", d.slug);
                }
            }
        }
        for &(a, b, w) in EDGES {
            assert!(lookup(a).is_some() && lookup(b).is_some(), "{a}~{b}");
            assert!(w > 0.0 && w < 1.0);
        }
        for s in Specialization::ALL {
            for slug in typical_skills(*s) {
                assert!(lookup(slug).is_some(), "{slug}");
            }
        }
    }

    #[test]
    fn required_vocabulary_present() {
        let required = "python javascript typescript go java kotlin rust cpp csharp php sql bash \
            postgresql mysql mongodb redis kafka rabbitmq clickhouse elasticsearch grpc rest graphql \
            http django fastapi spring nodejs react vue angular nextjs html css redux docker \
            kubernetes linux terraform ansible ci_cd prometheus nginx networking helm manual_testing \
            test_design selenium playwright pytest postman load_testing api_testing pandas numpy \
            scikit_learn pytorch statistics ml_metrics nlp computer_vision spark owasp pentest \
            cryptography network_security appsec git algorithms data_structures system_design \
            concurrency unit_testing";
        for slug in required.split_whitespace() {
            assert!(lookup(slug).is_some(), "missing {slug}");
        }
    }

    #[test]
    fn normalize_aliases() {
        let n = |s: &str| normalize(s).map(|d| d.slug);
        assert_eq!(n("Postgres"), Some("postgresql"));
        assert_eq!(n("постгрес"), Some("postgresql"));
        assert_eq!(n("pg"), Some("postgresql"));
        assert_eq!(n("k8s"), Some("kubernetes"));
        assert_eq!(n("Кубер"), Some("kubernetes"));
        assert_eq!(n("докер"), Some("docker"));
        assert_eq!(n("Питон"), Some("python"));
        assert_eq!(n("джава"), Some("java"));
        assert_eq!(n("жс"), Some("javascript"));
        assert_eq!(n("Spring Boot"), Some("spring"));
        assert_eq!(n("spring-boot"), Some("spring"));
        assert_eq!(n("C++"), Some("cpp"));
        assert_eq!(n("C#"), Some("csharp"));
        assert_eq!(n(".NET"), Some("csharp"));
        assert_eq!(n("GitLab CI"), Some("ci_cd"));
        assert_eq!(n("github actions"), Some("ci_cd"));
        assert_eq!(n("Machine Learning"), Some("machine_learning"));
        assert_eq!(n("go"), Some("go"));
        assert_eq!(n("scikit_learn"), Some("scikit_learn"));
        assert_eq!(n("на питоне"), Some("python"));
        assert_eq!(n("ёлки-палки"), None);
        assert_eq!(n(""), None);
    }

    #[test]
    fn extraction_respects_boundaries() {
        assert_eq!(extract_skills("Google, good, golang"), vec!["go"]);
        assert!(extract_skills("we will go to market").is_empty());
        assert_eq!(extract_skills("Senior Backend Developer (Go)"), vec!["go"]);
        assert_eq!(
            extract_skills("C/C++, C#, ASP.NET, Node.js"),
            vec!["cpp", "csharp", "nodejs"]
        );
        assert_eq!(
            extract_skills("JavaScript и Java"),
            vec!["javascript", "java"]
        );
        assert_eq!(
            extract_skills("Опыт с кубером и докером, деплой через GitLab CI, k8s"),
            vec!["kubernetes", "docker", "ci_cd"]
        );
        assert_eq!(
            extract_skills("знание постгреса, Kafka/RabbitMQ, REST API"),
            vec!["postgresql", "kafka", "rabbitmq", "rest"]
        );
        assert_eq!(
            extract_skills("опыт нагрузочного тестирования"),
            vec!["load_testing"]
        );
        assert!(extract_skills("rest of the day").is_empty());
        assert!(extract_skills("ivan.go@mail.ru").is_empty());
    }

    #[test]
    fn similarity_is_symmetric() {
        assert_eq!(similarity("postgresql", "mysql"), 0.5);
        assert_eq!(similarity("mysql", "postgresql"), 0.5);
        assert_eq!(similarity("react", "vue"), 0.4);
        assert_eq!(similarity("pytest", "unit_testing"), 0.6);
        assert_eq!(similarity("kubernetes", "helm"), 0.6);
        assert_eq!(similarity("javascript", "typescript"), 0.8);
        assert_eq!(similarity("k8s", "kubernetes"), 1.0);
        assert_eq!(similarity("go", "react"), 0.0);
        assert_eq!(similarity("Cobol", "cobol"), 1.0);
        assert_eq!(similarity("Cobol", "Fortran"), 0.0);
    }

    #[test]
    fn unknown_skills_intern_stably() {
        let a = SkillId::intern("Haskell");
        assert!(!a.is_known());
        assert_eq!(a, SkillId::intern(" haskell "));
        assert!(a.def().is_none());
        assert!(SkillId::intern("k8s").is_known());
    }

    #[test]
    fn search_ranks_prefixes() {
        let names: Vec<&str> = search("pos").iter().map(|d| d.slug).collect();
        assert_eq!(names.first(), Some(&"postgresql"));
        assert!(names.contains(&"postman"));
        assert_eq!(search("k8s")[0].slug, "kubernetes");
        assert!(search("").is_empty());
    }

    #[test]
    fn affinity_votes() {
        let react = specialization_affinity(SkillId::intern("react"));
        assert_eq!(react[Specialization::Frontend as usize], 1.0);
        let py = specialization_affinity(SkillId::intern("python"));
        assert!(
            py[Specialization::Backend as usize] > 0.0 && py[Specialization::Ml as usize] > 0.0
        );
    }
}
