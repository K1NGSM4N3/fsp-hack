import type { Catalog, FspDiscipline, Grade, SkillView } from "@/api/types";
import { FSP_DISCIPLINE_LABEL, INDUSTRIES, INDUSTRY_LABEL } from "@/lib/labels";

export const GRADE_THRESHOLD: Record<Grade, number> = { junior: -1, middle: 0, senior: 1 };

export const CATALOG: Catalog = {
  directions: [
    { slug: "development", name: "Разработка", specializations: [{ slug: "backend", name: "Бэкенд-разработка" }, { slug: "frontend", name: "Фронтенд-разработка" }] },
    { slug: "infrastructure", name: "Инфраструктура", specializations: [{ slug: "devops", name: "DevOps / SRE" }] },
    { slug: "quality", name: "Качество", specializations: [{ slug: "qa", name: "Тестирование (QA)" }] },
    { slug: "data_ai", name: "Данные и ИИ", specializations: [{ slug: "ml", name: "ML / Data Science" }] },
    { slug: "security", name: "Безопасность", specializations: [{ slug: "security", name: "Информационная безопасность" }] },
  ],
  grades: [
    { slug: "junior", name: "Junior" },
    { slug: "middle", name: "Middle" },
    { slug: "senior", name: "Senior" },
  ],
  work_formats: [
    { slug: "office", name: "Офис" },
    { slug: "hybrid", name: "Гибрид" },
    { slug: "remote", name: "Удалённо" },
  ],
  industries: INDUSTRIES.map((slug) => ({ slug, name: INDUSTRY_LABEL[slug] })),
  fsp_disciplines: (Object.keys(FSP_DISCIPLINE_LABEL) as FspDiscipline[]).map((slug) => ({ slug, name: FSP_DISCIPLINE_LABEL[slug] })),
};

const GROUPS: Array<[SkillView["group"], string, string[]]> = [
  ["language", "Языки", ["go", "python", "java", "kotlin", "csharp", "nodejs", "rust", "typescript", "javascript", "bash"]],
  ["database", "Базы данных", ["postgresql", "mysql", "clickhouse", "mongodb", "redis", "sql"]],
  ["messaging", "Очереди", ["kafka", "rabbitmq"]],
  ["api", "API", ["grpc", "rest", "graphql"]],
  ["frontend", "Фронтенд", ["react", "nextjs", "vue", "angular", "redux", "css", "html", "webpack", "a11y"]],
  ["infrastructure", "Инфраструктура", ["docker", "kubernetes", "linux", "terraform", "ansible", "prometheus", "grafana", "helm", "gitlab_ci", "nginx", "yandex_cloud"]],
  ["testing", "Тестирование", ["jest", "playwright", "manual_testing", "test_design", "selenium", "pytest", "postman", "api_testing", "load_testing", "allure"]],
  ["data_ml", "Данные и ML", ["pytorch", "tensorflow", "scikit_learn", "pandas", "numpy", "nlp", "computer_vision", "llm", "mlops", "spark", "airflow", "statistics", "catboost"]],
  ["security", "Безопасность", ["pentest", "owasp", "appsec", "siem", "burp", "reverse", "network_security", "cryptography", "ctf"]],
];

export function skillGroup(slug: string): { group: SkillView["group"]; group_name: string } {
  const g = GROUPS.find(([, , list]) => list.includes(slug));
  return g ? { group: g[0], group_name: g[1] } : { group: "backend", group_name: "Бэкенд" };
}

/** Skill proximity graph (README.md section 3): partial credit for a close substitute. */
const PROXIMITY: Array<[string, string, number]> = [
  ["postgresql", "mysql", 0.5],
  ["postgresql", "sql", 0.6],
  ["mysql", "sql", 0.6],
  ["clickhouse", "postgresql", 0.3],
  ["kafka", "rabbitmq", 0.6],
  ["redis", "mongodb", 0.2],
  ["go", "rust", 0.3],
  ["java", "kotlin", 0.8],
  ["kotlin", "spring", 0.4],
  ["java", "spring", 0.5],
  ["python", "fastapi", 0.4],
  ["python", "django", 0.4],
  ["django", "fastapi", 0.6],
  ["docker", "kubernetes", 0.4],
  ["kubernetes", "helm", 0.6],
  ["terraform", "ansible", 0.4],
  ["prometheus", "grafana", 0.6],
  ["gitlab_ci", "git", 0.3],
  ["react", "nextjs", 0.7],
  ["react", "vue", 0.4],
  ["vue", "angular", 0.4],
  ["react", "redux", 0.5],
  ["typescript", "javascript", 0.8],
  ["jest", "playwright", 0.3],
  ["selenium", "playwright", 0.6],
  ["postman", "api_testing", 0.7],
  ["pytest", "python", 0.4],
  ["pytorch", "tensorflow", 0.7],
  ["scikit_learn", "catboost", 0.5],
  ["pandas", "numpy", 0.6],
  ["spark", "airflow", 0.3],
  ["pentest", "burp", 0.6],
  ["appsec", "owasp", 0.7],
  ["siem", "network_security", 0.4],
  ["grpc", "rest", 0.4],
  ["microservices", "grpc", 0.3],
];

const PROX_MAP = new Map<string, number>();
for (const [a, b, w] of PROXIMITY) {
  PROX_MAP.set(`${a}|${b}`, w);
  PROX_MAP.set(`${b}|${a}`, w);
}

export function proximity(a: string, b: string): number {
  if (a === b) return 1;
  return PROX_MAP.get(`${a}|${b}`) ?? 0;
}
