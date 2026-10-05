import type {
  ConsentDocument,
  DeclineReason,
  ExperienceEntry,
  FspDiscipline,
  FspEventLevel,
  FspPlace,
  FspRank,
  Grade,
  Industry,
  InvitationStatus,
  SearchStatus,
  SkillSource,
  Specialization,
  UserConsent,
  WorkFormat,
} from "@/api/types";
import { GRADE_THRESHOLD } from "./catalog";
import { FSP_REGISTRY, type RawAchievement } from "./fsp";
import { chance, mulberry32, pick, randInt, sample, uuidFrom, type Rng } from "./rng";
import type {
  MockApplication,
  MockAssessment,
  MockCandidate,
  MockCompany,
  MockDb,
  MockInvitation,
  MockNeed,
  MockTask,
  MockTaskAssignment,
  MockUser,
  MockVacancy,
} from "./store";

export const SEED_VERSION = 6;
export const DEMO_PASSWORD = "demo12345";
export const DEMO_CANDIDATE_EMAIL = "candidate@demo.fsp";
export const DEMO_EMPLOYER_EMAIL = "employer@demo.fsp";

export const CONSENT_DOCS: ConsentDocument[] = [
  {
    id: "0192a000-0000-7000-8000-000000000001",
    kind: "pd_processing",
    version: 2,
    title: "Согласие на обработку персональных данных",
    body: [
      "Я даю согласие оператору платформы на обработку моих персональных данных в соответствии с Федеральным законом от 27.07.2006 № 152-ФЗ «О персональных данных».",
      "Состав данных: фамилия, имя, адрес электронной почты, номер телефона, имя пользователя в Telegram, город, сведения об опыте работы и навыках, результаты тестирования, сведения о достижениях в спортивном программировании.",
      "Цели обработки: подбор предложений о работе, присвоение категории по результатам тестирования, связь с работодателями после моего согласия.",
      "Данные хранятся на серверах в Российской Федерации. Контактные данные хранятся в зашифрованном виде.",
      "Согласие действует до его отзыва. Отозвать согласие можно в настройках аккаунта.",
    ].join("\n\n"),
    required_for: ["candidate", "employer"],
    applies_to: ["candidate", "employer"],
  },
  {
    id: "0192a000-0000-7000-8000-000000000002",
    kind: "pd_publication",
    version: 1,
    title: "Согласие на публикацию анонимного профиля",
    body: [
      "Я согласен, чтобы мой профиль был доступен зарегистрированным работодателям в анонимном виде: категория (специализация и грейд), результат теста, навыки, достижения ФСП, обобщённый опыт и ожидания по зарплате.",
      "Имя и контакты не публикуются. Работодатель получает контакты только после того, как я приму его приглашение или сам откликнусь на вакансию. Каждый доступ к контактам записывается в журнал.",
      "Что именно видно работодателям, можно изменить в настройках приватности. Отзыв этого согласия скрывает профиль из поиска.",
    ].join("\n\n"),
    required_for: ["candidate"],
    applies_to: ["candidate"],
  },
];

const DAY = 86400000;

const MALE = ["Алексей", "Дмитрий", "Иван", "Михаил", "Артём", "Никита", "Егор", "Кирилл", "Максим", "Роман", "Сергей", "Павел", "Глеб", "Тимур", "Арсений", "Владислав", "Андрей", "Илья"];
const FEMALE = ["Мария", "Екатерина", "Дарья", "Полина", "Алина", "Ксения", "Виктория", "Софья", "Елена", "Ольга", "Юлия", "Вера", "Анастасия"];
const LAST = ["Иванов", "Смирнов", "Кузнецов", "Попов", "Волков", "Лебедев", "Новиков", "Морозов", "Петров", "Орлов", "Захаров", "Белов", "Медведев", "Ершов", "Никитин", "Комаров", "Гусев", "Титов", "Макаров", "Фролов", "Широков", "Тарасов", "Гарипов", "Хабибуллин", "Алиев"];
const CITIES = ["Москва", "Москва", "Москва", "Санкт-Петербург", "Санкт-Петербург", "Казань", "Новосибирск", "Екатеринбург", "Нижний Новгород", "Томск", "Иннополис", "Краснодар"];
const INDUSTRIES: Industry[] = ["fintech", "ecommerce", "gamedev", "govtech", "telecom", "healthtech", "edtech", "logistics", "media", "manufacturing", "outsourcing"];
const COMPANY_TYPES = [
  "Финтех-компания, 1000+ сотрудников",
  "Маркетплейс, 5000+ сотрудников",
  "Интегратор для госсектора",
  "Игровая студия, 150 сотрудников",
  "Телеком-оператор",
  "Стартап в медтехе, 40 сотрудников",
  "Логистическая платформа",
  "Банк из топ-20",
  "Продуктовая EdTech-компания",
];

const STACKS: Record<Specialization, string[][]> = {
  backend: [
    ["go", "postgresql", "kafka", "docker", "kubernetes", "grpc", "redis", "clickhouse", "microservices", "linux"],
    ["go", "postgresql", "docker", "redis", "rest", "kafka", "git"],
    ["java", "kotlin", "spring", "postgresql", "kafka", "docker", "microservices", "redis", "sql"],
    ["python", "fastapi", "django", "postgresql", "redis", "rabbitmq", "docker", "sql"],
    ["csharp", "postgresql", "rabbitmq", "docker", "rest", "sql", "redis"],
  ],
  frontend: [
    ["typescript", "javascript", "react", "nextjs", "redux", "css", "html", "jest", "playwright", "a11y", "graphql"],
    ["typescript", "javascript", "vue", "css", "html", "webpack", "jest", "rest"],
  ],
  devops: [["kubernetes", "docker", "terraform", "ansible", "prometheus", "grafana", "helm", "gitlab_ci", "nginx", "linux", "bash", "yandex_cloud"]],
  qa: [
    ["python", "pytest", "selenium", "playwright", "api_testing", "postman", "sql", "allure", "gitlab_ci", "load_testing"],
    ["manual_testing", "test_design", "api_testing", "postman", "sql"],
  ],
  ml: [
    ["python", "pytorch", "nlp", "llm", "pandas", "numpy", "scikit_learn", "sql"],
    ["python", "pandas", "numpy", "scikit_learn", "catboost", "statistics", "sql", "spark", "airflow"],
    ["python", "pytorch", "computer_vision", "numpy", "mlops"],
  ],
  security: [
    ["pentest", "burp", "owasp", "appsec", "python", "linux", "ctf", "reverse"],
    ["siem", "network_security", "linux", "bash", "cryptography", "python"],
  ],
};

const ROLE_BY_SPEC: Record<Specialization, string> = {
  backend: "Backend-разработчик",
  frontend: "Frontend-разработчик",
  devops: "DevOps-инженер",
  qa: "QA-инженер",
  ml: "ML-инженер",
  security: "Специалист по ИБ",
};

const SUMMARIES: Record<Specialization, string[]> = {
  backend: [
    "Разрабатывал сервис платежей: приём и сверка транзакций, очереди на Kafka, PostgreSQL с партиционированием.",
    "Переводил монолит на микросервисы, настраивал gRPC-взаимодействие и мониторинг.",
    "Писал API личного кабинета, оптимизировал медленные SQL-запросы, внедрял кэширование в Redis.",
    "Поддерживал высоконагруженный сервис уведомлений: 20k RPS, ретраи, идемпотентность.",
  ],
  frontend: [
    "Делал личный кабинет на React и TypeScript, внедрил дизайн-систему и e2e-тесты на Playwright.",
    "Ускорил загрузку витрины в 2 раза: code splitting, оптимизация изображений, SSR на Next.js.",
    "Разрабатывал админку на Vue, писал компоненты таблиц и форм с валидацией.",
  ],
  devops: ["Перевёл CI/CD на GitLab CI, собрал кластеры Kubernetes в Yandex Cloud через Terraform.", "Настроил мониторинг на Prometheus и Grafana, алерты и дежурства."],
  qa: [
    "Построил автотесты API на pytest, внедрил Allure-отчёты в CI.",
    "Тестировал мобильный банк: тест-дизайн, регресс, нагрузочное тестирование в k6.",
    "Ручное тестирование медицинской информационной системы, ведение тест-кейсов.",
  ],
  ml: [
    "Обучал модели классификации обращений на трансформерах, выводил в прод через MLflow.",
    "Строил модели оттока на CatBoost, проводил A/B-тесты.",
    "Детекция дефектов на производстве: сбор датасета, обучение и деплой модели компьютерного зрения.",
  ],
  security: [
    "Проводил пентесты веб-приложений, писал отчёты и рекомендации разработчикам.",
    "Работал в SOC: разбор инцидентов, правила корреляции в SIEM.",
    "Внедрял SAST/DAST в процесс разработки, обучал команды безопасной разработке.",
  ],
};

const ABOUT: Record<Specialization, string[]> = {
  backend: ["Люблю распределённые системы и понятный код. Интересны платёжные и финансовые продукты.", "Пишу надёжные сервисы под нагрузкой, слежу за метриками и SLO."],
  frontend: ["Делаю быстрые и доступные интерфейсы, внимателен к деталям дизайна.", "Интересуюсь производительностью веба и дизайн-системами."],
  devops: ["Автоматизирую всё, что повторяется дважды. Инфраструктура как код."],
  qa: ["Нахожу баги до пользователей. Люблю автоматизацию и чистые тест-кейсы."],
  ml: ["Довожу модели до продакшена и меряю эффект в деньгах."],
  security: ["Смотрю на систему глазами атакующего, чтобы её защитить."],
};

const SALARY_BASE: Record<Grade, number> = { junior: 115000, middle: 235000, senior: 360000 };
const SPEC_K: Record<Specialization, number> = { backend: 1.05, frontend: 1, devops: 1.08, qa: 0.82, ml: 1.1, security: 1 };

const DISCIPLINES_FOR: Record<Specialization, FspDiscipline[]> = {
  backend: ["product", "algorithmic"],
  frontend: ["product", "algorithmic"],
  devops: ["product", "security"],
  qa: ["product", "algorithmic"],
  ml: ["ai", "algorithmic"],
  security: ["security", "algorithmic"],
};

function iso(now: Date, daysAgo: number): string {
  return new Date(now.getTime() - daysAgo * DAY).toISOString();
}

function ym(now: Date, yearsAgo: number): string {
  const d = new Date(now.getTime() - yearsAgo * 365.25 * DAY);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

function round10k(v: number): number {
  return Math.round(v / 10000) * 10000;
}

function publicCode(rng: Rng): string {
  return `K-${Array.from({ length: 6 }, () => "0123456789ABCDEF"[Math.floor(rng() * 16)]).join("")}`;
}

function thetaFromPercentile(grade: Grade, percentile: number): number {
  const lo = GRADE_THRESHOLD[grade];
  const hi = grade === "senior" ? lo + 1.5 : GRADE_THRESHOLD[grade === "junior" ? "middle" : "senior"];
  return Math.round((lo + (percentile / 100) * (hi - lo)) * 100) / 100;
}

function randomAchievements(rng: Rng, spec: Specialization, owner: number): RawAchievement[] {
  const n = randInt(rng, 1, 3);
  const levels: FspEventLevel[] = ["regional", "regional", "interregional", "national", "international"];
  const places: FspPlace[] = ["first", "second", "third", "finalist", "participant", "participant"];
  const ranks: FspRank[] = ["third", "second", "first", "cms"];
  return Array.from({ length: n }, (_, i): RawAchievement => {
    const discipline = pick(rng, DISCIPLINES_FOR[spec]);
    const year = randInt(rng, 2022, 2026);
    const date = `${year}-${String(randInt(rng, 1, year === 2026 ? 8 : 12)).padStart(2, "0")}-${String(randInt(rng, 1, 28)).padStart(2, "0")}`;
    const isRank = chance(rng, 0.3);
    return {
      id: `ACH-${String(900000 + owner).slice(-6)}-${String(i + 1).padStart(2, "0")}`,
      kind: isRank ? "rank" : "competition",
      discipline,
      rank: isRank ? pick(rng, ranks) : null,
      event_name: null,
      event_level: isRank ? null : pick(rng, levels),
      place: isRank ? null : pick(rng, places),
      team: null,
      role: null,
      date,
      protocol_url: null,
    };
  });
}

function sourcesFor(slugs: string[], rng: Rng, hasFsp: boolean): Record<string, SkillSource[]> {
  const out: Record<string, SkillSource[]> = {};
  slugs.forEach((slug, i) => {
    const src: SkillSource[] = [chance(rng, 0.55) ? "resume" : "declared"];
    if (i < 2) src.unshift("test");
    if (hasFsp && i === 2) src.unshift("fsp");
    out[slug] = src;
  });
  return out;
}

function buildPoolCandidate(rng: Rng, now: Date, spec: Specialization, grade: Grade, index: number): MockCandidate {
  const female = chance(rng, 0.38);
  const first = pick(rng, female ? FEMALE : MALE);
  const lastBase = pick(rng, LAST);
  const last = female ? `${lastBase}а` : lastBase;
  const stack = pick(rng, STACKS[spec]);
  const skillSlugs = stack.slice(0, randInt(rng, Math.min(5, stack.length), stack.length));
  const hasFsp = chance(rng, grade === "junior" ? 0.45 : 0.35);
  const percentile = Math.min(98, Math.max(4, Math.round(rng() * 100)));
  const expYears = grade === "junior" ? randInt(rng, 0, 2) : grade === "middle" ? randInt(rng, 2, 5) : randInt(rng, 5, 12);
  const salary = chance(rng, 0.06) ? null : round10k(SALARY_BASE[grade] * SPEC_K[spec] * (0.8 + rng() * 0.45));
  const formats = pick(rng, [["remote"], ["remote", "hybrid"], ["hybrid", "office"], ["remote", "hybrid", "office"], ["office"]] as WorkFormat[][]);
  const statuses: SearchStatus[] = ["active", "active", "open", "open", "open", "closed"];
  const jobs = Math.min(3, Math.max(1, Math.ceil(expYears / 3)));
  const experience: ExperienceEntry[] = Array.from({ length: jobs }, (_, k) => ({
    position: k === 0 && grade === "senior" ? `Ведущий ${ROLE_BY_SPEC[spec].toLowerCase()}` : ROLE_BY_SPEC[spec],
    company: pick(rng, COMPANY_TYPES),
    start: ym(now, Math.max(0.5, expYears - k * 2)),
    end: k === 0 ? null : ym(now, Math.max(0, expYears - k * 2 - 2)),
    description: pick(rng, SUMMARIES[spec]),
  }));
  const fspLinked = hasFsp || chance(rng, 0.05);
  return {
    id: uuidFrom(rng),
    user_id: null,
    code: publicCode(rng),
    profile: {
      full_name: `${first} ${last}`,
      contacts: {
        email: `dev${index}@example.ru`,
        phone: `+7 9${randInt(rng, 10, 99)} ${randInt(rng, 100, 999)}-${randInt(rng, 10, 99)}-${randInt(rng, 10, 99)}`,
        telegram: `@dev_${index}_${randInt(rng, 10, 99)}`,
      },
      city: pick(rng, CITIES),
      relocation: chance(rng, 0.2),
      work_formats: formats,
      salary_expectation_min: salary,
      about: pick(rng, ABOUT[spec]),
      experience_years: expYears,
      roles: [ROLE_BY_SPEC[spec]],
      soft_skills: sample(rng, ["Коммуникация", "Наставничество", "Ответственность", "Работа в команде", "Самостоятельность", "Аналитическое мышление"], 3),
      industries: sample(rng, INDUSTRIES, randInt(rng, 1, 3)),
      experience,
      skills: skillSlugs,
    },
    skill_sources: sourcesFor(skillSlugs, rng, hasFsp),
    profile_saved_at: iso(now, randInt(rng, 1, 200)),
    privacy: {
      visible: chance(rng, 0.95),
      show_name: chance(rng, 0.2),
      show_fsp: chance(rng, 0.95),
      show_experience: chance(rng, 0.8),
      search_status: pick(rng, statuses),
    },
    surveys: [],
    assignments: [
      {
        id: uuidFrom(rng),
        category: { specialization: spec, grade },
        theta: thetaFromPercentile(grade, percentile),
        strength: percentile / 100,
        reason: "placement",
        session_id: uuidFrom(rng),
        assigned_at: iso(now, randInt(rng, 6, 300)),
      },
    ],
    placement: { open: false, offer: null, offer_grade: null },
    retry_after: {},
    no_category_retry_after: null,
    fsp: {
      fsp_id: fspLinked ? `FSP-${String(100 + index).padStart(6, "0")}` : null,
      linked_at: fspLinked ? iso(now, randInt(rng, 30, 300)) : null,
      last_sync_at: fspLinked ? iso(now, randInt(rng, 0, 20)) : null,
      achievements: hasFsp ? randomAchievements(rng, spec, index) : [],
    },
    last_active_at: iso(now, Math.floor(Math.pow(rng(), 2) * 140)),
  };
}

const POOL: Array<[Specialization, Grade, number]> = [
  ["backend", "junior", 4],
  ["backend", "middle", 8],
  ["backend", "senior", 4],
  ["frontend", "junior", 3],
  ["frontend", "middle", 5],
  ["frontend", "senior", 3],
  ["devops", "junior", 2],
  ["devops", "middle", 3],
  ["devops", "senior", 2],
  ["qa", "junior", 3],
  ["qa", "middle", 3],
  ["qa", "senior", 1],
  ["ml", "junior", 2],
  ["ml", "middle", 3],
  ["ml", "senior", 2],
  ["security", "junior", 1],
  ["security", "middle", 3],
  ["security", "senior", 1],
];

type SeedCompany = Omit<MockCompany, "owner_user_id" | "updated_at">;

const COMPANIES: SeedCompany[] = [
  {
    id: "0192b000-0000-7000-8000-00000000c001",
    name: "Полярный код",
    industry: "Финтех",
    description:
      "Делаем платёжную инфраструктуру для банков и маркетплейсов: приём платежей по СБП и картам, антифрод, сверки. 180 инженеров, офисы в Москве и Санкт-Петербурге.",
    website: "https://polar-code.example",
    city: "Москва",
    size: "100–500 сотрудников",
    contact_email: "hr@polar-code.example",
    contact_phone: "+7 495 000-12-34",
    contact_telegram: "@polarcode_hr",
  },
  {
    id: "0192b000-0000-7000-8000-00000000c002",
    name: "Облачные системы",
    industry: "Облачная инфраструктура",
    description: "Публичное облако и managed-сервисы для бизнеса.",
    website: "https://cloud-sys.example",
    city: "Санкт-Петербург",
    size: "500–1000 сотрудников",
    contact_email: "talents@cloud-sys.example",
    contact_phone: null,
    contact_telegram: "@cloudsys_jobs",
  },
  {
    id: "0192b000-0000-7000-8000-00000000c003",
    name: "Ритм Маркет",
    industry: "E-commerce",
    description: "Маркетплейс товаров для дома, 4 млн покупателей в месяц.",
    website: "https://ritm-market.example",
    city: "Москва",
    size: "1000+ сотрудников",
    contact_email: "it-hiring@ritm-market.example",
    contact_phone: null,
    contact_telegram: null,
  },
  {
    id: "0192b000-0000-7000-8000-00000000c004",
    name: "Атом Софт",
    industry: "Госсектор",
    description: "Разрабатываем цифровые сервисы для региональных органов власти.",
    website: "https://atomsoft.example",
    city: "Казань",
    size: "100–500 сотрудников",
    contact_email: "job@atomsoft.example",
    contact_phone: "+7 843 000-55-66",
    contact_telegram: null,
  },
  {
    id: "0192b000-0000-7000-8000-00000000c005",
    name: "Квант Игры",
    industry: "Геймдев",
    description: "Мобильные онлайн-игры с аудиторией 10 млн игроков.",
    website: "https://kvant-games.example",
    city: "Новосибирск",
    size: "100–500 сотрудников",
    contact_email: "hr@kvant-games.example",
    contact_phone: null,
    contact_telegram: "@kvant_hr",
  },
  {
    id: "0192b000-0000-7000-8000-00000000c006",
    name: "Вектор Данных",
    industry: "Аналитика и ИИ",
    description: "ML-решения для ритейла и промышленности: прогноз спроса, компьютерное зрение, LLM-ассистенты.",
    website: "https://vector-data.example",
    city: "Москва",
    size: "50–100 сотрудников",
    contact_email: "team@vector-data.example",
    contact_phone: null,
    contact_telegram: "@vectordata",
  },
  {
    id: "0192b000-0000-7000-8000-00000000c007",
    name: "МедТех Лаб",
    industry: "Медицина",
    description: "Медицинские информационные системы для клиник.",
    website: "https://medtech-lab.example",
    city: "Екатеринбург",
    size: "50–100 сотрудников",
    contact_email: "hr@medtech-lab.example",
    contact_phone: null,
    contact_telegram: null,
  },
];

const [POLAR, CLOUD, RITM, ATOM, KVANT, VECTOR, MEDTECH] = COMPANIES.map((c) => c.id) as [string, string, string, string, string, string, string];

export const DEMO_COMPANY_ID = POLAR;
export const DEMO_CANDIDATE_ID = "0192c000-0000-7000-8000-0000000a0001";
const DEMO_CANDIDATE_USER = "0192d000-0000-7000-8000-000000001001";
const DEMO_EMPLOYER_USER = "0192d000-0000-7000-8000-000000001002";
export const DEMO_NEED_ID = "0192e000-0000-7000-8000-000000002001";
const DEMO_FSP_ID = "FSP-000002";

function grantedConsents(role: "candidate" | "employer", at: string, rng: Rng): UserConsent[] {
  return CONSENT_DOCS.filter((d) => d.required_for.includes(role)).map((d) => ({
    id: uuidFrom(rng),
    document_id: d.id,
    kind: d.kind,
    version: d.version,
    title: d.title,
    granted_at: at,
    revoked_at: null,
  }));
}

function demoCandidate(now: Date): MockCandidate {
  const rng = mulberry32(77);
  return {
    id: DEMO_CANDIDATE_ID,
    user_id: DEMO_CANDIDATE_USER,
    code: "K-2048AF",
    profile: {
      full_name: "Мария Кузнецова",
      contacts: { email: "maria.kuznetsova@example.ru", phone: "+7 916 555-12-34", telegram: "@maria_kuz_dev" },
      city: "Москва",
      relocation: false,
      work_formats: ["remote", "hybrid"],
      salary_expectation_min: 300000,
      about: "Backend-разработчица, 4 года в финтехе. Пишу на Go, проектирую сервисы вокруг Kafka и PostgreSQL. Интересны платёжные системы и антифрод.",
      experience_years: 4,
      roles: ["Backend-разработчик"],
      soft_skills: ["Ответственность", "Коммуникация", "Наставничество"],
      industries: ["fintech", "ecommerce"],
      experience: [
        {
          company: "Финтех-компания, 1000+ сотрудников",
          position: "Backend-разработчик",
          start: ym(now, 2.5),
          end: null,
          description: "Сервис платежей по СБП: идемпотентный приём вебхуков, очереди на Kafka, сверки с банками, PostgreSQL с партиционированием.",
        },
        {
          company: "Маркетплейс, 5000+ сотрудников",
          position: "Младший backend-разработчик",
          start: ym(now, 4.5),
          end: ym(now, 2.5),
          description: "API каталога на Go, кэширование в Redis, оптимизация медленных SQL-запросов.",
        },
      ],
      skills: ["go", "postgresql", "kafka", "docker", "redis", "grpc", "kubernetes", "microservices"],
    },
    skill_sources: {
      go: ["test", "declared"],
      postgresql: ["test", "resume"],
      kafka: ["resume"],
      docker: ["resume"],
      redis: ["resume"],
      grpc: ["declared"],
      kubernetes: ["declared"],
      microservices: ["fsp", "declared"],
    },
    profile_saved_at: iso(now, 12),
    privacy: { visible: true, show_name: false, show_fsp: true, show_experience: true, search_status: "open" },
    surveys: [
      {
        id: uuidFrom(rng),
        created_at: iso(now, 55),
        direction: "development",
        specialization: "backend",
        claimed_grade: "senior",
        skills: ["go", "postgresql", "kafka", "docker", "redis", "grpc", "kubernetes"],
        industries: ["fintech", "ecommerce"],
        experience_years: 4,
        self_assessment: { languages: 4, databases: 4, web: 3, algorithms: 3, architecture: 3 },
      },
    ],
    assignments: [
      {
        id: uuidFrom(rng),
        category: { specialization: "backend", grade: "middle" },
        theta: 0.62,
        strength: 0.62,
        reason: "placement",
        session_id: "0192f000-0000-7000-8000-000000003002",
        assigned_at: iso(now, 54),
      },
    ],
    placement: { open: false, offer: null, offer_grade: null },
    retry_after: {},
    no_category_retry_after: null,
    fsp: { fsp_id: DEMO_FSP_ID, linked_at: iso(now, 50), last_sync_at: iso(now, 2), achievements: FSP_REGISTRY[DEMO_FSP_ID] ?? [] },
    last_active_at: iso(now, 1),
  };
}

function demoAssessments(now: Date): MockAssessment[] {
  const at = (d: number, mins: number) => new Date(now.getTime() - d * DAY + mins * 60000).toISOString();
  const base = { candidate_id: DEMO_CANDIDATE_ID, items: [], posterior: [], last_activity_at: at(54, 0) };
  return [
    {
      ...base,
      session: {
        id: "0192f000-0000-7000-8000-000000003001",
        specialization: "backend",
        claimed_grade: "senior",
        attempt_kind: "placement",
        status: "failed",
        started_at: at(54, -50),
        finished_at: at(54, -31),
        min_items: 8,
        max_items: 20,
        result: {
          decision: { passed: false, theta: 0.64, se: 0.27, grade_measured: "middle", confident_above_next: false, person_fit_lz: 0.4, aberrant: false, items_administered: 14 },
          test_strength: null,
          assigned_grade: null,
          needs_review: false,
          topics: [
            { topic: "languages", topic_ru: "Языки программирования", answered: 3, correct: 3 },
            { topic: "databases", topic_ru: "Базы данных и SQL", answered: 3, correct: 2 },
            { topic: "web", topic_ru: "HTTP и веб", answered: 3, correct: 2 },
            { topic: "algorithms", topic_ru: "Алгоритмы и структуры данных", answered: 3, correct: 1 },
            { topic: "architecture", topic_ru: "Архитектура и надёжность", answered: 2, correct: 1 },
          ],
          offers: [],
        },
      },
    },
    {
      ...base,
      session: {
        id: "0192f000-0000-7000-8000-000000003002",
        specialization: "backend",
        claimed_grade: "middle",
        attempt_kind: "placement_step_down",
        status: "passed",
        started_at: at(54, -25),
        finished_at: at(54, -9),
        min_items: 8,
        max_items: 20,
        result: {
          decision: { passed: true, theta: 0.62, se: 0.24, grade_measured: "middle", confident_above_next: false, person_fit_lz: 0.6, aberrant: false, items_administered: 10 },
          test_strength: 0.62,
          assigned_grade: "middle",
          needs_review: false,
          topics: [
            { topic: "languages", topic_ru: "Языки программирования", answered: 2, correct: 2 },
            { topic: "databases", topic_ru: "Базы данных и SQL", answered: 2, correct: 2 },
            { topic: "web", topic_ru: "HTTP и веб", answered: 2, correct: 2 },
            { topic: "algorithms", topic_ru: "Алгоритмы и структуры данных", answered: 2, correct: 1 },
            { topic: "architecture", topic_ru: "Архитектура и надёжность", answered: 2, correct: 2 },
          ],
          offers: [],
        },
      },
    },
  ];
}

function vacancies(now: Date): MockVacancy[] {
  const v = (
    id: string,
    company_id: string,
    title: string,
    specialization: Specialization,
    grade: Grade,
    salary: [number, number],
    work_format: WorkFormat,
    city: string,
    skills: string[],
    description: string,
    daysAgo: number,
    status: MockVacancy["status"] = "published",
  ): MockVacancy => ({
    id,
    company_id,
    title,
    description,
    specialization,
    grade,
    skills,
    salary: { min: salary[0], max: salary[1] },
    work_format,
    city,
    status,
    published_at: status === "published" ? iso(now, daysAgo) : null,
    created_at: iso(now, daysAgo + 1),
    updated_at: iso(now, daysAgo),
  });
  return [
    v("0192e100-0000-7000-8000-000000004001", POLAR, "Go-разработчик в команду платежей", "backend", "middle", [260000, 340000], "hybrid", "Москва",
      ["go", "postgresql", "kafka", "docker", "kubernetes"],
      "Развиваем платёжный шлюз: СБП, карты, антифрод-сигналы.\n\nЗадачи: новые платёжные методы, идемпотентная обработка вебхуков банков, сверки, оптимизация PostgreSQL.\n\nПредлагаем: гибрид 2 дня в офисе у м. Белорусская, ДМС, бюджет на конференции.", 6),
    v("0192e100-0000-7000-8000-000000004002", POLAR, "Senior Frontend-разработчик (React)", "frontend", "senior", [330000, 420000], "remote", "Москва",
      ["typescript", "react", "nextjs", "a11y", "playwright"],
      "Личный кабинет мерчанта: дашборды, отчёты, настройки платёжных методов. Нужен опыт с дизайн-системами и доступностью.", 0, "draft"),
    v("0192e100-0000-7000-8000-000000004003", CLOUD, "Platform engineer (Kubernetes)", "devops", "senior", [350000, 450000], "remote", "Санкт-Петербург",
      ["kubernetes", "terraform", "helm", "prometheus", "go"],
      "Managed Kubernetes для клиентов облака: операторы, автоскейлинг, обновления без простоя.", 3),
    v("0192e100-0000-7000-8000-000000004004", CLOUD, "Backend-разработчик Go (сетевые сервисы)", "backend", "senior", [380000, 480000], "remote", "Санкт-Петербург",
      ["go", "grpc", "linux", "postgresql", "kubernetes"],
      "Сервисы виртуальных сетей облака: балансировщики, VPC, NAT. Высокие требования к надёжности.", 9),
    v("0192e100-0000-7000-8000-000000004005", RITM, "Frontend-разработчик (React, TypeScript)", "frontend", "middle", [220000, 290000], "hybrid", "Москва",
      ["typescript", "react", "redux", "css", "jest"],
      "Витрина маркетплейса: каталог, карточка товара, корзина. A/B-тесты каждую неделю.", 2),
    v("0192e100-0000-7000-8000-000000004006", RITM, "QA Automation (Python)", "qa", "middle", [180000, 240000], "remote", "Москва",
      ["python", "pytest", "api_testing", "playwright", "allure"],
      "Автотесты API и UI для сервисов заказов и доставки.", 11),
    v("0192e100-0000-7000-8000-000000004007", ATOM, "Java-разработчик (госуслуги)", "backend", "junior", [110000, 150000], "office", "Казань",
      ["java", "spring", "postgresql", "sql"],
      "Сервисы записи и уведомлений для региональных порталов. Наставник на испытательный срок.", 4),
    v("0192e100-0000-7000-8000-000000004008", ATOM, "Аналитик SOC", "security", "middle", [200000, 260000], "office", "Казань",
      ["siem", "network_security", "linux"],
      "Мониторинг и разбор инцидентов, написание правил корреляции.", 15),
    v("0192e100-0000-7000-8000-000000004009", KVANT, "C#-разработчик игровых серверов", "backend", "middle", [240000, 310000], "remote", "Новосибирск",
      ["csharp", "postgresql", "redis", "docker"],
      "Серверная логика PvP-режимов, матчмейкинг, лидерборды.", 7),
    v("0192e100-0000-7000-8000-000000004010", VECTOR, "ML-инженер (NLP, LLM)", "ml", "middle", [280000, 360000], "hybrid", "Москва",
      ["python", "pytorch", "nlp", "llm", "mlops"],
      "LLM-ассистент для операторов поддержки ритейла: RAG, дообучение, оценка качества.", 5),
    v("0192e100-0000-7000-8000-000000004011", VECTOR, "Junior Data Scientist", "ml", "junior", [120000, 160000], "remote", "Москва",
      ["python", "pandas", "scikit_learn", "sql", "statistics"],
      "Прогноз спроса для сетей магазинов. Сильная команда и менторство.", 1),
    v("0192e100-0000-7000-8000-000000004012", MEDTECH, "Тестировщик медицинских систем", "qa", "junior", [90000, 120000], "office", "Екатеринбург",
      ["manual_testing", "test_design", "sql", "postman"],
      "Ручное и API-тестирование МИС, работа с требованиями.", 20),
  ];
}

function needs(now: Date): MockNeed[] {
  return [
    {
      id: DEMO_NEED_ID,
      company_id: POLAR,
      title: "Backend-разработчик в команду платежей",
      specialization: "backend",
      grades: ["middle"],
      allow_adjacent_grades: false,
      must_skills: ["go", "postgresql", "kafka", "docker"],
      nice_skills: ["kubernetes", "grpc", "clickhouse", "redis"],
      salary: { min: 260000, max: 340000 },
      work_formats: ["hybrid", "remote"],
      city: "Москва",
      industries: ["fintech"],
      description:
        "Команда из 7 человек развивает платёжный шлюз: приём платежей по СБП и картам, антифрод-сигналы, сверки с банками. Go, PostgreSQL, Kafka, деплой в Kubernetes.",
      status: "active",
      filters: {},
      shortlist_generated_at: null,
      created_at: iso(now, 9),
      updated_at: iso(now, 2),
    },
    {
      id: "0192e000-0000-7000-8000-000000002002",
      company_id: POLAR,
      title: "Frontend-разработчик личного кабинета",
      specialization: "frontend",
      grades: ["middle", "senior"],
      allow_adjacent_grades: false,
      must_skills: ["typescript", "react"],
      nice_skills: ["nextjs", "a11y", "playwright"],
      salary: { min: 240000, max: 380000 },
      work_formats: ["remote", "hybrid"],
      city: "Москва",
      industries: ["fintech"],
      description: "Личный кабинет мерчанта: дашборды, отчёты, настройки платёжных методов, дизайн-система.",
      status: "active",
      filters: {},
      shortlist_generated_at: null,
      created_at: iso(now, 4),
      updated_at: iso(now, 4),
    },
  ];
}

interface InvitationSeed {
  company_id: string;
  candidate_id: string;
  status: InvitationStatus;
  title: string;
  message: string;
  salary: [number, number];
  work_format: WorkFormat | null;
  daysAgo: number;
  need_id?: string | null;
  vacancy_id?: string | null;
  decline?: DeclineReason;
  contact: string;
}

function invitation(rng: Rng, now: Date, p: InvitationSeed): MockInvitation {
  const created = iso(now, p.daysAgo);
  const events: MockInvitation["events"] = [{ status: "sent", at: created }];
  const step = (h: number) => new Date(new Date(created).getTime() + h * 3600000).toISOString();
  let viewed_at: string | null = null;
  let responded_at: string | null = null;
  if (p.status === "viewed" || p.status === "accepted" || p.status === "declined") {
    viewed_at = step(randInt(rng, 2, 20));
    events.push({ status: "viewed", at: viewed_at });
  }
  if (p.status === "accepted" || p.status === "declined") {
    responded_at = step(randInt(rng, 22, 40));
    events.push({ status: p.status, at: responded_at });
  }
  if (p.status === "expired") events.push({ status: "expired", at: iso(now, p.daysAgo - 14) });
  if (p.status === "withdrawn") events.push({ status: "withdrawn", at: step(30) });
  return {
    id: uuidFrom(rng),
    company_id: p.company_id,
    candidate_id: p.candidate_id,
    status: p.status,
    title: p.title,
    message: p.message,
    salary: { min: p.salary[0], max: p.salary[1] },
    work_format: p.work_format,
    contact: p.contact,
    need_id: p.need_id ?? null,
    vacancy_id: p.vacancy_id ?? null,
    created_at: created,
    viewed_at,
    responded_at,
    expires_at: iso(now, p.daysAgo - 14),
    decline_reason: p.decline ?? null,
    decline_comment: p.decline === "salary" ? "Рассматриваю предложения от 320 000 ₽" : null,
    events,
    idempotency_key: null,
  };
}

export function buildSeed(now: Date): MockDb {
  const rng = mulberry32(20261005);
  const pool: MockCandidate[] = [];
  let index = 1;
  for (const [spec, grade, count] of POOL) {
    for (let i = 0; i < count; i++) pool.push(buildPoolCandidate(rng, now, spec, grade, index++));
  }

  const users: MockUser[] = [
    {
      id: DEMO_CANDIDATE_USER,
      email: DEMO_CANDIDATE_EMAIL,
      password: DEMO_PASSWORD,
      role: "candidate",
      email_verified: true,
      verify_token: null,
      candidate_id: DEMO_CANDIDATE_ID,
      company_id: null,
      created_at: iso(now, 60),
      consents: grantedConsents("candidate", iso(now, 60), rng),
    },
    {
      id: DEMO_EMPLOYER_USER,
      email: DEMO_EMPLOYER_EMAIL,
      password: DEMO_PASSWORD,
      role: "employer",
      email_verified: true,
      verify_token: null,
      candidate_id: null,
      company_id: POLAR,
      created_at: iso(now, 30),
      consents: grantedConsents("employer", iso(now, 30), rng),
    },
  ];

  const companies: MockCompany[] = COMPANIES.map((c) => ({ ...c, owner_user_id: c.id === POLAR ? DEMO_EMPLOYER_USER : null, updated_at: iso(now, 10) }));

  const invitations: MockInvitation[] = [
    invitation(rng, now, {
      company_id: CLOUD,
      candidate_id: DEMO_CANDIDATE_ID,
      status: "sent",
      title: "Go-разработчик в платформенную команду",
      message:
        "Мария, здравствуйте! Мы строим сетевые сервисы облака на Go и ищем инженера уровня Middle+. Ваш результат теста и опыт с Kafka выглядят как то, что нам нужно. Расскажем подробнее на коротком созвоне.",
      salary: [290000, 360000],
      work_format: "remote",
      daysAgo: 1,
      contact: "Telegram @cloudsys_jobs",
      vacancy_id: "0192e100-0000-7000-8000-000000004004",
    }),
    invitation(rng, now, {
      company_id: RITM,
      candidate_id: DEMO_CANDIDATE_ID,
      status: "viewed",
      title: "Backend-разработчик в сервис заказов",
      message: "Добрый день! Ищем разработчика в сервис оформления заказов (Go, PostgreSQL, Kafka). Нагрузка — до 5000 заказов в минуту в пике.",
      salary: [260000, 300000],
      work_format: "hybrid",
      daysAgo: 4,
      contact: "it-hiring@ritm-market.example",
    }),
    invitation(rng, now, {
      company_id: KVANT,
      candidate_id: DEMO_CANDIDATE_ID,
      status: "accepted",
      title: "Backend игровых сервисов",
      message: "Привет! Нужен человек на сервисы матчмейкинга и платежей внутри игры. Будем рады пообщаться.",
      salary: [300000, 380000],
      work_format: "remote",
      daysAgo: 12,
      contact: "Telegram @kvant_hr",
    }),
    invitation(rng, now, {
      company_id: ATOM,
      candidate_id: DEMO_CANDIDATE_ID,
      status: "declined",
      title: "Разработчик портала госуслуг",
      message: "Здравствуйте! Приглашаем в команду портала региональных госуслуг. Офис в Казани.",
      salary: [180000, 220000],
      work_format: "office",
      daysAgo: 20,
      decline: "salary",
      contact: "+7 843 000-55-66",
    }),
  ];

  const backendPool = pool.filter((c) => c.assignments[0]?.category.specialization === "backend");
  const otherPool = pool.filter((c) => c.assignments[0]?.category.specialization !== "backend");
  const plan: Array<[InvitationStatus, number, DeclineReason?]> = [
    ["sent", 0],
    ["sent", 1],
    ["sent", 2],
    ["viewed", 1],
    ["viewed", 3],
    ["viewed", 5],
    ["accepted", 4],
    ["accepted", 6],
    ["declined", 2, "salary"],
    ["declined", 3, "salary"],
    ["declined", 5, "format"],
    ["declined", 6, "stack"],
    ["declined", 9, "salary"],
    ["expired", 16],
  ];
  const targets = [...backendPool, ...otherPool];
  plan.forEach(([status, daysAgo, decline], i) => {
    const target = targets[i];
    if (!target) return;
    const isBackend = target.assignments[0]?.category.specialization === "backend";
    invitations.push(
      invitation(rng, now, {
        company_id: POLAR,
        candidate_id: target.id,
        status,
        title: isBackend ? "Backend-разработчик в команду платежей" : "Инженер в платёжную платформу",
        message: "Здравствуйте! Нам понравился ваш профиль — особенно результат теста. Предлагаем обсудить роль в команде платёжного шлюза.",
        salary: isBackend ? [260000, 340000] : [220000, 300000],
        work_format: "hybrid",
        daysAgo,
        need_id: isBackend ? DEMO_NEED_ID : null,
        decline,
        contact: "Telegram @polarcode_hr",
      }),
    );
  });

  const applications: MockApplication[] = [
    {
      id: uuidFrom(rng),
      vacancy_id: "0192e100-0000-7000-8000-000000004004",
      candidate_id: DEMO_CANDIDATE_ID,
      status: "viewed",
      cover_letter: "Есть опыт с сетевыми сервисами на Go и gRPC, интересна облачная инфраструктура.",
      created_at: iso(now, 6),
      updated_at: iso(now, 3),
    },
    ...backendPool.slice(10, 13).map(
      (c, i): MockApplication => ({
        id: uuidFrom(rng),
        vacancy_id: "0192e100-0000-7000-8000-000000004001",
        candidate_id: c.id,
        status: (["sent", "viewed", "accepted"] as const)[i] ?? "sent",
        cover_letter: i === 0 ? "Давно хочу в финтех, есть опыт с Kafka и PostgreSQL." : "",
        created_at: iso(now, 5 - i),
        updated_at: iso(now, 4 - i),
      }),
    ),
  ];

  const tasks: MockTask[] = [
    {
      id: "0192e200-0000-7000-8000-000000005001",
      company_id: POLAR,
      title: "Идемпотентный приём вебхуков",
      body: "Банк присылает вебхуки о статусе платежа и может повторить один и тот же вебхук до 5 раз, иногда параллельно. Опишите, как обеспечить ровно однократное изменение статуса платежа: схема данных, транзакции, что делать с порядком событий.",
      kind: "approach",
      specialization: "backend",
      grades: ["middle", "senior"],
      skills: ["postgresql", "kafka"],
      active: true,
      created_at: iso(now, 14),
    },
    {
      id: "0192e200-0000-7000-8000-000000005002",
      company_id: CLOUD,
      title: "Найти гонку в воркере",
      body: "Воркер читает задачи из канала и увеличивает общий счётчик:\n\n```go\nvar processed int\nfor i := 0; i < workers; i++ {\n    go func() {\n        for t := range tasks {\n            handle(t)\n            processed++\n        }\n    }()\n}\n```\n\nНайдите проблему и предложите исправление.",
      kind: "solution",
      specialization: "backend",
      grades: ["middle"],
      skills: ["go"],
      active: true,
      created_at: iso(now, 20),
    },
    {
      id: "0192e200-0000-7000-8000-000000005003",
      company_id: RITM,
      title: "Индексы для отчёта по заказам",
      body: "Отчёт строится запросом с фильтром по `merchant_id` и диапазону `created_at`, сортировка по `created_at DESC`. Какой индекс создать и почему?",
      kind: "solution",
      specialization: "backend",
      grades: ["junior", "middle"],
      skills: ["postgresql", "sql"],
      active: true,
      created_at: iso(now, 40),
    },
  ];

  const taskAssignments: MockTaskAssignment[] = [
    {
      id: uuidFrom(rng),
      task_id: "0192e200-0000-7000-8000-000000005002",
      candidate_id: DEMO_CANDIDATE_ID,
      status: "assigned",
      answer: null,
      rating: null,
      review_comment: null,
      assigned_at: iso(now, 2),
      due_at: iso(now, -5),
      submitted_at: null,
      reviewed_at: null,
    },
    {
      id: uuidFrom(rng),
      task_id: "0192e200-0000-7000-8000-000000005003",
      candidate_id: DEMO_CANDIDATE_ID,
      status: "reviewed",
      answer: "CREATE INDEX ON orders (merchant_id, created_at DESC); — составной индекс: равенство по merchant_id, затем диапазон и сортировка по created_at читаются из индекса без отдельной сортировки.",
      rating: 5,
      review_comment: "Отличный ответ. Можно ещё упомянуть INCLUDE для покрывающего индекса.",
      assigned_at: iso(now, 21),
      due_at: iso(now, 14),
      submitted_at: iso(now, 18),
      reviewed_at: iso(now, 16),
    },
    ...backendPool.slice(0, 3).map(
      (c, i): MockTaskAssignment => ({
        id: uuidFrom(rng),
        task_id: "0192e200-0000-7000-8000-000000005001",
        candidate_id: c.id,
        status: i < 2 ? "submitted" : "assigned",
        answer:
          i === 0
            ? "Уникальный ключ (bank_id, event_id) в таблице входящих событий, вставка с ON CONFLICT DO NOTHING в одной транзакции с изменением статуса. Порядок — через версию статуса и конечный автомат: переход назад игнорируем."
            : i === 1
              ? "Хранить обработанные id в Redis с TTL и проверять перед обработкой."
              : null,
        rating: null,
        review_comment: null,
        assigned_at: iso(now, 3),
        due_at: iso(now, -4),
        submitted_at: i < 2 ? iso(now, 1 - i) : null,
        reviewed_at: null,
      }),
    ),
  ];

  return {
    version: SEED_VERSION,
    seeded_at: now.toISOString(),
    session: null,
    users,
    candidates: [demoCandidate(now), ...pool],
    companies,
    needs: needs(now),
    shortlists: {},
    invitations,
    vacancies: vacancies(now),
    applications,
    assessments: demoAssessments(now),
    tasks,
    task_assignments: taskAssignments,
    contact_log: [],
    reports: [],
  };
}
