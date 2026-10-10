import type { components } from "@/api/schema";
import type { FspAchievement, FspDiscipline, FspEventLevel, FspPlace, FspRank, FspScore, Specialization } from "@/api/types";
import { fspAchievementTitle } from "@/lib/labels";

/** Registry record as stored by `crates/fsp-mock`; the API strips the id. */
export type RawAchievement = components["schemas"]["Achievement"] & { id: string };

const LEVEL_W: Record<FspEventLevel, number> = { regional: 0.3, interregional: 0.5, national: 0.8, international: 1 };
const PLACE_W: Record<FspPlace, number> = { first: 1, second: 0.8, third: 0.7, finalist: 0.4, participant: 0.15 };
const RANK_W: Record<FspRank, number> = { ms: 1, cms: 0.8, first: 0.6, second: 0.4, third: 0.25, youth: 0.1 };

const RELEVANCE: Record<FspDiscipline, Record<Specialization, number>> = {
  algorithmic: { backend: 0.8, frontend: 0.6, devops: 0.5, qa: 0.5, ml: 0.8, security: 0.6 },
  product: { backend: 1, frontend: 1, devops: 0.6, qa: 0.7, ml: 0.6, security: 0.4 },
  security: { backend: 0.5, frontend: 0.3, devops: 0.6, qa: 0.4, ml: 0.3, security: 1 },
  ai: { backend: 0.5, frontend: 0.3, devops: 0.3, qa: 0.3, ml: 1, security: 0.3 },
  robotics: { backend: 0.4, frontend: 0.3, devops: 0.3, qa: 0.3, ml: 0.5, security: 0.3 },
  uav: { backend: 0.4, frontend: 0.3, devops: 0.3, qa: 0.3, ml: 0.5, security: 0.3 },
};

const HALF_LIFE_YEARS = 2;

/** architecture section 7.3: level × place (or rank) × relevance to the specialization × 2-year half-life decay. */
export function achievementWeight(a: RawAchievement, spec: Specialization, now: Date): number {
  const years = Math.max(0, (now.getTime() - new Date(a.date).getTime()) / (365.25 * 86400000));
  const decay = Math.pow(0.5, years / HALF_LIFE_YEARS);
  const base = a.kind === "rank" && a.rank ? RANK_W[a.rank] : (a.event_level ? LEVEL_W[a.event_level] : 0.3) * (a.place ? PLACE_W[a.place] : 0.15);
  return base * RELEVANCE[a.discipline][spec] * decay;
}

/** 1 − Π(1 − vᵢ): grows with every achievement, saturates at 1, so FSP never dominates the score. */
export function fspScore(achievements: RawAchievement[], spec: Specialization, now: Date): number {
  const rest = achievements.reduce((acc, a) => acc * (1 - achievementWeight(a, spec, now)), 1);
  return Math.round((1 - rest) * 1000) / 1000;
}

export function toView(a: RawAchievement): FspAchievement {
  const { id: _id, ...rest } = a;
  return { ...rest, text: fspAchievementTitle(a) };
}

export function fspScoreDto(achievements: RawAchievement[], spec: Specialization, now: Date): FspScore {
  const contributions = achievements
    .map((a, index) => ({ index, value: Math.round(achievementWeight(a, spec, now) * 1000) / 1000, explanation_ru: fspAchievementTitle(a) }))
    .sort((x, y) => y.value - x.value);
  const score = fspScore(achievements, spec, now);
  return {
    score,
    has_history: achievements.length > 0,
    contributions,
    summary_ru: achievements.length ? (contributions[0]?.explanation_ru ?? "") : "Нет данных ФСП",
  };
}

/** Subset of crates/fsp-mock/data/participants.json, so demo FSP IDs behave the same with and without the API. */
export const FSP_REGISTRY: Record<string, RawAchievement[]> = {
  "FSP-000001": [
    {
      "id": "ACH-000001-01",
      "kind": "rank",
      "discipline": "algorithmic",
      "rank": "first",
      "event_name": null,
      "event_level": null,
      "place": null,
      "team": null,
      "role": null,
      "date": "2021-06-10",
      "protocol_url": "https://registry.fsp-mock.example/protocols/orders/2021/algorithmic-first-2021-06-10.pdf"
    },
    {
      "id": "ACH-000001-02",
      "kind": "competition",
      "discipline": "algorithmic",
      "rank": null,
      "event_name": "Чемпионат Москвы по спортивному программированию (алгоритмическое программирование)",
      "event_level": "regional",
      "place": "first",
      "team": null,
      "role": null,
      "date": "2022-03-20",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2022/reg-algorithmic-2022-03-20.pdf"
    },
    {
      "id": "ACH-000001-03",
      "kind": "competition",
      "discipline": "algorithmic",
      "rank": null,
      "event_name": "Чемпионат России по спортивному программированию (алгоритмическое программирование)",
      "event_level": "national",
      "place": "second",
      "team": "Нулевой указатель",
      "role": "капитан команды",
      "date": "2022-05-15",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2022/chr-algorithmic-2022-05-15.pdf"
    },
    {
      "id": "ACH-000001-04",
      "kind": "rank",
      "discipline": "algorithmic",
      "rank": "cms",
      "event_name": null,
      "event_level": null,
      "place": null,
      "team": null,
      "role": null,
      "date": "2022-07-01",
      "protocol_url": "https://registry.fsp-mock.example/protocols/orders/2022/algorithmic-cms-2022-07-01.pdf"
    },
    {
      "id": "ACH-000001-05",
      "kind": "competition",
      "discipline": "algorithmic",
      "rank": null,
      "event_name": "Кубок России по спортивному программированию (алгоритмическое программирование)",
      "event_level": "national",
      "place": "first",
      "team": "Нулевой указатель",
      "role": "капитан команды",
      "date": "2023-10-08",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2023/cup-algorithmic-2023-10-08.pdf"
    },
    {
      "id": "ACH-000001-06",
      "kind": "competition",
      "discipline": "algorithmic",
      "rank": null,
      "event_name": "Международный кубок по спортивному программированию «Евразия» (алгоритмическое программирование)",
      "event_level": "international",
      "place": "third",
      "team": "Нулевой указатель",
      "role": "капитан команды",
      "date": "2024-08-21",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2024/eurasia-algorithmic-2024-08-21.pdf"
    },
    {
      "id": "ACH-000001-07",
      "kind": "rank",
      "discipline": "algorithmic",
      "rank": "ms",
      "event_name": null,
      "event_level": null,
      "place": null,
      "team": null,
      "role": null,
      "date": "2024-12-16",
      "protocol_url": "https://registry.fsp-mock.example/protocols/orders/2024/algorithmic-ms-2024-12-16.pdf"
    },
    {
      "id": "ACH-000001-08",
      "kind": "competition",
      "discipline": "algorithmic",
      "rank": null,
      "event_name": "Чемпионат России по спортивному программированию (алгоритмическое программирование)",
      "event_level": "national",
      "place": "first",
      "team": "Нулевой указатель",
      "role": "капитан команды",
      "date": "2025-05-18",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2025/chr-algorithmic-2025-05-18.pdf"
    },
    {
      "id": "ACH-000001-09",
      "kind": "competition",
      "discipline": "product",
      "rank": null,
      "event_name": "Чемпионат России по спортивному программированию (продуктовое программирование)",
      "event_level": "national",
      "place": "finalist",
      "team": "Нулевой указатель",
      "role": "бэкенд-разработчик",
      "date": "2025-11-23",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2025/chr-product-2025-11-23.pdf"
    }
  ],
  "FSP-000002": [
    {
      "id": "ACH-000002-01",
      "kind": "competition",
      "discipline": "product",
      "rank": null,
      "event_name": "Чемпионат Санкт-Петербурга по спортивному программированию (продуктовое программирование)",
      "event_level": "regional",
      "place": "third",
      "team": "Синтаксический сахар",
      "role": "фронтенд-разработчик",
      "date": "2022-04-17",
      "protocol_url": null
    },
    {
      "id": "ACH-000002-02",
      "kind": "competition",
      "discipline": "product",
      "rank": null,
      "event_name": "Чемпионат Северо-Западного федерального округа по спортивному программированию (продуктовое программирование)",
      "event_level": "interregional",
      "place": "first",
      "team": "Синтаксический сахар",
      "role": "капитан команды",
      "date": "2023-03-26",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2023/fo-product-2023-03-26.pdf"
    },
    {
      "id": "ACH-000002-03",
      "kind": "rank",
      "discipline": "product",
      "rank": "first",
      "event_name": null,
      "event_level": null,
      "place": null,
      "team": null,
      "role": null,
      "date": "2023-06-02",
      "protocol_url": "https://registry.fsp-mock.example/protocols/orders/2023/product-first-2023-06-02.pdf"
    },
    {
      "id": "ACH-000002-04",
      "kind": "competition",
      "discipline": "product",
      "rank": null,
      "event_name": "Чемпионат России по спортивному программированию (продуктовое программирование)",
      "event_level": "national",
      "place": "second",
      "team": "Синтаксический сахар",
      "role": "капитан команды",
      "date": "2023-11-19",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2023/chr-product-2023-11-19.pdf"
    },
    {
      "id": "ACH-000002-05",
      "kind": "competition",
      "discipline": "product",
      "rank": null,
      "event_name": "Кубок России по спортивному программированию (продуктовое программирование)",
      "event_level": "national",
      "place": "first",
      "team": "Синтаксический сахар",
      "role": "капитан команды",
      "date": "2024-10-13",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2024/cup-product-2024-10-13.pdf"
    },
    {
      "id": "ACH-000002-06",
      "kind": "rank",
      "discipline": "product",
      "rank": "cms",
      "event_name": null,
      "event_level": null,
      "place": null,
      "team": null,
      "role": null,
      "date": "2024-12-20",
      "protocol_url": "https://registry.fsp-mock.example/protocols/orders/2024/product-cms-2024-12-20.pdf"
    },
    {
      "id": "ACH-000002-07",
      "kind": "competition",
      "discipline": "product",
      "rank": null,
      "event_name": "Чемпионат России по спортивному программированию (продуктовое программирование)",
      "event_level": "national",
      "place": "finalist",
      "team": "Синтаксический сахар",
      "role": "капитан команды",
      "date": "2025-11-23",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2025/chr-product-2025-11-23.pdf"
    }
  ],
  "FSP-000003": [
    {
      "id": "ACH-000003-01",
      "kind": "competition",
      "discipline": "security",
      "rank": null,
      "event_name": "Чемпионат Новосибирской области по спортивному программированию (программирование систем информационной безопасности)",
      "event_level": "regional",
      "place": "first",
      "team": "КиберЩит",
      "role": "специалист по реверс-инжинирингу",
      "date": "2021-11-27",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2021/reg-security-2021-11-27.pdf"
    },
    {
      "id": "ACH-000003-02",
      "kind": "rank",
      "discipline": "security",
      "rank": "second",
      "event_name": null,
      "event_level": null,
      "place": null,
      "team": null,
      "role": null,
      "date": "2022-02-10",
      "protocol_url": "https://registry.fsp-mock.example/protocols/orders/2022/security-second-2022-02-10.pdf"
    },
    {
      "id": "ACH-000003-03",
      "kind": "competition",
      "discipline": "security",
      "rank": null,
      "event_name": "Чемпионат Сибирского федерального округа по спортивному программированию (программирование систем информационной безопасности)",
      "event_level": "interregional",
      "place": "second",
      "team": "КиберЩит",
      "role": "специалист по реверс-инжинирингу",
      "date": "2022-04-23",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2022/fo-security-2022-04-23.pdf"
    },
    {
      "id": "ACH-000003-04",
      "kind": "competition",
      "discipline": "security",
      "rank": null,
      "event_name": "Всероссийские соревнования по спортивному программированию (программирование систем информационной безопасности)",
      "event_level": "national",
      "place": "third",
      "team": "КиберЩит",
      "role": "капитан команды",
      "date": "2023-09-17",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2023/vs-security-2023-09-17.pdf"
    },
    {
      "id": "ACH-000003-05",
      "kind": "rank",
      "discipline": "security",
      "rank": "first",
      "event_name": null,
      "event_level": null,
      "place": null,
      "team": null,
      "role": null,
      "date": "2023-12-05",
      "protocol_url": "https://registry.fsp-mock.example/protocols/orders/2023/security-first-2023-12-05.pdf"
    },
    {
      "id": "ACH-000003-06",
      "kind": "competition",
      "discipline": "security",
      "rank": null,
      "event_name": "Чемпионат России по спортивному программированию (программирование систем информационной безопасности)",
      "event_level": "national",
      "place": "first",
      "team": "КиберЩит",
      "role": "капитан команды",
      "date": "2024-11-10",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2024/chr-security-2024-11-10.pdf"
    },
    {
      "id": "ACH-000003-07",
      "kind": "rank",
      "discipline": "security",
      "rank": "cms",
      "event_name": null,
      "event_level": null,
      "place": null,
      "team": null,
      "role": null,
      "date": "2025-02-14",
      "protocol_url": "https://registry.fsp-mock.example/protocols/orders/2025/security-cms-2025-02-14.pdf"
    },
    {
      "id": "ACH-000003-08",
      "kind": "competition",
      "discipline": "security",
      "rank": null,
      "event_name": "Международный кубок по спортивному программированию «Евразия» (программирование систем информационной безопасности)",
      "event_level": "international",
      "place": "participant",
      "team": "КиберЩит",
      "role": "капитан команды",
      "date": "2025-08-30",
      "protocol_url": null
    }
  ],
  "FSP-000006": [
    {
      "id": "ACH-000006-01",
      "kind": "competition",
      "discipline": "product",
      "rank": null,
      "event_name": "Чемпионат Нижегородской области по спортивному программированию (продуктовое программирование)",
      "event_level": "regional",
      "place": "participant",
      "team": "Тихий деплой",
      "role": "аналитик",
      "date": "2024-03-16",
      "protocol_url": null
    },
    {
      "id": "ACH-000006-02",
      "kind": "competition",
      "discipline": "product",
      "rank": null,
      "event_name": "Чемпионат Приволжского федерального округа по спортивному программированию (продуктовое программирование)",
      "event_level": "interregional",
      "place": "third",
      "team": "Тихий деплой",
      "role": "продакт-менеджер",
      "date": "2025-04-12",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2025/fo-product-2025-04-12.pdf"
    },
    {
      "id": "ACH-000006-03",
      "kind": "competition",
      "discipline": "product",
      "rank": null,
      "event_name": "Чемпионат России по спортивному программированию (продуктовое программирование)",
      "event_level": "national",
      "place": "participant",
      "team": "Тихий деплой",
      "role": "продакт-менеджер",
      "date": "2025-11-22",
      "protocol_url": "https://registry.fsp-mock.example/protocols/2025/chr-product-2025-11-22.pdf"
    }
  ],
  "FSP-000010": []
};

export const DEMO_FSP_IDS = [
  { id: "FSP-000001", note: "Смирнов Алексей — 9 достижений" },
  { id: "FSP-000002", note: "Кузнецова Мария — 7 достижений" },
  { id: "FSP-000003", note: "Иванов Дмитрий — 8 достижений" },
  { id: "FSP-000006", note: "Лебедева Екатерина — 3 достижения" },
  { id: "FSP-000010", note: "Волкова Дарья — без достижений" },
];
