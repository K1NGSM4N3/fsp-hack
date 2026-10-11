import type { SkillView, Specialization } from "@/api/types";

export interface SkillDef {
  slug: string;
  name: string;
  synonyms: string[];
  specs: Specialization[];
}

export const SKILLS: SkillDef[] = [
  { slug: "go", name: "Go", synonyms: ["golang", "го"], specs: ["backend", "devops"] },
  { slug: "python", name: "Python", synonyms: ["питон", "py"], specs: ["backend", "ml", "qa", "security"] },
  { slug: "java", name: "Java", synonyms: ["джава"], specs: ["backend", "qa"] },
  { slug: "kotlin", name: "Kotlin", synonyms: ["котлин"], specs: ["backend"] },
  { slug: "csharp", name: "C#", synonyms: [".net", "dotnet", "c sharp"], specs: ["backend"] },
  { slug: "nodejs", name: "Node.js", synonyms: ["node", "нода"], specs: ["backend", "frontend"] },
  { slug: "rust", name: "Rust", synonyms: ["раст"], specs: ["backend"] },
  { slug: "postgresql", name: "PostgreSQL", synonyms: ["postgres", "постгрес", "pg", "постгре"], specs: ["backend", "qa"] },
  { slug: "mysql", name: "MySQL", synonyms: ["мускул"], specs: ["backend"] },
  { slug: "clickhouse", name: "ClickHouse", synonyms: ["кликхаус", "ch"], specs: ["backend", "ml"] },
  { slug: "mongodb", name: "MongoDB", synonyms: ["mongo", "монго"], specs: ["backend"] },
  { slug: "redis", name: "Redis", synonyms: ["valkey", "редис"], specs: ["backend"] },
  { slug: "kafka", name: "Kafka", synonyms: ["кафка", "apache kafka"], specs: ["backend"] },
  { slug: "rabbitmq", name: "RabbitMQ", synonyms: ["rabbit", "кролик"], specs: ["backend"] },
  { slug: "grpc", name: "gRPC", synonyms: ["protobuf"], specs: ["backend"] },
  { slug: "rest", name: "REST API", synonyms: ["rest", "openapi"], specs: ["backend", "frontend", "qa"] },
  { slug: "microservices", name: "Микросервисы", synonyms: ["microservices", "микросервисная архитектура"], specs: ["backend"] },
  { slug: "sql", name: "SQL", synonyms: ["скуль"], specs: ["backend", "qa", "ml"] },
  { slug: "docker", name: "Docker", synonyms: ["докер", "контейнеры"], specs: ["backend", "devops", "qa"] },
  { slug: "kubernetes", name: "Kubernetes", synonyms: ["k8s", "кубер", "кубернетес"], specs: ["devops", "backend"] },
  { slug: "linux", name: "Linux", synonyms: ["линукс", "unix"], specs: ["devops", "security", "backend"] },
  { slug: "git", name: "Git", synonyms: ["гит"], specs: ["backend", "frontend", "devops", "qa", "ml"] },
  { slug: "django", name: "Django", synonyms: ["джанго"], specs: ["backend"] },
  { slug: "fastapi", name: "FastAPI", synonyms: ["fast api"], specs: ["backend"] },
  { slug: "spring", name: "Spring Boot", synonyms: ["spring", "спринг"], specs: ["backend"] },
  { slug: "typescript", name: "TypeScript", synonyms: ["ts", "тайпскрипт"], specs: ["frontend", "backend"] },
  { slug: "javascript", name: "JavaScript", synonyms: ["js", "джаваскрипт", "es6"], specs: ["frontend"] },
  { slug: "react", name: "React", synonyms: ["реакт", "react.js", "reactjs"], specs: ["frontend"] },
  { slug: "nextjs", name: "Next.js", synonyms: ["next", "некст"], specs: ["frontend"] },
  { slug: "vue", name: "Vue.js", synonyms: ["vue", "вью"], specs: ["frontend"] },
  { slug: "angular", name: "Angular", synonyms: ["ангуляр"], specs: ["frontend"] },
  { slug: "redux", name: "Redux", synonyms: ["redux toolkit", "rtk"], specs: ["frontend"] },
  { slug: "css", name: "CSS", synonyms: ["css3", "scss", "sass"], specs: ["frontend"] },
  { slug: "html", name: "HTML", synonyms: ["html5", "вёрстка", "верстка"], specs: ["frontend"] },
  { slug: "webpack", name: "Webpack", synonyms: ["вебпак"], specs: ["frontend"] },
  { slug: "graphql", name: "GraphQL", synonyms: ["гкл"], specs: ["frontend", "backend"] },
  { slug: "jest", name: "Jest", synonyms: [], specs: ["frontend"] },
  { slug: "playwright", name: "Playwright", synonyms: [], specs: ["frontend", "qa"] },
  { slug: "a11y", name: "Доступность (a11y)", synonyms: ["accessibility", "доступность", "wcag"], specs: ["frontend"] },
  { slug: "terraform", name: "Terraform", synonyms: ["терраформ", "iac"], specs: ["devops"] },
  { slug: "ansible", name: "Ansible", synonyms: ["ансибл"], specs: ["devops"] },
  { slug: "prometheus", name: "Prometheus", synonyms: ["прометей"], specs: ["devops"] },
  { slug: "grafana", name: "Grafana", synonyms: ["графана"], specs: ["devops"] },
  { slug: "helm", name: "Helm", synonyms: [], specs: ["devops"] },
  { slug: "ci_cd", name: "CI/CD", synonyms: ["ci/cd", "ci", "cd", "github actions", "jenkins"], specs: ["devops", "qa"] },
  { slug: "gitlab_ci", name: "GitLab CI", synonyms: ["gitlab"], specs: ["devops", "qa"] },
  { slug: "networking", name: "Сети (TCP/IP)", synonyms: ["tcp/ip", "сети", "networking"], specs: ["devops", "security"] },
  { slug: "system_design", name: "Проектирование систем", synonyms: ["system design", "архитектура"], specs: ["backend"] },
  { slug: "nginx", name: "Nginx", synonyms: ["энджинкс"], specs: ["devops"] },
  { slug: "yandex_cloud", name: "Yandex Cloud", synonyms: ["яндекс облако", "yc"], specs: ["devops"] },
  { slug: "bash", name: "Bash", synonyms: ["shell", "sh"], specs: ["devops", "security"] },
  { slug: "manual_testing", name: "Ручное тестирование", synonyms: ["manual", "ручное"], specs: ["qa"] },
  { slug: "test_design", name: "Тест-дизайн", synonyms: ["test design", "тест дизайн"], specs: ["qa"] },
  { slug: "selenium", name: "Selenium", synonyms: ["селениум"], specs: ["qa"] },
  { slug: "pytest", name: "pytest", synonyms: [], specs: ["qa", "backend"] },
  { slug: "postman", name: "Postman", synonyms: ["постман"], specs: ["qa"] },
  { slug: "api_testing", name: "Тестирование API", synonyms: ["api testing", "тестирование апи"], specs: ["qa"] },
  { slug: "load_testing", name: "Нагрузочное тестирование", synonyms: ["jmeter", "k6", "нагрузка"], specs: ["qa", "devops"] },
  { slug: "allure", name: "Allure", synonyms: [], specs: ["qa"] },
  { slug: "pytorch", name: "PyTorch", synonyms: ["torch", "пайторч"], specs: ["ml"] },
  { slug: "tensorflow", name: "TensorFlow", synonyms: ["tf", "keras"], specs: ["ml"] },
  { slug: "scikit_learn", name: "scikit-learn", synonyms: ["sklearn", "склерн"], specs: ["ml"] },
  { slug: "pandas", name: "pandas", synonyms: ["пандас"], specs: ["ml"] },
  { slug: "numpy", name: "NumPy", synonyms: ["нампай"], specs: ["ml"] },
  { slug: "nlp", name: "NLP", synonyms: ["обработка текста", "natural language processing"], specs: ["ml"] },
  { slug: "computer_vision", name: "Компьютерное зрение", synonyms: ["computer vision", "cv", "opencv"], specs: ["ml"] },
  { slug: "machine_learning", name: "Машинное обучение", synonyms: ["ml", "machine learning", "мл"], specs: ["ml"] },
  { slug: "ml_metrics", name: "Метрики качества ML", synonyms: ["метрики", "roc auc", "f1"], specs: ["ml"] },
  { slug: "llm", name: "LLM", synonyms: ["большие языковые модели", "gpt", "rag"], specs: ["ml"] },
  { slug: "mlops", name: "MLOps", synonyms: ["mlflow"], specs: ["ml", "devops"] },
  { slug: "spark", name: "Apache Spark", synonyms: ["spark", "pyspark", "спарк"], specs: ["ml"] },
  { slug: "airflow", name: "Airflow", synonyms: ["эйрфлоу"], specs: ["ml", "devops"] },
  { slug: "statistics", name: "Статистика", synonyms: ["statistics", "a/b тесты", "ab"], specs: ["ml"] },
  { slug: "catboost", name: "CatBoost", synonyms: ["кэтбуст", "градиентный бустинг", "xgboost"], specs: ["ml"] },
  { slug: "pentest", name: "Пентест", synonyms: ["pentest", "тестирование на проникновение"], specs: ["security"] },
  { slug: "owasp", name: "OWASP Top 10", synonyms: ["owasp"], specs: ["security", "backend"] },
  { slug: "appsec", name: "AppSec", synonyms: ["безопасная разработка", "sast", "dast"], specs: ["security"] },
  { slug: "siem", name: "SIEM", synonyms: ["soc", "мониторинг иб"], specs: ["security"] },
  { slug: "burp", name: "Burp Suite", synonyms: ["burp"], specs: ["security"] },
  { slug: "reverse", name: "Реверс-инжиниринг", synonyms: ["reverse", "реверс", "ida", "ghidra"], specs: ["security"] },
  { slug: "network_security", name: "Сетевая безопасность", synonyms: ["network security", "firewall"], specs: ["security"] },
  { slug: "cryptography", name: "Криптография", synonyms: ["crypto", "tls"], specs: ["security"] },
  { slug: "ctf", name: "CTF", synonyms: ["capture the flag"], specs: ["security"] },
];

export const SKILL_BY_SLUG = new Map(SKILLS.map((s) => [s.slug, s]));

const remoteNames = new Map<string, string>();

/** Names seen in API responses win over the bundled dictionary. */
export function rememberSkillNames(list: ReadonlyArray<{ slug: string; name: string }>) {
  for (const s of list) remoteNames.set(s.slug, s.name);
}

export function skillName(slug: string): string {
  const known = remoteNames.get(slug) ?? SKILL_BY_SLUG.get(slug)?.name;
  if (known) return known;
  const words = slug.replace(/[_-]+/g, " ").trim();
  return words.charAt(0).toUpperCase() + words.slice(1);
}

export function searchSkillsLocal(q: string, specialization?: Specialization, limit = 12): SkillView[] {
  const query = q.trim().toLowerCase();
  const pool = specialization && !query ? SKILLS.filter((s) => s.specs.includes(specialization)) : SKILLS;
  const ranked = pool.map((s) => {
    const name = s.name.toLowerCase();
    const rank = !query ? 0 : name.startsWith(query) ? 0 : name.includes(query) ? 1 : s.synonyms.some((x) => x.toLowerCase().startsWith(query)) ? 2 : -1;
    return { s, rank };
  })
    .filter((x) => x.rank >= 0)
    .sort((a, b) => a.rank - b.rank)
    .slice(0, limit);
  return ranked.map(({ s }) => ({ slug: s.slug, name: s.name, aliases: s.synonyms, group: "backend", group_name: "" }));
}
