import type {
  BankItem,
  CandidateCard,
  FeatureKey,
  FeatureScore,
  Grade,
  InvitationStatus,
  ProfileSkill,
  ScoreBreakdown,
  SkillMatch,
  SkillSource,
  Specialization,
  WorkFormat,
} from "@/api/types";
import { plural, pluralize } from "@/lib/format";
import { categoryLabel, FEATURE_LABEL, fspAchievementTitle, GRADE_LABEL, GRADES, SPEC_LONG_LABEL, strongestSource } from "@/lib/labels";
import { skillName } from "@/lib/skills";
import { proximity } from "./catalog";
import { achievementWeight, fspScore, fspScoreDto } from "./fsp";
import type { MockCandidate } from "./store";

const WEIGHTS: Record<FeatureKey, number> = {
  must: 0.3,
  test: 0.25,
  grade: 0.1,
  fsp: 0.1,
  nice: 0.08,
  salary: 0.07,
  fresh: 0.05,
  text: 0.05,
};

const SOURCE_CONFIDENCE: Record<SkillSource, number> = { test: 1, fsp: 1, resume: 0.7, declared: 0.5 };

export interface NeedLike {
  specialization: Specialization;
  grades: Grade[];
  must_skills: string[];
  nice_skills: string[];
  salary_max: number;
  description: string;
}

const DAY = 86400000;

export function currentAssignment(c: MockCandidate) {
  return c.assignments.at(-1) ?? null;
}

export function candidateSpec(c: MockCandidate): Specialization {
  return currentAssignment(c)?.category.specialization ?? c.surveys.at(-1)?.specialization ?? "backend";
}

export function skillDetails(c: MockCandidate): ProfileSkill[] {
  return Object.entries(c.skill_sources).map(([slug, sources]) => ({ slug, name: skillName(slug), sources }));
}

export function freshness(c: MockCandidate, now: Date): number {
  const days = Math.max(0, (now.getTime() - new Date(c.last_active_at).getTime()) / DAY);
  return Math.exp(-days / 60);
}

export function strength(c: MockCandidate, now: Date): number {
  const a = currentAssignment(c);
  const test = a ? a.strength : 0;
  const fsp = c.privacy.show_fsp ? fspScore(c.fsp.achievements, candidateSpec(c), now) : 0;
  return Math.round((0.6 * test + 0.25 * fsp + 0.15 * freshness(c, now)) * 1000) / 1000;
}

export interface RelationInfo {
  invitation_status: InvitationStatus | null;
  applied: boolean;
  contacts_available: boolean;
}

export function toCard(c: MockCandidate, now: Date, relation: RelationInfo): CandidateCard {
  const a = currentAssignment(c);
  const spec = candidateSpec(c);
  const sorted = [...c.fsp.achievements].sort((x, y) => achievementWeight(y, spec, now) - achievementWeight(x, spec, now));
  return {
    id: c.id,
    code: c.code,
    display_name: c.privacy.show_name ? c.profile.full_name : null,
    category: a?.category ?? null,
    category_name: a ? categoryLabel(a.category) : null,
    test_strength: a?.strength ?? null,
    category_assigned_at: a?.assigned_at ?? null,
    skills: skillDetails(c).map((s) => ({ ...s, verified: s.sources.includes("test") || s.sources.includes("fsp") })),
    fsp: c.privacy.show_fsp
      ? { score: fspScoreDto(c.fsp.achievements, spec, now), achievements: sorted.map((x) => ({ text: fspAchievementTitle(x), date: x.date, protocol_url: x.protocol_url ?? null })) }
      : null,
    experience_years: c.profile.experience_years ?? null,
    roles: c.profile.roles ?? [],
    soft_skills: c.profile.soft_skills ?? [],
    industries: c.profile.industries ?? [],
    experience: (c.profile.experience ?? []).map((e) => ({ ...e, description: e.description ?? "", company: c.privacy.show_experience ? e.company : null })),
    about: c.profile.about ?? "",
    city: c.profile.city ?? null,
    relocation: c.profile.relocation ?? false,
    work_formats: c.profile.work_formats ?? [],
    salary_expectation_min: c.profile.salary_expectation_min ?? null,
    search_status: c.privacy.search_status,
    last_activity_at: c.last_active_at,
    task_rating: null,
    tasks_done: 0,
    result_confirmed: false,
    minor: false,
    relation,
  };
}

function skillMatches(c: MockCandidate, wanted: string[], required: boolean): SkillMatch[] {
  return wanted.map((w) => {
    let best = 0;
    let similarity = 0;
    let via: { slug: string; source: SkillSource } | null = null;
    for (const [slug, sources] of Object.entries(c.skill_sources)) {
      const source = strongestSource(sources);
      const sim = proximity(w, slug);
      const v = sim * SOURCE_CONFIDENCE[source];
      if (v > best) {
        best = v;
        similarity = sim;
        via = { slug, source };
      }
    }
    return {
      skill: skillName(w),
      slug: w,
      required,
      matched_by: via ? skillName(via.slug) : null,
      source: via?.source ?? null,
      similarity: Math.round(similarity * 100) / 100,
      credit: Math.round(best * 100) / 100,
    };
  });
}

function mean(xs: number[]): number {
  return xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : 1;
}

function words(s: string): Set<string> {
  return new Set(
    s
      .toLowerCase()
      .split(/[^a-zа-яё0-9+#]+/i)
      .filter((w) => w.length > 3),
  );
}

export function explain(c: MockCandidate, need: NeedLike, now: Date): ScoreBreakdown {
  const a = currentAssignment(c);
  const reasons: string[] = [];
  const gaps: string[] = [];
  const features: FeatureScore[] = [];
  const add = (key: FeatureKey, value: number, note: string) => {
    const v = Math.max(0, Math.min(1, value));
    features.push({ key, label_ru: FEATURE_LABEL[key], value: Math.round(v * 100) / 100, weight: WEIGHTS[key], contribution: Math.round(WEIGHTS[key] * v * 1000) / 10, explanation_ru: note });
  };

  const must = skillMatches(c, need.must_skills, true);
  const nice = skillMatches(c, need.nice_skills, false);
  const n = need.must_skills.length;
  const hit = must.filter((m) => m.credit >= 0.5);
  const miss = must.filter((m) => m.credit < 0.5);
  const confirmed = must.filter((m) => (m.source === "test" || m.source === "fsp") && m.similarity >= 0.99);
  if (n) {
    const missNote = miss.length ? ` (нет ${miss.map((m) => m.skill).join(", ")})` : "";
    add("must", mean(must.map((m) => m.credit)), `${hit.length} из ${n} обязательных навыков${missNote}`);
    if (hit.length === n) reasons.push(n === 1 ? "Обязательный навык есть в профиле" : `Все ${n} ${plural(n, ["обязательный навык", "обязательных навыка", "обязательных навыков"])} в профиле`);
    else if (hit.length) reasons.push(`${hit.length} из ${n} обязательных навыков`);
    if (confirmed.length) reasons.push(`Подтверждены тестом или ФСП: ${confirmed.map((m) => m.skill).join(", ")}`);
    for (const m of miss) {
      gaps.push(m.credit >= 0.25 && m.matched_by && m.similarity < 0.99 ? `${m.skill} — только близкий навык ${m.matched_by}` : `Нет навыка ${m.skill}`);
    }
  } else {
    add("must", 1, "Обязательные навыки не заданы");
  }

  const pct = a ? Math.round(a.strength * 100) : 0;
  add("test", a?.strength ?? 0, a ? `Сила теста ${pct}/100 в категории ${categoryLabel(a.category)}` : "Нет результата теста");
  if (pct >= 70) reasons.push(`Сильный результат теста: ${pct}/100 в категории`);
  else if (a && pct < 30) gaps.push(`Результат теста ниже среднего в категории (${pct}/100)`);

  let gradeValue = 0;
  if (a) {
    if (need.grades.includes(a.category.grade)) gradeValue = 1;
    else gradeValue = need.grades.some((g) => GRADES.indexOf(g) === GRADES.indexOf(a.category.grade) - 1) ? 0.6 : 0.5;
  }
  const gradeName = a ? GRADE_LABEL[a.category.grade] : "";
  add("grade", gradeValue, gradeValue === 1 ? `Грейд ${gradeName} совпадает с запросом` : `Соседний грейд: ${gradeName}`);
  if (gradeValue < 1 && a) gaps.push(`Грейд ${gradeName} — соседний с запрошенным`);

  const fspVisible = c.privacy.show_fsp;
  const fsp = fspVisible ? fspScoreDto(c.fsp.achievements, need.specialization, now) : null;
  add("fsp", fsp?.score ?? 0, fsp?.has_history ? fsp.summary_ru : "Нет данных ФСП — на оценку не влияет отрицательно");
  if (fsp && fsp.score >= 0.2) reasons.push(fsp.summary_ru);

  const niceHit = nice.filter((m) => m.credit >= 0.5);
  add("nice", need.nice_skills.length ? mean(nice.map((m) => m.credit)) : 0, need.nice_skills.length ? `${niceHit.length} из ${need.nice_skills.length} желательных` : "Желательные навыки не заданы");
  if (niceHit.length >= 2) reasons.push(`Плюсом: ${niceHit.map((m) => m.skill).join(", ")}`);

  const exp = c.profile.salary_expectation_min ?? null;
  let salary = 0.8;
  let salaryNote = "Ожидания не указаны";
  if (exp !== null) {
    if (exp <= need.salary_max) {
      salary = 1;
      salaryNote = "Ожидания укладываются в вилку";
      reasons.push("Ожидания по зарплате укладываются в вилку");
    } else {
      const over = (exp - need.salary_max) / need.salary_max;
      salary = Math.max(0, 1 - over / 0.3);
      salaryNote = `Ожидания выше вилки на ${Math.round(over * 100)} %`;
      gaps.push(salaryNote);
    }
  }
  add("salary", salary, salaryNote);

  const fresh = freshness(c, now);
  const days = Math.round((now.getTime() - new Date(c.last_active_at).getTime()) / DAY);
  add("fresh", fresh, days <= 0 ? "Активность сегодня" : `Последняя активность ${pluralize(days, ["день", "дня", "дней"])} назад`);
  if (days > 90) gaps.push(`Давно не обновлял профиль (${pluralize(days, ["день", "дня", "дней"])})`);

  const needWords = words(need.description);
  const candWords = words(`${c.profile.about ?? ""} ${(c.profile.experience ?? []).map((e) => e.description).join(" ")}`);
  const overlap = [...needWords].filter((w) => candWords.has(w)).length;
  add("text", needWords.size ? Math.min(1, 0.25 + overlap / Math.max(4, needWords.size / 3)) : 0.5, overlap ? `Совпадений с описанием задачи: ${overlap}` : "Мало пересечений с описанием задачи");

  const total = Math.round(features.reduce((acc, f) => acc + f.contribution, 0) * 10) / 10;
  return { total, features, skills: [...must, ...nice], reasons: reasons.slice(0, 4), gaps: gaps.slice(0, 4) };
}

export function bankItem(card: CandidateCard, c: MockCandidate, now: Date, need: NeedLike | null): BankItem {
  if (!need) return { candidate: card, strength: strength(c, now), score: null, reasons: [], gaps: [] };
  const b = explain(c, need, now);
  return { candidate: card, strength: strength(c, now), score: b.total, reasons: b.reasons, gaps: b.gaps };
}

export function categoryTitle(spec: Specialization, grade: Grade): string {
  return `${SPEC_LONG_LABEL[spec]} · ${GRADE_LABEL[grade]}`;
}

export function formatCompatible(candidate: WorkFormat[], wanted: WorkFormat[], candidateCity: string | null, city: string | null, relocation: boolean): boolean {
  if (!wanted.length) return true;
  const common = wanted.filter((f) => candidate.includes(f));
  if (!common.length) return false;
  if (common.includes("remote")) return true;
  if (!city) return true;
  return candidateCity === city || relocation;
}
