import type {
  ApplicationStatus,
  AssessmentStatus,
  AssessmentTopic,
  AttemptKind,
  Category,
  DeclineReason,
  Direction,
  FeatureKey,
  FspAchievement,
  FspDiscipline,
  FspEventLevel,
  FspPlace,
  FspRank,
  Grade,
  Industry,
  InvitationStatus,
  NeedStatus,
  SearchStatus,
  SkillSource,
  Specialization,
  TaskKind,
  TaskStatus,
  TrustLevel,
  VacancyStatus,
  WorkFormat,
} from "@/api/types";

export const GRADES: Grade[] = ["junior", "middle", "senior"];
export const SPECIALIZATIONS: Specialization[] = ["backend", "frontend", "devops", "qa", "ml", "security"];
export const WORK_FORMATS: WorkFormat[] = ["remote", "hybrid", "office"];
export const INDUSTRIES: Industry[] = [
  "fintech",
  "ecommerce",
  "gamedev",
  "govtech",
  "telecom",
  "edtech",
  "healthtech",
  "media",
  "manufacturing",
  "logistics",
  "outsourcing",
];

export const GRADE_LABEL: Record<Grade, string> = {
  junior: "Junior",
  middle: "Middle",
  senior: "Senior",
};

export const GRADE_HINT: Record<Grade, string> = {
  junior: "До ~2 лет опыта, решает задачи с поддержкой команды",
  middle: "Самостоятельно ведёт задачи и фичи целиком",
  senior: "Отвечает за архитектуру и качество, растит команду",
};

export const SPEC_LABEL: Record<Specialization, string> = {
  backend: "Backend",
  frontend: "Frontend",
  devops: "DevOps",
  qa: "QA",
  ml: "ML",
  security: "ИБ",
};

export const SPEC_LONG_LABEL: Record<Specialization, string> = {
  backend: "Бэкенд-разработка",
  frontend: "Фронтенд-разработка",
  devops: "DevOps / SRE",
  qa: "Тестирование (QA)",
  ml: "ML / Data Science",
  security: "Информационная безопасность",
};

export const DIRECTION_LABEL: Record<Direction, string> = {
  development: "Разработка",
  infrastructure: "Инфраструктура",
  quality: "Качество",
  data_ai: "Данные и ИИ",
  security: "Безопасность",
};

export const SPEC_DIRECTION: Record<Specialization, Direction> = {
  backend: "development",
  frontend: "development",
  devops: "infrastructure",
  qa: "quality",
  ml: "data_ai",
  security: "security",
};

export const INDUSTRY_LABEL: Record<Industry, string> = {
  fintech: "Финтех и банки",
  ecommerce: "E-commerce и ритейл",
  gamedev: "Геймдев",
  govtech: "Госсектор",
  telecom: "Телеком",
  edtech: "Образование",
  healthtech: "Медицина",
  media: "Медиа и контент",
  manufacturing: "Промышленность",
  logistics: "Логистика и транспорт",
  outsourcing: "Заказная разработка",
};

export function categoryKey(c: Category): string {
  return `${c.specialization}:${c.grade}`;
}

export function categoryLabel(c: Category): string {
  return `${SPEC_LABEL[c.specialization]} · ${GRADE_LABEL[c.grade]}`;
}

export function categoryLongLabel(c: Category): string {
  return `${SPEC_LONG_LABEL[c.specialization]} · ${GRADE_LABEL[c.grade]}`;
}

export const WORK_FORMAT_LABEL: Record<WorkFormat, string> = {
  office: "Офис",
  hybrid: "Гибрид",
  remote: "Удалённо",
};

export const SEARCH_STATUS_LABEL: Record<SearchStatus, string> = {
  active: "Активно ищу работу",
  open: "Открыт к предложениям",
  closed: "Не ищу работу",
};

export const INVITATION_STATUS_LABEL: Record<InvitationStatus, string> = {
  sent: "Отправлено",
  viewed: "Просмотрено",
  accepted: "Принято",
  declined: "Отклонено",
  expired: "Истекло",
  withdrawn: "Отозвано",
};

export const RECEIVED_INVITATION_STATUS_LABEL: Record<InvitationStatus, string> = {
  sent: "Новое",
  viewed: "Просмотрено",
  accepted: "Принято",
  declined: "Отклонено",
  expired: "Истекло",
  withdrawn: "Отозвано компанией",
};

export const APPLICATION_STATUS_LABEL: Record<ApplicationStatus, string> = {
  sent: "Отправлен",
  viewed: "Просмотрен",
  accepted: "Приглашён",
  rejected: "Отказ",
  withdrawn: "Отозван",
};

export const ASSESSMENT_STATUS_LABEL: Record<AssessmentStatus, string> = {
  in_progress: "В процессе",
  passed: "Пройден",
  failed: "Не пройден",
  expired: "Прерван",
};

export const VACANCY_STATUS_LABEL: Record<VacancyStatus, string> = {
  draft: "Черновик",
  published: "Опубликована",
  closed: "Закрыта",
};

export const NEED_STATUS_LABEL: Record<NeedStatus, string> = {
  active: "Активна",
  closed: "Закрыта",
};

export const TASK_STATUS_LABEL: Record<TaskStatus, string> = {
  assigned: "Ждёт ответа",
  submitted: "На проверке",
  reviewed: "Проверено",
  skipped: "Пропущено",
  expired: "Срок истёк",
};

export const TASK_KIND_LABEL: Record<TaskKind, string> = {
  solution: "Решить задачу",
  approach: "Предложить подход",
};

export const DECLINE_REASON_LABEL: Record<DeclineReason, string> = {
  salary: "Зарплата",
  stack: "Стек",
  format: "Формат работы",
  company: "Компания",
  not_looking: "Не ищу работу",
  other: "Другое",
};

export const DECLINE_REASONS: DeclineReason[] = ["salary", "stack", "format", "company", "not_looking", "other"];

export const SKILL_SOURCE_LABEL: Record<SkillSource, string> = {
  test: "подтверждён тестом",
  fsp: "подтверждён ФСП",
  resume: "из резюме",
  declared: "заявлен",
};

export const SKILL_SOURCE_SHORT: Record<SkillSource, string> = {
  test: "тест",
  fsp: "ФСП",
  resume: "резюме",
  declared: "заявлен",
};

const SOURCE_RANK: Record<SkillSource, number> = { test: 0, fsp: 1, resume: 2, declared: 3 };

export function strongestSource(sources: SkillSource[]): SkillSource {
  return [...sources].sort((a, b) => SOURCE_RANK[a] - SOURCE_RANK[b])[0] ?? "declared";
}

export const FEATURE_LABEL: Record<FeatureKey, string> = {
  must: "Обязательные навыки",
  test: "Результат теста",
  grade: "Грейд",
  fsp: "Достижения ФСП",
  nice: "Желательные навыки",
  salary: "Зарплата",
  fresh: "Актуальность",
  text: "Описание",
};

export const TOPIC_LABEL: Record<AssessmentTopic, string> = {
  algorithms: "Алгоритмы и структуры данных",
  databases: "Базы данных и SQL",
  web: "HTTP и веб",
  languages: "Языки программирования",
  architecture: "Архитектура и надёжность",
  tools: "Git и командная строка",
  javascript: "JavaScript",
  layout: "HTML и CSS",
  frameworks: "React",
  networking: "Сети",
  linux: "Linux и автоматизация",
  containers: "Контейнеры и оркестрация",
  observability: "Мониторинг и SRE",
  test_design: "Тест-дизайн",
  qa_process: "Процессы тестирования",
  test_automation: "Автоматизация и API",
  statistics: "Статистика и вероятности",
  ml_metrics: "Метрики качества моделей",
  modeling: "Обучение моделей",
  data_tools: "Инструменты анализа данных",
  appsec: "Безопасность приложений",
  cryptography: "Криптография",
  network_security: "Сетевая безопасность",
};

/** Topics balanced by the adaptive test; also the survey self-assessment scale. Not in /catalog yet. */
export const SPEC_TOPICS: Record<Specialization, AssessmentTopic[]> = {
  backend: ["languages", "databases", "web", "algorithms", "architecture"],
  frontend: ["javascript", "layout", "frameworks", "web", "algorithms"],
  devops: ["linux", "containers", "networking", "observability", "tools"],
  qa: ["test_design", "qa_process", "test_automation", "databases", "web"],
  ml: ["statistics", "ml_metrics", "modeling", "data_tools", "algorithms"],
  security: ["appsec", "cryptography", "network_security", "linux", "web"],
};

export const FSP_DISCIPLINE_LABEL: Record<FspDiscipline, string> = {
  algorithmic: "Алгоритмическое программирование",
  product: "Продуктовое программирование",
  security: "Программирование систем информационной безопасности",
  ai: "Программирование систем искусственного интеллекта",
  robotics: "Программирование робототехники",
  uav: "Программирование беспилотных авиационных систем",
};

const FSP_DISCIPLINE_DATIVE: Record<FspDiscipline, string> = {
  algorithmic: "алгоритмическому программированию",
  product: "продуктовому программированию",
  security: "программированию систем информационной безопасности",
  ai: "программированию систем искусственного интеллекта",
  robotics: "программированию робототехники",
  uav: "программированию беспилотных авиационных систем",
};

const FSP_LEVEL_GENITIVE: Record<FspEventLevel, string> = {
  regional: "региональных",
  interregional: "межрегиональных",
  national: "всероссийских",
  international: "международных",
};

export const FSP_LEVEL_LABEL: Record<FspEventLevel, string> = {
  regional: "Региональный уровень",
  interregional: "Межрегиональный уровень",
  national: "Всероссийский уровень",
  international: "Международный уровень",
};

export const FSP_RANK_LABEL: Record<FspRank, string> = {
  youth: "Юношеский разряд",
  third: "III спортивный разряд",
  second: "II спортивный разряд",
  first: "I спортивный разряд",
  cms: "Кандидат в мастера спорта",
  ms: "Мастер спорта",
};

export const FSP_PLACE_LABEL: Record<FspPlace, string> = {
  first: "Победитель (1 место)",
  second: "Призёр (2 место)",
  third: "Призёр (3 место)",
  finalist: "Финалист",
  participant: "Участник",
};

/** «Призёр (2 место) всероссийских соревнований по продуктовому программированию, 2025» */
export function fspAchievementTitle(a: Pick<FspAchievement, "kind" | "discipline" | "rank" | "event_level" | "place" | "date">): string {
  const year = a.date.slice(0, 4);
  if (a.kind === "rank" && a.rank) return `${FSP_RANK_LABEL[a.rank]} по ${FSP_DISCIPLINE_DATIVE[a.discipline]}, ${year}`;
  const place = a.place ? FSP_PLACE_LABEL[a.place] : "Участник";
  const level = a.event_level ? `${FSP_LEVEL_GENITIVE[a.event_level]} ` : "";
  return `${place} ${level}соревнований по ${FSP_DISCIPLINE_DATIVE[a.discipline]}, ${year}`;
}

export function isPrize(a: Pick<FspAchievement, "kind" | "place">): boolean {
  return a.kind === "rank" || a.place === "first" || a.place === "second" || a.place === "third";
}

export const SPEC_DESCRIPTION: Record<Specialization, string> = {
  backend: "Серверная логика, API, базы данных, очереди",
  frontend: "Интерфейсы, браузер, React и экосистема",
  devops: "CI/CD, контейнеры, Kubernetes, мониторинг",
  qa: "Тест-дизайн, автоматизация, тестирование API",
  ml: "Модели, метрики качества, работа с данными",
  security: "Веб-уязвимости, сети, криптография, SOC",
};

export const SELF_ASSESSMENT_LABEL: Record<number, string> = {
  1: "Не знаком",
  2: "Базово",
  3: "Уверенно",
  4: "Глубоко",
  5: "Эксперт",
};

export const ATTEMPT_KIND_LABEL: Record<AttemptKind, string> = {
  placement: "Первичный тест",
  placement_step_down: "Тест на грейд ниже",
  placement_upgrade: "Дотест на грейд выше",
  upgrade: "Повышение грейда",
  downgrade: "Смена грейда на ниже",
  retry: "Повторный тест",
};

export const ATTEMPT_KIND_HINT: Record<AttemptKind, string> = {
  placement: "Первичный тест. Не пройдёте — сразу можно на грейд ниже",
  placement_step_down: "Это ещё размещение: тест на грейд ниже доступен сразу",
  placement_upgrade: "Вы уверенно прошли — дотест на грейд выше, один раз. Текущий грейд не понизится",
  upgrade: "Повышение. Неудачная попытка не понизит текущий грейд",
  downgrade: "Смена на грейд ниже — только если вы сами этого хотите",
  retry: "Повторный тест после перерыва",
};

export const TRUST_LEVEL_LABEL: Record<TrustLevel, string> = {
  high: "Высокий уровень доверия",
  standard: "Обычный уровень доверия",
  new: "Новая компания",
  low: "Квота временно снижена",
};

export const TRUST_LEVEL_HINT: Record<TrustLevel, string> = {
  high: "Полная квота. Кандидаты откликаются на ваши предложения — так и держим.",
  standard: "Квота растёт с подтверждённым доменом, временем на платформе и принятыми приглашениями.",
  new: "Так начинает любая компания: квота вырастет сама, по мере работы на платформе.",
  low: "Есть замечания — они перестают учитываться через 90 дней. Ниже видно, какие именно и что с ними сделать.",
};
