import { ApiError, type FieldErrors } from "@/api/client";
import type * as T from "@/api/types";
import { formatDate } from "@/lib/format";
import { GRADE_LABEL, GRADES, SPEC_DIRECTION, SPEC_TOPICS, SPECIALIZATIONS, TOPIC_LABEL } from "@/lib/labels";
import { SKILL_BY_SLUG, SKILLS, searchSkillsLocal, skillName } from "@/lib/skills";
import { checkAnswer, MOCK_ITEMS_MAX, MOCK_ITEMS_MIN, posteriorStats, priorFor, probAbove, selectNext, updatePosterior } from "./assessment";
import { CATALOG, GRADE_THRESHOLD, skillGroup } from "./catalog";
import { FSP_REGISTRY, fspScoreDto, toView } from "./fsp";
import { newId, seedFromString } from "./rng";
import { bankItem, candidateSpec, categoryTitle, currentAssignment, explain, formatCompatible, skillDetails, toCard, type NeedLike, type RelationInfo } from "./scoring";
import { CONSENT_DOCS } from "./seed";
import {
  getDb,
  persist,
  resetDb,
  type MockApplication,
  type MockAssessment,
  type MockCandidate,
  type MockCompany,
  type MockInvitation,
  type MockItem,
  type MockNeed,
  type MockShortlist,
  type MockTask,
  type MockTaskAssignment,
  type MockUser,
  type MockVacancy,
} from "./store";

const DAY = 86400000;
const INVITATION_QUOTA = 30;
const QUOTA_PERIOD_DAYS = 7;
const INVITATION_TTL_DAYS = 14;
const GRADE_CHANGE_COOLDOWN_DAYS = 90;
const RETRY_DAYS = 30;
const SHORTLIST_SIZE = 20;
const GRACE_MS = 5000;
const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

const now = () => new Date();
const nowIso = () => new Date().toISOString();
const addDays = (iso: string | Date, days: number) => new Date(new Date(iso).getTime() + days * DAY);

/** Index cursors: opaque to the pages, like the API's keyset ones. */
export function pageOf<X>(all: X[], cursor: string | null | undefined, limit: number): { items: X[]; next_cursor: string | null } {
  const start = Math.max(0, Number(cursor ?? 0) || 0);
  const end = start + Math.max(1, limit);
  return { items: all.slice(start, end), next_cursor: end < all.length ? String(end) : null };
}

function statusCounts(list: { status: T.InvitationStatus }[]): T.InvitationStatusCount[] {
  const counts = new Map<T.InvitationStatus, number>();
  for (const i of list) counts.set(i.status, (counts.get(i.status) ?? 0) + 1);
  return [...counts].map(([status, count]) => ({ status, count }));
}

function delay(min = 120, max = 380): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, min + Math.random() * (max - min)));
}

function fail(status: number, code: string, title: string, detail?: string, errors?: FieldErrors): never {
  throw new ApiError({ status, code, title, detail: detail ?? null, errors: errors ?? null });
}

function validation(errors: FieldErrors): never {
  fail(422, "validation", "Проверьте введённые данные", undefined, errors);
}

function clone<V>(value: V): V {
  return structuredClone(value);
}

function setSessionCookie(on: boolean) {
  if (typeof document === "undefined") return;
  document.cookie = on ? "fsp_session=1; path=/; max-age=2592000; samesite=lax" : "fsp_session=; path=/; max-age=0; samesite=lax";
}

function sessionUser(): MockUser {
  const db = getDb();
  const user = db.session ? db.users.find((u) => u.id === db.session?.user_id) : undefined;
  if (!user) fail(401, "unauthorized", "Нужно войти в аккаунт");
  return user;
}

function requireCandidate(): { user: MockUser; cand: MockCandidate } {
  const user = sessionUser();
  if (user.role !== "candidate") fail(403, "forbidden", "Раздел доступен только кандидатам");
  const cand = getDb().candidates.find((c) => c.id === user.candidate_id);
  if (!cand) fail(404, "not_found", "Профиль не найден");
  return { user, cand };
}

function requireEmployer(): { user: MockUser; company: MockCompany } {
  const user = sessionUser();
  if (user.role !== "employer") fail(403, "forbidden", "Раздел доступен только работодателям");
  const company = getDb().companies.find((c) => c.id === user.company_id);
  if (!company) fail(404, "not_found", "Компания не найдена");
  return { user, company };
}

function companyOf(id: string): MockCompany | undefined {
  return getDb().companies.find((c) => c.id === id);
}

function refreshInvitationStatuses() {
  const t = now();
  for (const inv of getDb().invitations) {
    if ((inv.status === "sent" || inv.status === "viewed") && new Date(inv.expires_at) < t) {
      inv.status = "expired";
      inv.events.push({ status: "expired", at: inv.expires_at });
    }
  }
}

function expireStaleSessions() {
  const t = now().getTime();
  for (const a of getDb().assessments) {
    if (a.session.status === "in_progress" && t - new Date(a.last_activity_at).getTime() > 60 * 60 * 1000) {
      a.session.status = "expired";
      a.session.finished_at = a.last_activity_at;
    }
  }
}

function searchable(c: MockCandidate): boolean {
  return c.privacy.visible && c.privacy.search_status !== "closed" && c.assignments.length > 0;
}

function median(xs: number[]): number | null {
  if (!xs.length) return null;
  const s = [...xs].sort((a, b) => a - b);
  const mid = Math.floor(s.length / 2);
  return s.length % 2 ? (s[mid] as number) : Math.round(((s[mid - 1] as number) + (s[mid] as number)) / 2);
}

function salaryErrors(salary: T.SalaryRange | undefined, errors: FieldErrors) {
  if (!salary || !(salary.min > 0)) errors["salary.min"] = ["Укажите зарплату «от»"];
  if (!salary || !(salary.max > 0)) errors["salary.max"] = ["Укажите зарплату «до»"];
  else if (salary.max < salary.min) errors["salary.max"] = ["«До» не может быть меньше «от»"];
}

function relationOf(companyId: string | null, candidateId: string): RelationInfo {
  if (!companyId) return { invitation_status: null, applied: false, contacts_available: false };
  const db = getDb();
  const invitations = db.invitations.filter((i) => i.company_id === companyId && i.candidate_id === candidateId).sort((a, b) => b.created_at.localeCompare(a.created_at));
  const vacancyIds = new Set(db.vacancies.filter((v) => v.company_id === companyId).map((v) => v.id));
  const applied = db.applications.some((a) => a.candidate_id === candidateId && vacancyIds.has(a.vacancy_id) && a.status !== "withdrawn");
  const accepted = invitations.some((i) => i.status === "accepted");
  return { invitation_status: invitations[0]?.status ?? null, applied, contacts_available: accepted || applied };
}

const lastVerification = new Map<string, string>();

/** Mock-only "mailbox": the verification link the real API would email. */
export function mockVerificationLink(email: string): string | null {
  const key = email.trim().toLowerCase();
  const token = lastVerification.get(key) ?? getDb().users.find((u) => u.email === key)?.verify_token;
  return token ? `/verify-email?token=${token}` : null;
}

export function mockReset(): void {
  resetDb();
  setSessionCookie(false);
}

export async function getCatalog(): Promise<T.Catalog> {
  await delay(60, 160);
  return clone(CATALOG);
}

export async function searchSkills(q: string, specialization?: T.Specialization): Promise<T.SkillView[]> {
  await delay(40, 120);
  return searchSkillsLocal(q, specialization).map((s) => ({ ...s, ...skillGroup(s.slug) }));
}

export async function getCurrentConsents(): Promise<T.ConsentDocument[]> {
  await delay(60, 160);
  return clone(CONSENT_DOCS);
}

export async function register(body: T.RegisterRequest): Promise<T.RegisterResponse> {
  await delay(250, 500);
  const db = getDb();
  const errors: FieldErrors = {};
  const email = body.email.trim().toLowerCase();
  if (!EMAIL_RE.test(email)) errors.email = ["Введите корректный email"];
  if (body.password.length < 8) errors.password = ["Минимум 8 символов"];
  if (body.role === "employer" && !body.company_name?.trim()) errors.company_name = ["Укажите название компании"];
  const required = CONSENT_DOCS.filter((d) => d.required_for.includes(body.role));
  if (required.some((d) => !body.consents.includes(d.id))) errors.consents = ["Нужно принять обязательные согласия"];
  if (Object.keys(errors).length) validation(errors);
  if (db.users.some((u) => u.email === email)) fail(409, "email_taken", "Этот email уже зарегистрирован", "Войдите в аккаунт");

  const userId = newId();
  const token = newId().replace(/-/g, "");
  const created = nowIso();
  let candidateId: string | null = null;
  let companyId: string | null = null;
  if (body.role === "candidate") {
    candidateId = newId();
    db.candidates.push({
      id: candidateId,
      user_id: userId,
      code: `K-${userId.replace(/-/g, "").slice(-6).toUpperCase()}`,
      profile: {
        full_name: "",
        contacts: { email, phone: null, telegram: null },
        city: null,
        relocation: false,
        work_formats: ["remote"],
        salary_expectation_min: null,
        about: "",
        experience_years: null,
        roles: [],
        soft_skills: [],
        industries: [],
        experience: [],
        skills: [],
      },
      skill_sources: {},
      profile_saved_at: null,
      privacy: { visible: true, show_name: false, show_fsp: true, show_experience: true, search_status: "active" },
      surveys: [],
      assignments: [],
      placement: { open: true, offer: null, offer_grade: null },
      retry_after: {},
      no_category_retry_after: null,
      fsp: { fsp_id: null, linked_at: null, last_sync_at: null, achievements: [] },
      last_active_at: created,
    });
  } else {
    companyId = newId();
    db.companies.push({
      id: companyId,
      owner_user_id: userId,
      name: body.company_name?.trim() ?? "",
      description: "",
      industry: null,
      website: null,
      city: null,
      size: null,
      contact_email: email,
      contact_phone: null,
      contact_telegram: null,
      updated_at: created,
    });
  }
  db.users.push({
    id: userId,
    email,
    password: body.password,
    role: body.role === "employer" ? "employer" : "candidate",
    email_verified: false,
    verify_token: token,
    candidate_id: candidateId,
    company_id: companyId,
    created_at: created,
    consents: CONSENT_DOCS.filter((d) => body.consents.includes(d.id)).map((d) => ({
      id: newId(),
      document_id: d.id,
      kind: d.kind,
      version: d.version,
      title: d.title,
      granted_at: created,
      revoked_at: null,
    })),
  });
  lastVerification.set(email, token);
  persist();
  return { user_id: userId, verification_required: true };
}

export async function verifyEmail(token: string): Promise<void> {
  await delay(200, 400);
  const user = getDb().users.find((u) => u.verify_token === token);
  if (!user) fail(400, "invalid_token", "Ссылка недействительна или устарела", "Запросите письмо повторно");
  user.email_verified = true;
  user.verify_token = null;
  persist();
}

export async function resendVerification(email: string): Promise<void> {
  await delay(200, 400);
  const user = getDb().users.find((u) => u.email === email.trim().toLowerCase());
  if (user && !user.email_verified) {
    user.verify_token = newId().replace(/-/g, "");
    lastVerification.set(user.email, user.verify_token);
    persist();
  }
}

function sessionDto(u: MockUser): T.SessionUser {
  return { id: u.id, role: u.role, email: u.email, email_verified: u.email_verified };
}

export async function login(body: T.LoginRequest): Promise<T.LoginResponse> {
  await delay(250, 450);
  const db = getDb();
  const user = db.users.find((u) => u.email === body.email.trim().toLowerCase() && u.password === body.password);
  if (!user) fail(401, "invalid_credentials", "Неверный email или пароль");
  if (!user.email_verified) fail(403, "email_not_verified", "Email не подтверждён", "Перейдите по ссылке из письма или запросите новое");
  db.session = { user_id: user.id };
  setSessionCookie(true);
  persist();
  return { access_token: "mock-access-token", expires_in: 900, user: sessionDto(user) };
}

export async function logout(): Promise<void> {
  await delay(80, 160);
  getDb().session = null;
  setSessionCookie(false);
  persist();
}

function onboardingStep(c: MockCandidate): T.OnboardingStep {
  if (!c.profile_saved_at) return "profile";
  if (!c.surveys.length) return "survey";
  if (!c.assignments.length) return "assessment";
  return "done";
}

export async function getMe(): Promise<T.Me> {
  await delay(60, 140);
  const user = sessionUser();
  const db = getDb();
  if (user.role === "candidate") {
    const c = db.candidates.find((x) => x.id === user.candidate_id);
    const active = db.assessments.find((a) => a.candidate_id === c?.id && a.session.status === "in_progress");
    return {
      ...sessionDto(user),
      candidate: c
        ? { public_code: c.code, onboarding_step: onboardingStep(c), category: currentAssignment(c)?.category ?? null, active_session_id: active?.session.id ?? null, age: { bracket: "adult", adult_from: null, guardian: null } }
        : null,
      company: null,
    };
  }
  const company = db.companies.find((x) => x.id === user.company_id);
  return { ...sessionDto(user), candidate: null, company: company ? { id: company.id, name: company.name } : null };
}

export async function deleteMe(): Promise<void> {
  await delay(300, 500);
  const user = sessionUser();
  const db = getDb();
  db.users = db.users.filter((u) => u.id !== user.id);
  if (user.candidate_id) {
    const id = user.candidate_id;
    db.candidates = db.candidates.filter((c) => c.id !== id);
    db.applications = db.applications.filter((a) => a.candidate_id !== id);
    db.task_assignments = db.task_assignments.filter((s) => s.candidate_id !== id);
    db.assessments = db.assessments.filter((a) => a.candidate_id !== id);
  }
  db.session = null;
  setSessionCookie(false);
  persist();
}

export async function getMyConsents(): Promise<T.UserConsent[]> {
  await delay();
  return clone(sessionUser().consents);
}

export async function grantConsent(documentId: T.Uuid): Promise<T.UserConsent> {
  await delay();
  const user = sessionUser();
  const doc = CONSENT_DOCS.find((d) => d.id === documentId);
  if (!doc || !doc.applies_to.includes(user.role)) validation({ document_id: ["Этот документ не относится к вашему аккаунту"] });
  const active = user.consents.find((c) => c.document_id === documentId && !c.revoked_at);
  if (active) return clone(active);
  const consent: T.UserConsent = { id: newId(), document_id: doc.id, kind: doc.kind, version: doc.version, title: doc.title, granted_at: nowIso(), revoked_at: null };
  user.consents.unshift(consent);
  persist();
  return clone(consent);
}

export async function getContactAccess(): Promise<T.ContactAccessEntry[]> {
  await delay();
  const { cand } = requireCandidate();
  const db = getDb();
  return db.invitations
    .filter((i) => i.candidate_id === cand.id && i.status === "accepted")
    .map((i) => ({
      company_id: i.company_id,
      company_name: db.companies.find((c) => c.id === i.company_id)?.name ?? null,
      basis: "invitation" as const,
      channel: "cabinet" as const,
      accessed_at: i.responded_at ?? i.created_at,
    }));
}

export async function exportMyData(): Promise<Blob> {
  await delay();
  const user = sessionUser();
  const db = getDb();
  const data = {
    format: "fsp-talents-export/1",
    generated_at: nowIso(),
    account: { id: user.id, role: user.role, email: user.email },
    consents: user.consents,
    candidate: db.candidates.find((c) => c.id === user.candidate_id) ?? null,
  };
  return new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
}

export async function revokeConsent(documentId: T.Uuid): Promise<void> {
  await delay();
  const user = sessionUser();
  const consent = user.consents.find((c) => c.document_id === documentId && !c.revoked_at);
  if (!consent) fail(404, "not_found", "Согласие не найдено");
  consent.revoked_at = nowIso();
  if ((consent.kind === "pd_publication" || consent.kind === "pd_processing") && user.candidate_id) {
    const c = getDb().candidates.find((x) => x.id === user.candidate_id);
    if (c) c.privacy.visible = false;
  }
  persist();
}

function profileDto(c: MockCandidate): T.Profile {
  const a = currentAssignment(c);
  return {
    ...clone(c.profile),
    public_code: c.code,
    skill_details: skillDetails(c),
    category: a?.category ?? null,
    category_assigned_at: a?.assigned_at ?? null,
    updated_at: c.profile_saved_at ?? c.last_active_at,
  };
}

/** Demo accounts are adults: the representative's flow exists only against the real API. */
export async function getGuardianStatus(): Promise<T.GuardianStatus> {
  await delay(40, 90);
  return { required: false, adult_from: null, state: null, guardian_name: null, guardian_email: null, sent_at: null, link_expires_at: null, answered_at: null, revoked_at: null };
}

export async function resendGuardianLetter(): Promise<T.GuardianStatus> {
  await delay(60, 120);
  fail(409, "guardian_not_needed", "Вам уже есть 18 — согласие представителя не нужно");
}

export async function changeGuardian(body: T.GuardianContact): Promise<T.GuardianStatus> {
  void body;
  return resendGuardianLetter();
}

export async function getGuardianConsent(token: string): Promise<T.GuardianConsentView> {
  void token;
  await delay(60, 120);
  fail(400, "invalid_token", "Ссылка недействительна", "В демо-режиме писем представителю нет");
}

export async function answerGuardianConsent(token: string, action: T.GuardianAction): Promise<T.GuardianConsentView> {
  void action;
  return getGuardianConsent(token);
}

export async function getProfile(): Promise<T.Profile> {
  await delay();
  return profileDto(requireCandidate().cand);
}

export async function updateProfile(body: T.ProfileInput): Promise<T.Profile> {
  await delay(250, 450);
  const { cand } = requireCandidate();
  const errors: FieldErrors = {};
  if (!body.full_name.trim()) errors.full_name = ["Укажите имя"];
  if (body.contacts.email && !EMAIL_RE.test(body.contacts.email)) errors["contacts.email"] = ["Некорректный email"];
  if (!body.contacts.email && !body.contacts.phone && !body.contacts.telegram) errors["contacts.email"] = ["Нужен хотя бы один контакт"];
  if ((body.salary_expectation_min ?? 0) < 0) errors.salary_expectation_min = ["Не может быть отрицательной"];
  const years = body.experience_years ?? null;
  if (years !== null && (years < 0 || years > 60)) errors.experience_years = ["От 0 до 60 лет"];
  if (Object.keys(errors).length) validation(errors);
  const sources: Record<string, T.SkillSource[]> = {};
  for (const slug of body.skills ?? []) sources[slug] = cand.skill_sources[slug] ?? ["declared"];
  for (const [slug, src] of Object.entries(cand.skill_sources)) {
    if (!sources[slug] && src.some((s) => s === "test" || s === "fsp")) sources[slug] = src.filter((s) => s === "test" || s === "fsp");
  }
  cand.profile = clone(body);
  cand.skill_sources = sources;
  cand.profile_saved_at = nowIso();
  cand.last_active_at = nowIso();
  persist();
  return profileDto(cand);
}

export async function uploadResume(file: File): Promise<T.ResumeDraft> {
  const { user, cand } = requireCandidate();
  const isPdf = file.type === "application/pdf" || file.name.toLowerCase().endsWith(".pdf");
  if (!isPdf) fail(415, "unsupported_media_type", "Нужен файл в формате PDF");
  if (file.size > 5 * 1024 * 1024) fail(413, "payload_too_large", "Файл больше 5 МБ");
  await delay(900, 1400);
  const spec = candidateSpec(cand);
  const stacks: Record<T.Specialization, string[]> = {
    backend: ["go", "postgresql", "docker", "kafka", "redis", "grpc"],
    frontend: ["typescript", "react", "nextjs", "css", "jest"],
    devops: ["kubernetes", "terraform", "prometheus", "gitlab_ci", "linux"],
    qa: ["python", "pytest", "api_testing", "postman", "sql"],
    ml: ["python", "pandas", "pytorch", "scikit_learn", "sql"],
    security: ["pentest", "burp", "owasp", "linux", "python"],
  };
  const roles: Record<T.Specialization, string> = {
    backend: "Backend-разработчик",
    frontend: "Frontend-разработчик",
    devops: "DevOps-инженер",
    qa: "QA-инженер",
    ml: "ML-инженер",
    security: "Специалист по ИБ",
  };
  const parsed: T.ParsedResume = {
    full_name: cand.profile.full_name || "Дмитрий Орлов",
    email: user.email,
    phone: "+7 915 123-45-67",
    telegram: null,
    city: "Москва",
    relocation: null,
    skills: stacks[spec],
    specialization_hint: { specialization: spec, confidence: 0.82 },
    grade_hint: "middle",
    experience_years: 3.5,
    roles: [roles[spec]],
    soft_skills: ["Работа в команде", "Ответственность"],
    industries: ["fintech"],
    work_formats: ["remote", "hybrid"],
    salary_expectation: 250000,
  };
  const recognized = Object.entries(parsed)
    .filter(([, v]) => v !== null && v !== undefined && !(Array.isArray(v) && v.length === 0))
    .map(([k]) => k);
  return { parsed, recognized_fields: recognized };
}

export async function getProfilePdf(): Promise<Blob> {
  await delay();
  fail(501, "not_available_in_mock", "PDF формирует сервер", "В демо-режиме откройте версию для печати");
}

export async function getPrivacy(): Promise<T.Privacy> {
  await delay();
  return clone(requireCandidate().cand.privacy);
}

export async function updatePrivacy(body: T.Privacy): Promise<T.Privacy> {
  await delay();
  const { user, cand } = requireCandidate();
  if (body.visible) {
    const missing = CONSENT_DOCS.filter((d) => d.required_for.includes("candidate") && !user.consents.some((c) => c.kind === d.kind && !c.revoked_at)).sort((a, b) => a.kind.localeCompare(b.kind))[0];
    if (missing)
      throw new ApiError({
        status: 409,
        code: "consent_required",
        title: "Нужно согласие",
        detail: `«${missing.title}»: без него профиль не показывается работодателям`,
        documentId: missing.id,
        documentKind: missing.kind,
      });
  }
  cand.privacy = clone(body);
  persist();
  return clone(cand.privacy);
}

export async function submitSurvey(body: T.SurveyInput): Promise<T.SurveyResponse> {
  await delay(250, 450);
  const { cand } = requireCandidate();
  const errors: FieldErrors = {};
  if (!SPECIALIZATIONS.includes(body.specialization)) errors.specialization = ["Выберите специализацию"];
  else if (SPEC_DIRECTION[body.specialization] !== body.direction) errors.specialization = ["Специализация не из выбранного направления"];
  if (Object.values(body.self_assessment ?? {}).some((x) => x < 1 || x > 5)) errors.self_assessment = ["Оценки от 1 до 5"];
  if (Object.keys(errors).length) validation(errors);
  const survey: T.SurveyResponse = {
    id: newId(),
    created_at: nowIso(),
    direction: body.direction,
    specialization: body.specialization,
    claimed_grade: body.claimed_grade,
    skills: body.skills ?? [],
    industries: body.industries ?? [],
    experience_years: body.experience_years ?? null,
    self_assessment: body.self_assessment ?? {},
  };
  cand.surveys.push(survey);
  const profileSkills = cand.profile.skills ?? [];
  for (const slug of survey.skills) {
    const src = cand.skill_sources[slug] ?? [];
    if (!src.includes("declared")) cand.skill_sources[slug] = [...src, "declared"];
    if (!profileSkills.includes(slug)) profileSkills.push(slug);
  }
  cand.profile.skills = profileSkills;
  cand.profile.industries = [...new Set([...(cand.profile.industries ?? []), ...survey.industries])];
  if (cand.profile.experience_years === null && survey.experience_years !== null) cand.profile.experience_years = survey.experience_years;
  cand.last_active_at = nowIso();
  persist();
  return clone(survey);
}

export async function getSurveyHistory(): Promise<T.SurveyResponse[]> {
  await delay();
  return clone([...requireCandidate().cand.surveys].reverse());
}

const KIND_RU: Record<T.AttemptKind, string> = {
  placement: "Первичный тест",
  placement_step_down: "Тест на грейд ниже",
  placement_upgrade: "Дотест на грейд выше",
  upgrade: "Повышение грейда",
  downgrade: "Смена грейда на ниже",
  retry: "Повторный тест",
};

/** Grade policy (architecture §5.4): which attempts are allowed right now. */
function attemptFor(c: MockCandidate, g: T.Grade, t: Date): T.AttemptKind | null {
  const current = currentAssignment(c);
  if (!current) {
    if (c.no_category_retry_after) return new Date(c.no_category_retry_after) <= t ? "retry" : null;
    if (c.placement.offer === "step_down") return g === c.placement.offer_grade ? "placement_step_down" : null;
    return "placement";
  }
  if (g === current.category.grade) return null;
  if (c.placement.open && c.placement.offer === "upgrade" && g === c.placement.offer_grade) return "placement_upgrade";
  if (addDays(current.assigned_at, GRADE_CHANGE_COOLDOWN_DAYS) > t) return null;
  const retry = c.retry_after[g];
  if (retry && new Date(retry) > t) return null;
  return GRADES.indexOf(g) > GRADES.indexOf(current.category.grade) ? "upgrade" : "downgrade";
}

function offersFor(c: MockCandidate): T.Offer[] {
  if (!c.surveys.length && !c.assignments.length) return [];
  const t = now();
  return GRADES.flatMap((grade) => {
    const kind = attemptFor(c, grade, t);
    return kind ? [{ grade, kind, label: `${KIND_RU[kind]}: ${GRADE_LABEL[grade]}`, ordered_retest: false, camera_required: false }] : [];
  });
}

function nextChangeAt(c: MockCandidate): string | null {
  const t = now();
  const current = currentAssignment(c);
  if (!current) return c.no_category_retry_after && new Date(c.no_category_retry_after) > t ? c.no_category_retry_after : null;
  const next = addDays(current.assigned_at, GRADE_CHANGE_COOLDOWN_DAYS);
  return next > t ? next.toISOString() : null;
}

const REASON_RU: Record<"placement" | "upgrade" | "downgrade", string> = { placement: "Первичное размещение", upgrade: "Повышение", downgrade: "Смена на грейд ниже" };

export async function getCandidateCategory(): Promise<T.CategoryStatus> {
  await delay();
  const { cand } = requireCandidate();
  const current = currentAssignment(cand);
  const survey = cand.surveys.at(-1) ?? null;
  return {
    category: current?.category ?? null,
    category_name: current ? categoryTitle(current.category.specialization, current.category.grade) : null,
    theta: current?.theta ?? null,
    test_strength: current?.strength ?? null,
    assigned_at: current?.assigned_at ?? null,
    specialization: current?.category.specialization ?? survey?.specialization ?? null,
    claimed_grade_in_survey: survey?.claimed_grade ?? null,
    available: offersFor(cand),
    locked: [],
    next_change_at: nextChangeAt(cand),
    history: [...cand.assignments].reverse().map((a) => ({
      specialization: a.category.specialization,
      grade: a.category.grade,
      reason: REASON_RU[a.reason],
      theta: a.theta,
      session_id: a.session_id,
      assigned_at: a.assigned_at,
    })),
  };
}

function sessionDtoOf(m: MockAssessment): T.AssessmentSession {
  const current = m.session.status === "in_progress" ? m.items.find((i) => !i.answered) : undefined;
  return {
    ...clone(m.session),
    answered: m.items.filter((i) => i.answered).length,
    current: current
      ? {
          seq: current.seq,
          deadline: current.deadline_at,
          item: {
            template_id: current.template_id,
            level: current.level,
            title: current.title,
            topic: current.topic,
            kind: current.kind,
            prompt: current.prompt,
            code: current.code,
            options: [...current.options],
            input_hint: current.input_hint,
            time_limit_secs: current.time_limit_secs,
          },
        }
      : null,
  };
}

function nextItem(m: MockAssessment): MockItem | null {
  const { theta } = posteriorStats(m.posterior);
  const picked = selectNext(
    m.session.specialization,
    theta,
    m.items.map((i) => ({ template_id: i.template_id, topic: i.topic })),
    seedFromString(`${m.session.id}:${m.items.length}`),
  );
  if (!picked) return null;
  const shown = now();
  return {
    seq: m.items.length,
    template_id: picked.template_id,
    level: picked.level,
    topic: picked.topic,
    title: picked.title,
    kind: picked.item.kind,
    prompt: picked.item.prompt,
    code: picked.item.code,
    options: picked.item.options,
    input_hint: picked.item.input_hint,
    key: picked.item.key,
    a: picked.a,
    b: picked.b,
    c: picked.c,
    time_limit_secs: picked.item.time_limit_secs,
    shown_at: shown.toISOString(),
    deadline_at: new Date(shown.getTime() + picked.item.time_limit_secs * 1000).toISOString(),
    answered: false,
    answer: null,
    correct: null,
    answered_at: null,
  };
}

export async function startAssessment(body: T.StartInput): Promise<T.AssessmentSession> {
  await delay(250, 450);
  const { cand } = requireCandidate();
  const db = getDb();
  expireStaleSessions();
  if (db.assessments.some((a) => a.candidate_id === cand.id && a.session.status === "in_progress")) {
    fail(409, "assessment_in_progress", "Сначала завершите начатый тест");
  }
  if (!cand.surveys.length) fail(409, "survey_required", "Сначала пройдите опрос");
  const kind = attemptFor(cand, body.grade, now());
  if (!kind) {
    const next = nextChangeAt(cand);
    if (next) fail(409, "grade_change_cooldown", "Смена грейда пока недоступна", `Следующая попытка — после ${formatDate(next)}`);
    fail(409, "attempt_not_allowed", "Такая попытка сейчас недоступна");
  }
  const m: MockAssessment = {
    session: {
      id: newId(),
      specialization: body.specialization,
      claimed_grade: body.grade,
      attempt_kind: kind,
      status: "in_progress",
      started_at: nowIso(),
      finished_at: null,
      min_items: MOCK_ITEMS_MIN,
      max_items: MOCK_ITEMS_MAX,
      recording_views: [],
      result: null,
    },
    candidate_id: cand.id,
    items: [],
    posterior: priorFor(body.grade),
    last_activity_at: nowIso(),
  };
  const first = nextItem(m);
  if (first) m.items.push(first);
  db.assessments.push(m);
  persist();
  return sessionDtoOf(m);
}

export async function listAssessments(): Promise<T.AssessmentSession[]> {
  await delay();
  const { cand } = requireCandidate();
  expireStaleSessions();
  return getDb()
    .assessments.filter((a) => a.candidate_id === cand.id)
    .sort((a, b) => (b.session.started_at ?? "").localeCompare(a.session.started_at ?? ""))
    .map(sessionDtoOf);
}

function findSession(id: string): { m: MockAssessment; cand: MockCandidate } {
  const { cand } = requireCandidate();
  expireStaleSessions();
  const m = getDb().assessments.find((a) => a.session.id === id && a.candidate_id === cand.id);
  if (!m) fail(404, "not_found", "Тест не найден");
  return { m, cand };
}

export async function getAssessment(id: T.Uuid): Promise<T.AssessmentSession> {
  await delay(80, 200);
  return sessionDtoOf(findSession(id).m);
}

function strengthFor(grade: T.Grade, theta: number): number {
  const lo = GRADE_THRESHOLD[grade];
  const hi = grade === "senior" ? lo + 1.5 : GRADE_THRESHOLD[GRADES[GRADES.indexOf(grade) + 1] as T.Grade];
  return Math.max(0.03, Math.min(0.99, Math.round(((theta - lo) / (hi - lo)) * 100) / 100));
}

function shouldStop(m: MockAssessment): boolean {
  const answered = m.items.filter((i) => i.answered).length;
  if (answered >= m.session.max_items) return true;
  if (answered < m.session.min_items) return false;
  const p = probAbove(m.posterior, GRADE_THRESHOLD[m.session.claimed_grade]);
  return p > 0.975 || p < 0.025;
}

function finishSession(m: MockAssessment, cand: MockCandidate) {
  const t = now();
  const { theta, se } = posteriorStats(m.posterior);
  const grade = m.session.claimed_grade;
  const idx = GRADES.indexOf(grade);
  const higher = GRADES[idx + 1] as T.Grade | undefined;
  const lower = GRADES[idx - 1] as T.Grade | undefined;
  const passed = theta >= GRADE_THRESHOLD[grade];
  const confidentAboveNext = Boolean(higher && probAbove(m.posterior, GRADE_THRESHOLD[higher]) >= 0.9);
  const kind = m.session.attempt_kind;
  let assigned = false;

  if (passed) {
    cand.assignments.push({
      id: newId(),
      category: { specialization: m.session.specialization, grade },
      theta,
      strength: strengthFor(grade, theta),
      reason: kind === "upgrade" || kind === "placement_upgrade" ? "upgrade" : kind === "downgrade" ? "downgrade" : "placement",
      session_id: m.session.id,
      assigned_at: t.toISOString(),
    });
    assigned = true;
    cand.no_category_retry_after = null;
    if (kind === "placement" && higher && confidentAboveNext) cand.placement = { open: true, offer: "upgrade", offer_grade: higher };
    else cand.placement = { open: false, offer: null, offer_grade: null };
    const topicSkills = (cand.surveys.at(-1)?.skills ?? []).filter((s) => cand.skill_sources[s] && !cand.skill_sources[s]?.includes("test")).slice(0, 2);
    for (const s of topicSkills) cand.skill_sources[s] = ["test", ...(cand.skill_sources[s] ?? [])];
  } else if (kind === "placement" || kind === "placement_step_down" || kind === "retry") {
    if (lower) cand.placement = { open: true, offer: "step_down", offer_grade: lower };
    else {
      cand.no_category_retry_after = addDays(t, RETRY_DAYS).toISOString();
      cand.placement = { open: true, offer: null, offer_grade: null };
    }
  } else {
    cand.retry_after[grade] = addDays(t, RETRY_DAYS).toISOString();
    if (kind === "placement_upgrade") cand.placement = { open: false, offer: null, offer_grade: null };
  }

  const answered = m.items.filter((i) => i.answered);
  const byTopic = new Map<T.AssessmentTopic, { answered: number; correct: number }>();
  for (const i of answered) {
    const r = byTopic.get(i.topic) ?? { answered: 0, correct: 0 };
    r.answered++;
    if (i.correct) r.correct++;
    byTopic.set(i.topic, r);
  }
  const hardRight = answered.filter((i) => i.correct && i.b > theta + 1).length;
  const easyWrong = answered.filter((i) => !i.correct && i.b < theta - 1).length;
  const aberrant = hardRight >= 2 && easyWrong >= 2;
  const measured = [...GRADES].reverse().find((g) => theta >= GRADE_THRESHOLD[g]) ?? null;

  m.session.status = passed ? "passed" : "failed";
  m.session.finished_at = t.toISOString();
  m.session.result = {
    decision: {
      passed,
      theta,
      se,
      grade_measured: measured,
      confident_above_next: confidentAboveNext,
      person_fit_lz: aberrant ? -2.4 : 0.3,
      aberrant,
      items_administered: answered.length,
    },
    test_strength: passed ? strengthFor(grade, theta) : null,
    assigned_grade: assigned ? grade : null,
    needs_review: aberrant,
    topics: [...byTopic.entries()].map(([topic, r]) => ({ topic, topic_ru: TOPIC_LABEL[topic], ...r })),
    offers: offersFor(cand),
    locked: [],
  };
  cand.last_active_at = t.toISOString();
}

function record(m: MockAssessment, cand: MockCandidate, item: MockItem, answer: T.Answer | null, onTime: boolean) {
  item.answered = true;
  item.answer = answer;
  item.answered_at = nowIso();
  item.correct = onTime && checkAnswer(item.key, answer);
  m.posterior = updatePosterior(m.posterior, item.correct, item.a, item.b, item.c);
  m.last_activity_at = nowIso();
  if (shouldStop(m)) {
    finishSession(m, cand);
    return;
  }
  const next = nextItem(m);
  if (next) m.items.push(next);
  else finishSession(m, cand);
}

export async function getCurrentItem(id: T.Uuid): Promise<T.AssessmentSession> {
  await delay(120, 300);
  const { m, cand } = findSession(id);
  const current = m.items.find((i) => !i.answered);
  if (m.session.status === "in_progress" && current && now().getTime() > new Date(current.deadline_at).getTime() + GRACE_MS) {
    record(m, cand, current, null, false);
    persist();
  }
  return sessionDtoOf(m);
}

export async function answerItem(id: T.Uuid, body: T.AnswerInput): Promise<T.AssessmentSession> {
  await delay(150, 300);
  const { m, cand } = findSession(id);
  if (m.session.status !== "in_progress") fail(409, "assessment_finished", "Тест уже завершён");
  const current = m.items.find((i) => !i.answered);
  if (!current || current.seq !== body.seq) fail(409, "stale_item", "Это задание уже засчитано, обновите страницу");
  const onTime = now().getTime() <= new Date(current.deadline_at).getTime() + GRACE_MS;
  record(m, cand, current, body.answer, onTime);
  persist();
  return sessionDtoOf(m);
}

function receivedDto(inv: MockInvitation): T.ReceivedInvitation {
  const c = companyOf(inv.company_id);
  return {
    id: inv.id,
    company_id: inv.company_id,
    company_name: c?.name ?? "Компания",
    company_description: c?.description ?? "",
    company_industry: c?.industry ?? null,
    company_city: c?.city ?? null,
    company_website: c?.website ?? null,
    vacancy_id: inv.vacancy_id,
    title: inv.title,
    message: inv.message,
    salary: clone(inv.salary),
    work_format: inv.work_format,
    contact: inv.contact,
    status: inv.status,
    decline_reason: inv.decline_reason,
    created_at: inv.created_at,
    viewed_at: inv.viewed_at,
    responded_at: inv.responded_at,
    expires_at: inv.expires_at,
    outcomes: [],
  };
}

function ownInvitation(id: string): { inv: MockInvitation; cand: MockCandidate } {
  const { cand } = requireCandidate();
  refreshInvitationStatuses();
  const inv = getDb().invitations.find((i) => i.id === id && i.candidate_id === cand.id);
  if (!inv) fail(404, "not_found", "Приглашение не найдено");
  return { inv, cand };
}

export async function listReceivedInvitations(query: T.InvitationListQuery = {}): Promise<T.ReceivedInvitationPage> {
  await delay();
  const { cand } = requireCandidate();
  refreshInvitationStatuses();
  persist();
  const mine = getDb()
    .invitations.filter((i) => i.candidate_id === cand.id)
    .sort((a, b) => b.created_at.localeCompare(a.created_at));
  const wanted = mine.filter((i) => !query.status?.length || query.status.includes(i.status));
  const page = pageOf(wanted, query.cursor, query.limit ?? 30);
  return { items: page.items.map(receivedDto), next_cursor: page.next_cursor, by_status: statusCounts(mine) };
}

export async function getReceivedInvitation(id: T.Uuid): Promise<T.ReceivedInvitation> {
  await delay();
  const { inv } = ownInvitation(id);
  if (inv.status === "sent") {
    inv.status = "viewed";
    inv.viewed_at = nowIso();
    inv.events.push({ status: "viewed", at: inv.viewed_at });
    persist();
  }
  return receivedDto(inv);
}

function assertActive(inv: MockInvitation) {
  if (inv.status !== "sent" && inv.status !== "viewed") fail(409, "invitation_closed", "Приглашение уже неактивно", "Его приняли, отклонили, отозвали или истёк срок");
}

export async function acceptInvitation(id: T.Uuid): Promise<T.ReceivedInvitation> {
  await delay(250, 450);
  const { inv, cand } = ownInvitation(id);
  assertActive(inv);
  const at = nowIso();
  if (!inv.viewed_at) {
    inv.viewed_at = at;
    inv.events.push({ status: "viewed", at });
  }
  inv.status = "accepted";
  inv.responded_at = at;
  inv.events.push({ status: "accepted", at });
  cand.last_active_at = at;
  persist();
  return receivedDto(inv);
}

export async function declineInvitation(id: T.Uuid, body: T.DeclineRequest): Promise<T.ReceivedInvitation> {
  await delay(250, 450);
  const { inv, cand } = ownInvitation(id);
  assertActive(inv);
  const at = nowIso();
  inv.status = "declined";
  inv.responded_at = at;
  inv.decline_reason = body.reason;
  inv.decline_comment = body.comment?.trim() || null;
  inv.events.push({ status: "declined", at });
  if (body.reason === "not_looking") cand.privacy.search_status = "closed";
  cand.last_active_at = at;
  persist();
  return receivedDto(inv);
}

function inCategory(spec: T.Specialization, grade: T.Grade) {
  return getDb().candidates.filter((c) => {
    const a = currentAssignment(c);
    return searchable(c) && a?.category.specialization === spec && a.category.grade === grade;
  });
}

export async function getCandidateMarket(): Promise<T.MarketStats> {
  await delay();
  const { cand } = requireCandidate();
  const a = currentAssignment(cand);
  if (!a) fail(409, "no_category", "Категория ещё не присвоена", "Статистика спроса появится после теста");
  const { specialization: spec, grade } = a.category;
  const db = getDb();
  const t = now().getTime();
  const ids = new Set(inCategory(spec, grade).map((c) => c.id));
  const inv30 = db.invitations.filter((i) => ids.has(i.candidate_id) && t - new Date(i.created_at).getTime() < 30 * DAY);
  const needs = db.needs.filter((n) => n.status === "active" && n.specialization === spec && n.grades.includes(grade));
  const vacs = db.vacancies.filter((v) => v.status === "published" && v.specialization === spec && v.grade === grade);
  const viewed = db.invitations.filter((i) => i.candidate_id === cand.id && t - new Date(i.created_at).getTime() < 30 * DAY);
  return {
    category: a.category,
    companies_viewed_you_30d: new Set(viewed.map((i) => i.company_id)).size + 2 + (seedFromString(cand.id) % 4),
    invitations_in_category_30d: inv30.length,
    open_needs: needs.length + vacs.length,
    peers_in_category: ids.size,
    median_offer_min: median([...inv30.map((i) => i.salary.min), ...vacs.map((v) => v.salary.min)]),
    median_offer_max: median([...inv30.map((i) => i.salary.max), ...vacs.map((v) => v.salary.max)]),
  };
}

function myApplicationDto(a: MockApplication): T.MyApplication | null {
  const v = getDb().vacancies.find((x) => x.id === a.vacancy_id);
  if (!v) return null;
  return {
    id: a.id,
    vacancy_id: v.id,
    vacancy_title: v.title,
    company_name: companyOf(v.company_id)?.name ?? "Компания",
    salary: clone(v.salary),
    cover_letter: a.cover_letter,
    status: a.status,
    created_at: a.created_at,
    updated_at: a.updated_at,
    outcomes: [],
  };
}

export async function listMyApplications(): Promise<T.MyApplication[]> {
  await delay();
  const { cand } = requireCandidate();
  return getDb()
    .applications.filter((a) => a.candidate_id === cand.id)
    .sort((a, b) => b.created_at.localeCompare(a.created_at))
    .map(myApplicationDto)
    .filter((x): x is T.MyApplication => x !== null);
}

export async function withdrawApplication(id: T.Uuid): Promise<void> {
  await delay();
  const { cand } = requireCandidate();
  const a = getDb().applications.find((x) => x.id === id && x.candidate_id === cand.id);
  if (!a) fail(404, "not_found", "Отклик не найден");
  if (a.status !== "sent" && a.status !== "viewed") fail(409, "application_closed", "По отклику уже принято решение");
  a.status = "withdrawn";
  a.updated_at = nowIso();
  persist();
}

function myTaskDto(asg: MockTaskAssignment): T.MyTask | null {
  const task = getDb().tasks.find((t) => t.id === asg.task_id);
  if (!task) return null;
  return {
    id: asg.id,
    title: task.title,
    body: task.body,
    kind: task.kind,
    skills: [...task.skills],
    company_name: companyOf(task.company_id)?.name ?? null,
    status: asg.status,
    answer: asg.answer,
    rating: asg.rating,
    review_comment: asg.review_comment,
    assigned_at: asg.assigned_at,
    due_at: asg.due_at,
    submitted_at: asg.submitted_at,
    reviewed_at: asg.reviewed_at,
  };
}

/** Same lazy rule as the API: hand out a new task only when nothing is open and the last one is a week old. */
function assignIfDue(cand: MockCandidate) {
  const db = getDb();
  const t = now();
  const mine = db.task_assignments.filter((a) => a.candidate_id === cand.id);
  for (const a of mine) if (a.status === "assigned" && new Date(a.due_at) < t) a.status = "expired";
  if (mine.some((a) => a.status === "assigned")) return;
  const last = mine.map((a) => a.assigned_at).sort().at(-1);
  if (last && t.getTime() - new Date(last).getTime() < 7 * DAY) return;
  const cat = currentAssignment(cand)?.category;
  if (!cat) return;
  const seen = new Set(mine.map((a) => a.task_id));
  const task = db.tasks.find((x) => x.active && !seen.has(x.id) && x.specialization === cat.specialization && x.grades.includes(cat.grade));
  if (!task) return;
  db.task_assignments.push({
    id: newId(),
    task_id: task.id,
    candidate_id: cand.id,
    status: "assigned",
    answer: null,
    rating: null,
    review_comment: null,
    assigned_at: t.toISOString(),
    due_at: addDays(t, 7).toISOString(),
    submitted_at: null,
    reviewed_at: null,
  });
}

function loadMyTasks(cand: MockCandidate): T.MyTask[] {
  return getDb()
    .task_assignments.filter((a) => a.candidate_id === cand.id)
    .sort((a, b) => b.assigned_at.localeCompare(a.assigned_at))
    .map(myTaskDto)
    .filter((x): x is T.MyTask => x !== null);
}

export async function listMyTasks(): Promise<T.MyTask[]> {
  await delay();
  const { cand } = requireCandidate();
  assignIfDue(cand);
  persist();
  return loadMyTasks(cand);
}

export async function submitTask(id: T.Uuid, body: T.SubmitInput): Promise<T.MyTask[]> {
  await delay(300, 500);
  const { cand } = requireCandidate();
  const answer = body.answer.trim();
  if (!answer || answer.length > 20000) validation({ answer: ["Ответ от 1 до 20 000 символов"] });
  const asg = getDb().task_assignments.find((a) => a.id === id && a.candidate_id === cand.id);
  if (!asg || asg.status !== "assigned" || new Date(asg.due_at) < now()) fail(409, "task_closed", "Задание уже закрыто или просрочено");
  asg.status = "submitted";
  asg.answer = answer;
  asg.submitted_at = nowIso();
  cand.last_active_at = nowIso();
  persist();
  return loadMyTasks(cand);
}

export async function skipTask(id: T.Uuid): Promise<void> {
  await delay();
  const { cand } = requireCandidate();
  const asg = getDb().task_assignments.find((a) => a.id === id && a.candidate_id === cand.id);
  if (!asg || asg.status !== "assigned") fail(409, "task_closed", "Задание уже закрыто");
  asg.status = "skipped";
  persist();
}

function fspStatusDto(c: MockCandidate): T.FspStatus {
  return {
    linked: Boolean(c.fsp.fsp_id),
    fsp_id: c.fsp.fsp_id,
    linked_at: c.fsp.linked_at,
    last_sync_at: c.fsp.last_sync_at,
    achievements: c.fsp.achievements.map(toView),
    score: c.fsp.fsp_id ? fspScoreDto(c.fsp.achievements, candidateSpec(c), now()) : null,
  };
}

function syncFrom(c: MockCandidate, fspId: string) {
  c.fsp.fsp_id = fspId;
  c.fsp.last_sync_at = nowIso();
  c.fsp.linked_at ??= c.fsp.last_sync_at;
  c.fsp.achievements = clone(FSP_REGISTRY[fspId] ?? []);
}

export async function fspLinkDev(fspId: string): Promise<T.FspStatus> {
  await delay(500, 900);
  const { cand } = requireCandidate();
  const id = fspId.trim().toUpperCase();
  if (!id || id.length > 40) validation({ fsp_id: ["Укажите ФСП ID"] });
  if (!(id in FSP_REGISTRY)) fail(404, "fsp_participant_not_found", "Участник ФСП не найден", "Проверьте ФСП ID, например FSP-000001");
  if (getDb().candidates.some((c) => c.id !== cand.id && c.fsp.fsp_id === id)) fail(409, "fsp_id_taken", "Этот ФСП ID уже привязан к другому профилю");
  cand.fsp.linked_at = nowIso();
  syncFrom(cand, id);
  persist();
  return fspStatusDto(cand);
}

export async function getFspStatus(): Promise<T.FspStatus> {
  await delay();
  return fspStatusDto(requireCandidate().cand);
}

export async function syncFsp(): Promise<T.FspStatus> {
  await delay(500, 900);
  const { cand } = requireCandidate();
  if (!cand.fsp.fsp_id) fail(404, "not_found", "ФСП ID не привязан");
  syncFrom(cand, cand.fsp.fsp_id);
  persist();
  return fspStatusDto(cand);
}

export async function unlinkFsp(): Promise<void> {
  await delay();
  const { cand } = requireCandidate();
  cand.fsp = { fsp_id: null, linked_at: null, last_sync_at: null, achievements: [] };
  for (const [slug, src] of Object.entries(cand.skill_sources)) {
    const rest = src.filter((s) => s !== "fsp");
    if (rest.length) cand.skill_sources[slug] = rest;
    else delete cand.skill_sources[slug];
  }
  persist();
}

function companyDto(c: MockCompany): T.Company {
  const { owner_user_id: _owner, ...rest } = c;
  return clone(rest);
}

export async function getCompany(): Promise<T.Company> {
  await delay();
  return companyDto(requireEmployer().company);
}

export async function updateCompany(body: T.CompanyProfile): Promise<T.Company> {
  await delay(250, 450);
  const { company } = requireEmployer();
  const errors: FieldErrors = {};
  if (!body.name.trim()) errors.name = ["Укажите название"];
  if (body.contact_email && !EMAIL_RE.test(body.contact_email)) errors.contact_email = ["Некорректный email"];
  if (Object.keys(errors).length) validation(errors);
  Object.assign(company, clone(body), { updated_at: nowIso() });
  persist();
  return companyDto(company);
}

function needDto(n: MockNeed): T.HiringNeed {
  const db = getDb();
  const sl = db.shortlists[n.id];
  const { company_id: _company, ...rest } = n;
  return {
    ...clone(rest),
    shortlisted: sl ? Object.values(sl.entries).filter((e) => !e.dismissed).length : 0,
    invited: db.invitations.filter((i) => i.need_id === n.id).length,
  };
}

function needFields(body: T.NeedInput) {
  const errors: FieldErrors = {};
  if (!body.title.trim()) errors.title = ["Укажите название"];
  if (!body.grades.length) errors.grades = ["Выберите хотя бы один грейд"];
  salaryErrors(body.salary, errors);
  if (Object.keys(errors).length) validation(errors);
  return {
    title: body.title.trim(),
    description: body.description ?? "",
    specialization: body.specialization,
    grades: body.grades,
    allow_adjacent_grades: body.allow_adjacent_grades ?? false,
    must_skills: body.must_skills ?? [],
    nice_skills: body.nice_skills ?? [],
    salary: clone(body.salary),
    work_formats: body.work_formats ?? [],
    city: body.city ?? null,
    industries: body.industries ?? [],
    status: body.status ?? "active",
  };
}

function ownNeed(id: string): { need: MockNeed; company: MockCompany } {
  const { company } = requireEmployer();
  const need = getDb().needs.find((n) => n.id === id && n.company_id === company.id);
  if (!need) fail(404, "not_found", "Потребность не найдена");
  return { need, company };
}

export async function listNeeds(): Promise<T.HiringNeed[]> {
  await delay();
  const { company } = requireEmployer();
  return getDb()
    .needs.filter((n) => n.company_id === company.id)
    .sort((a, b) => b.updated_at.localeCompare(a.updated_at))
    .map(needDto);
}

export async function createNeed(body: T.NeedInput): Promise<T.HiringNeed> {
  await delay(300, 500);
  const { company } = requireEmployer();
  const need: MockNeed = { ...needFields(body), id: newId(), company_id: company.id, filters: {}, shortlist_generated_at: null, created_at: nowIso(), updated_at: nowIso() };
  getDb().needs.push(need);
  persist();
  return needDto(need);
}

export async function getNeed(id: T.Uuid): Promise<T.HiringNeed> {
  await delay();
  return needDto(ownNeed(id).need);
}

export async function updateNeed(id: T.Uuid, body: T.NeedInput): Promise<T.HiringNeed> {
  await delay(250, 450);
  const { need } = ownNeed(id);
  Object.assign(need, needFields(body), { updated_at: nowIso() });
  persist();
  return needDto(need);
}

export async function deleteNeed(id: T.Uuid): Promise<void> {
  await delay();
  ownNeed(id);
  const db = getDb();
  db.needs = db.needs.filter((n) => n.id !== id);
  delete db.shortlists[id];
  persist();
}

const SPEC_KEYWORDS: Record<T.Specialization, RegExp> = {
  backend: /бэкенд|бекенд|backend|back-end|серверн|микросервис|\bapi\b|golang|\bgo\b|java\b|python/gi,
  frontend: /фронтенд|frontend|front-end|react|vue|angular|вёрстк|верстк|интерфейс/gi,
  devops: /devops|\bsre\b|kubernetes|k8s|инфраструктур|ci\/cd|terraform/gi,
  qa: /тестировщик|\bqa\b|автотест|тестирован/gi,
  ml: /\bml\b|machine learning|машинн|data scien|нейросет|\bllm\b|\bnlp\b|компьютерн/gi,
  security: /\bиб\b|безопасност|pentest|пентест|\bsoc\b|siem|appsec/gi,
};

const CITY_STEMS: Array<[string, RegExp]> = [
  ["Москва", /москв|\bмск\b/i],
  ["Санкт-Петербург", /петербург|\bспб\b|питер/i],
  ["Казань", /казан/i],
  ["Новосибирск", /новосибирск/i],
  ["Екатеринбург", /екатеринбург|\bекб\b/i],
  ["Нижний Новгород", /нижн\S* новгород/i],
  ["Томск", /томск/i],
  ["Иннополис", /иннополис/i],
  ["Краснодар", /краснодар/i],
];

export async function parseNeed(text: string): Promise<T.ParsedNeed> {
  await delay(500, 900);
  requireEmployer();
  const lower = text.toLowerCase();
  const scores = SPECIALIZATIONS.map((s) => ({ s, n: (lower.match(SPEC_KEYWORDS[s]) ?? []).length })).sort((a, b) => b.n - a.n);
  const total = scores.reduce((a, x) => a + x.n, 0);
  const top = scores[0];

  const grades: T.Grade[] = [];
  if (/junior|джун|младш|стаж[её]р/i.test(text)) grades.push("junior");
  if (/middle|мидл|миддл/i.test(text)) grades.push("middle");
  if (/senior|сеньор|синьор|ведущ|старш/i.test(text)) grades.push("senior");

  const must: string[] = [];
  const nice: string[] = [];
  for (const sentence of text.split(/(?<=[.!?\n;])/)) {
    const isNice = /желательн|плюсом|будет плюс|nice to have|приветству/i.test(sentence);
    const sl = sentence.toLowerCase();
    for (const skill of SKILLS) {
      if (must.includes(skill.slug) || nice.includes(skill.slug)) continue;
      const terms = [skill.name.toLowerCase(), ...skill.synonyms.map((x) => x.toLowerCase())].filter((term) => term.length >= 2);
      const hit = terms.some((term) => new RegExp(`(^|[^a-zа-яё0-9])${term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}($|[^a-zа-яё0-9])`, "i").test(sl));
      if (hit) (isNice ? nice : must).push(skill.slug);
    }
  }

  const toRub = (raw: string, unit: string | undefined) => {
    const n = Number(raw.replace(/\s/g, ""));
    if (unit && /тыс|k|к/i.test(unit)) return n * 1000;
    return n < 1000 ? n * 1000 : n;
  };
  let min: number | null = null;
  let max: number | null = null;
  const range = text.match(/(\d[\d\s]{1,8}?)\s*(тыс\.?|k|к)?\s*(?:₽|руб\.?)?\s*(?:–|-|—|до)\s*(\d[\d\s]{1,8}?)\s*(тыс\.?|k|к)?(?=\D|$)/i);
  if (range) {
    const unit = range[4] ?? range[2];
    min = toRub(range[1] ?? "0", range[2] ?? unit);
    max = toRub(range[3] ?? "0", unit);
  } else {
    const from = text.match(/от\s*(\d[\d\s]{1,8}?)\s*(тыс\.?|k|к)?(?=\D|$)/i);
    if (from) min = toRub(from[1] ?? "0", from[2]);
  }

  const work_formats: T.WorkFormat[] = [];
  if (/удал[её]н|remote|из любого города/i.test(text)) work_formats.push("remote");
  if (/гибрид|hybrid/i.test(text)) work_formats.push("hybrid");
  if (/офис|office/i.test(text)) work_formats.push("office");
  const industries: T.Industry[] = [];
  if (/финтех|банк|платеж|платёж/i.test(text)) industries.push("fintech");
  if (/маркетплейс|e-commerce|ритейл/i.test(text)) industries.push("ecommerce");
  const years = text.match(/(?:от|опыт)\s*(\d+)\+?\s*(?:лет|года|год)/i);

  return {
    specialization: top && top.n > 0 ? { specialization: top.s, confidence: Math.round((top.n / Math.max(1, total)) * 100) / 100 } : null,
    grade: grades[0] ?? null,
    grades,
    must_skills: must,
    nice_skills: nice,
    salary_min: min,
    salary_max: max,
    work_formats,
    city: CITY_STEMS.find(([, re]) => re.test(text))?.[0] ?? null,
    industries,
    experience_years_min: years ? Number(years[1]) : null,
  };
}

function effectiveNeed(n: MockNeed, f: T.ShortlistFilters): NeedLike & { allow_adjacent_grades: boolean; work_formats: T.WorkFormat[]; city: string | null } {
  return {
    specialization: n.specialization,
    grades: f.grades ?? n.grades,
    allow_adjacent_grades: f.allow_adjacent_grades ?? n.allow_adjacent_grades,
    must_skills: f.must_skills ?? n.must_skills,
    nice_skills: f.nice_skills ?? n.nice_skills,
    salary_max: n.salary.max,
    work_formats: f.work_formats ?? n.work_formats,
    city: f.city ?? n.city,
    description: `${n.title}\n${n.description}`,
  };
}

export async function getNeedCategories(id: T.Uuid): Promise<T.NeedCategory[]> {
  await delay();
  const { need } = ownNeed(id);
  return GRADES.map((grade) => {
    const list = inCategory(need.specialization, grade);
    const exps = list.map((c) => c.profile.salary_expectation_min).filter((x): x is number => typeof x === "number");
    const stack = list.length ? list.reduce((acc, c) => acc + (need.must_skills.length ? need.must_skills.filter((s) => s in c.skill_sources).length / need.must_skills.length : 1), 0) / list.length : 0;
    return {
      category: { specialization: need.specialization, grade },
      candidates: list.length,
      with_fsp: list.filter((c) => c.privacy.show_fsp && c.fsp.achievements.length > 0).length,
      median_salary: median(exps),
      stack_match: Math.round(stack * 100) / 100,
      requested: need.grades.includes(grade),
    };
  });
}

function rankFor(need: MockNeed, f: T.ShortlistFilters) {
  const t = now();
  const eff = effectiveNeed(need, f);
  const out: Array<{ c: MockCandidate; b: T.ScoreBreakdown }> = [];
  for (const c of getDb().candidates) {
    if (!searchable(c)) continue;
    const a = currentAssignment(c);
    if (!a || a.category.specialization !== need.specialization) continue;
    const g = a.category.grade;
    const gradeOk = eff.grades.includes(g) || (eff.allow_adjacent_grades && eff.grades.some((x) => Math.abs(GRADES.indexOf(x) - GRADES.indexOf(g)) === 1));
    if (!gradeOk) continue;
    if (!formatCompatible(c.profile.work_formats ?? [], eff.work_formats, c.profile.city ?? null, eff.city, c.profile.relocation ?? false)) continue;
    if (f.fsp_only && !(c.privacy.show_fsp && c.fsp.achievements.length)) continue;
    const exp = c.profile.salary_expectation_min;
    if (f.max_expectation && typeof exp === "number" && exp > f.max_expectation) continue;
    out.push({ c, b: explain(c, eff, t) });
  }
  return out.sort((x, y) => y.b.total - x.b.total).slice(0, Math.min(f.limit ?? SHORTLIST_SIZE, 50));
}

function shortlistDto(need: MockNeed, companyId: string): T.Shortlist {
  const t = now();
  refreshInvitationStatuses();
  const db = getDb();
  const sl = db.shortlists[need.id];
  const ranked = rankFor(need, need.filters);
  const current = new Map(ranked.map((r, i) => [r.c.id, { ...r, rank: i + 1 }]));
  const items: T.ShortlistItem[] = [];
  const gen = need.shortlist_generated_at;
  const hadPrevious = Boolean(gen && sl && Object.values(sl.entries).some((e) => e.first_seen_at < gen));
  for (const [id, entry] of Object.entries(sl?.entries ?? {})) {
    const inCurrent = current.get(id);
    if (!inCurrent && !entry.pinned) continue;
    const c = db.candidates.find((x) => x.id === id);
    if (!c) continue;
    const eff = effectiveNeed(need, need.filters);
    const b = inCurrent?.b ?? explain(c, eff, t);
    items.push({
      candidate: toCard(c, t, relationOf(companyId, c.id)),
      score: b.total,
      rank: inCurrent?.rank ?? 999,
      breakdown: b,
      in_current: Boolean(inCurrent),
      pinned: entry.pinned,
      dismissed: entry.dismissed,
      is_new: hadPrevious && Boolean(gen) && entry.first_seen_at >= (gen as string),
      first_seen_at: entry.first_seen_at,
    });
  }
  items.sort((a, b) => Number(b.pinned) - Number(a.pinned) || Number(b.in_current) - Number(a.in_current) || a.rank - b.rank);
  return { need: needDto(need), items };
}

export async function refineShortlist(id: T.Uuid, filters: T.ShortlistFilters): Promise<T.Shortlist> {
  await delay(400, 700);
  const { need, company } = ownNeed(id);
  const db = getDb();
  for (const list of [filters.must_skills, filters.nice_skills]) {
    const unknown = (list ?? []).filter((s) => !SKILL_BY_SLUG.has(s));
    if (unknown.length) validation({ skills: [`Неизвестные навыки: ${unknown.join(", ")}`] });
  }
  const at = nowIso();
  need.filters = clone(filters);
  need.shortlist_generated_at = at;
  const sl: MockShortlist = db.shortlists[id] ?? { need_id: id, entries: {} };
  for (const { c } of rankFor(need, filters)) {
    sl.entries[c.id] ??= { pinned: false, dismissed: false, is_new: false, first_seen_at: at };
  }
  db.shortlists[id] = sl;
  persist();
  return shortlistDto(need, company.id);
}

export async function getShortlist(id: T.Uuid): Promise<T.Shortlist> {
  await delay(300, 550);
  const { need, company } = ownNeed(id);
  return shortlistDto(need, company.id);
}

export async function patchShortlistItem(id: T.Uuid, candidateId: T.Uuid, body: T.ShortlistPatch): Promise<void> {
  await delay(100, 220);
  ownNeed(id);
  const db = getDb();
  const sl: MockShortlist = db.shortlists[id] ?? { need_id: id, entries: {} };
  const entry = sl.entries[candidateId];
  if (!entry) fail(404, "not_found", "Кандидата нет в подборке");
  if (body.pinned !== null && body.pinned !== undefined) entry.pinned = body.pinned;
  if (body.dismissed !== null && body.dismissed !== undefined) {
    entry.dismissed = body.dismissed;
    if (body.dismissed) entry.pinned = false;
  }
  db.shortlists[id] = sl;
  persist();
}

function testPlan(spec: T.Specialization, grade: T.Grade, skills: string[]): T.TestPlanDemo {
  const topics = SPEC_TOPICS[spec];
  const samples = ["Кандидат А", "Кандидат Б"].map((label, k) => {
    const items: T.ItemView[] = [];
    const used: Array<{ template_id: string; topic: T.AssessmentTopic }> = [];
    for (let i = 0; i < 3; i++) {
      const picked = selectNext(spec, GRADE_THRESHOLD[grade], used, seedFromString(`${spec}:${grade}:${k}:${i}:${skills.join(",")}`));
      if (!picked) break;
      used.push({ template_id: picked.template_id, topic: picked.topic });
      items.push({
        template_id: picked.template_id,
        level: picked.level,
        title: picked.title,
        topic: picked.topic,
        kind: picked.item.kind,
        prompt: picked.item.prompt,
        code: picked.item.code,
        options: picked.item.options,
        input_hint: picked.item.input_hint,
        time_limit_secs: picked.item.time_limit_secs,
      });
    }
    return { label, items };
  });
  return {
    plan: {
      specialization: spec,
      grade,
      target_theta: GRADE_THRESHOLD[grade],
      min_items: 8,
      max_items: 20,
      topics: topics.map((t) => ({ topic: t, topic_ru: TOPIC_LABEL[t], templates: [] })),
      focus_templates: [],
    },
    samples,
  };
}

export async function getNeedTestPlan(id: T.Uuid): Promise<T.TestPlanDemo> {
  await delay(300, 600);
  const { need } = ownNeed(id);
  return testPlan(need.specialization, need.grades[0] ?? "middle", need.must_skills);
}

export async function getTestPlan(params: { specialization: T.Specialization; grade: T.Grade; skills?: string }): Promise<T.TestPlanDemo> {
  await delay(300, 600);
  return testPlan(params.specialization, params.grade, (params.skills ?? "").split(",").filter(Boolean));
}

export async function listCategories(): Promise<T.CategoryCard[]> {
  await delay();
  requireEmployer();
  return SPECIALIZATIONS.flatMap((specialization) =>
    GRADES.map((grade) => {
      const list = inCategory(specialization, grade);
      const exps = list.map((c) => c.profile.salary_expectation_min).filter((x): x is number => typeof x === "number");
      return {
        category: { specialization, grade },
        name: categoryTitle(specialization, grade),
        candidates: list.length,
        with_fsp: list.filter((c) => c.privacy.show_fsp && c.fsp.achievements.length > 0).length,
        actively_looking: list.filter((c) => c.privacy.search_status === "active").length,
        median_expectation: median(exps),
      };
    }),
  );
}

export async function searchCandidates(params: T.CandidateSearchParams): Promise<T.Page<T.BankItem>> {
  await delay(200, 420);
  const { company } = requireEmployer();
  const db = getDb();
  const t = now();
  if (!params.specialization) validation({ specialization: ["Выберите специализацию"] });
  const skills = (params.skills ?? "")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  let q = params.q?.trim().toLowerCase() ?? "";
  const asSkill = q ? SKILLS.find((s) => s.name.toLowerCase() === q || s.slug === q || s.synonyms.includes(q)) : undefined;
  if (asSkill) {
    skills.push(asSkill.slug);
    q = "";
  }
  const need = params.need_id ? db.needs.find((n) => n.id === params.need_id && n.company_id === company.id) : undefined;
  const items = db.candidates
    .filter(searchable)
    .filter((c) => {
      const a = currentAssignment(c);
      if (!a || a.category.specialization !== params.specialization) return false;
      if (params.grade && a.category.grade !== params.grade) return false;
      if (params.fsp_only && !(c.privacy.show_fsp && c.fsp.achievements.length)) return false;
      if (params.work_format && !(c.profile.work_formats ?? []).includes(params.work_format)) return false;
      if (params.city && c.profile.city !== params.city && !(c.profile.work_formats ?? []).includes("remote")) return false;
      const exp = c.profile.salary_expectation_min;
      if (params.max_expectation && typeof exp === "number" && exp > params.max_expectation) return false;
      if (skills.length && !skills.every((s) => s in c.skill_sources)) return false;
      if (q && !(c.profile.about ?? "").toLowerCase().includes(q)) return false;
      return true;
    })
    .map((c) => bankItem(toCard(c, t, relationOf(company.id, c.id)), c, t, need ? effectiveNeed(need, need.filters) : null))
    .sort((a, b) => (need ? (b.score ?? 0) - (a.score ?? 0) : b.strength - a.strength));
  const offset = params.offset ?? 0;
  const limit = Math.min(params.limit ?? 20, 100);
  return { items: items.slice(offset, offset + limit), total: items.length };
}

export async function getCandidateForEmployer(id: T.Uuid, needId?: T.Uuid | null): Promise<T.CandidateProfileView> {
  await delay();
  const { company } = requireEmployer();
  const db = getDb();
  const t = now();
  refreshInvitationStatuses();
  const c = db.candidates.find((x) => x.id === id);
  const relation = relationOf(company.id, id);
  if (!c || (!searchable(c) && !relation.invitation_status && !relation.applied)) fail(404, "not_found", "Кандидат не найден или скрыл профиль");
  const need = needId ? db.needs.find((n) => n.id === needId && n.company_id === company.id) : undefined;
  return { candidate: toCard(c, t, relation), breakdown: need ? explain(c, effectiveNeed(need, need.filters), t) : null };
}

export async function getCandidateContacts(id: T.Uuid): Promise<T.CandidateContacts> {
  await delay();
  const { company } = requireEmployer();
  const db = getDb();
  const c = db.candidates.find((x) => x.id === id);
  if (!c) fail(404, "not_found", "Кандидат не найден");
  const invitation = db.invitations.find((i) => i.company_id === company.id && i.candidate_id === id && i.status === "accepted");
  const vacancyIds = new Set(db.vacancies.filter((v) => v.company_id === company.id).map((v) => v.id));
  const application = db.applications.find((a) => a.candidate_id === id && vacancyIds.has(a.vacancy_id) && a.status !== "withdrawn");
  if (!invitation && !application) fail(403, "contacts_not_granted", "Контакты пока недоступны", "Они откроются, когда кандидат примет приглашение или сам откликнется на вакансию");
  const basis: T.ContactBasis = invitation ? "invitation" : "application";
  db.contact_log.push({ candidate_id: c.id, company_id: company.id, basis, at: nowIso() });
  persist();
  return { candidate_id: c.id, full_name: c.profile.full_name || null, ...clone(c.profile.contacts), basis, basis_id: (invitation?.id ?? application?.id) as string };
}

function sentDto(inv: MockInvitation, withEvents = false): T.SentInvitation {
  const c = getDb().candidates.find((x) => x.id === inv.candidate_id);
  return {
    id: inv.id,
    candidate_id: inv.candidate_id,
    candidate_code: c?.code ?? "—",
    candidate_category: c ? (currentAssignment(c)?.category ?? null) : null,
    need_id: inv.need_id,
    vacancy_id: inv.vacancy_id,
    title: inv.title,
    message: inv.message,
    salary: clone(inv.salary),
    work_format: inv.work_format,
    contact: inv.contact,
    status: inv.status,
    decline_reason: inv.decline_reason,
    decline_comment: inv.decline_comment,
    created_at: inv.created_at,
    viewed_at: inv.viewed_at,
    responded_at: inv.responded_at,
    expires_at: inv.expires_at,
    ...(withEvents ? { events: clone(inv.events) } : {}),
  };
}

export async function listSentInvitations(query: T.InvitationListQuery = {}): Promise<T.SentInvitationPage> {
  await delay();
  const { company } = requireEmployer();
  refreshInvitationStatuses();
  persist();
  const sent = getDb()
    .invitations.filter((i) => i.company_id === company.id)
    .sort((a, b) => b.created_at.localeCompare(a.created_at));
  const wanted = sent.filter((i) => !query.status?.length || query.status.includes(i.status));
  const page = pageOf(wanted, query.cursor, query.limit ?? 30);
  return { items: page.items.map((i) => sentDto(i)), next_cursor: page.next_cursor, by_status: statusCounts(sent) };
}

function quotaInfo(companyId: string) {
  const t = now().getTime();
  const recent = getDb()
    .invitations.filter((i) => i.company_id === companyId && t - new Date(i.created_at).getTime() < QUOTA_PERIOD_DAYS * DAY)
    .sort((a, b) => a.created_at.localeCompare(b.created_at));
  const oldest = recent[0];
  const resetsInDays = oldest ? Math.max(1, Math.ceil((addDays(oldest.created_at, QUOTA_PERIOD_DAYS).getTime() - t) / DAY)) : QUOTA_PERIOD_DAYS;
  return { used: recent.length, resetsInDays };
}

export async function sendInvitation(body: T.SendInvitation, idempotencyKey: string): Promise<T.SentInvitation> {
  await delay(350, 600);
  const { company } = requireEmployer();
  const db = getDb();
  const existing = db.invitations.find((i) => i.idempotency_key === idempotencyKey && i.company_id === company.id);
  if (existing) return sentDto(existing, true);

  const errors: FieldErrors = {};
  if (body.title.trim().length < 3) errors.title = ["Укажите, на какую роль приглашаете"];
  if (body.message.trim().length < 20) errors.message = ["Расскажите о задаче подробнее — минимум 20 символов"];
  salaryErrors(body.salary, errors);
  if (!body.contact.trim()) errors.contact = ["Укажите, как с вами связаться"];
  if (Object.keys(errors).length) validation(errors);

  refreshInvitationStatuses();
  const c = db.candidates.find((x) => x.id === body.candidate_id);
  if (!c || !currentAssignment(c) || !c.privacy.visible) fail(404, "not_found", "Кандидат не найден или скрыт");
  if (c.privacy.search_status === "closed") fail(409, "candidate_not_available", "Кандидат сейчас не ищет работу");
  if (db.invitations.some((i) => i.company_id === company.id && i.candidate_id === c.id && (i.status === "sent" || i.status === "viewed"))) {
    fail(409, "invitation_already_active", "Приглашение уже отправлено", "У этого кандидата уже есть ваше активное приглашение — дождитесь ответа");
  }
  const quota = quotaInfo(company.id);
  if (quota.used >= INVITATION_QUOTA) {
    fail(409, "invitation_quota_exceeded", "Лимит приглашений исчерпан", `Не больше ${INVITATION_QUOTA} приглашений за ${QUOTA_PERIOD_DAYS} дней. Квота обновится через ${quota.resetsInDays} дн.`);
  }
  const at = nowIso();
  const inv: MockInvitation = {
    id: newId(),
    company_id: company.id,
    candidate_id: c.id,
    status: "sent",
    title: body.title.trim(),
    message: body.message.trim(),
    salary: clone(body.salary),
    work_format: body.work_format ?? null,
    contact: body.contact.trim(),
    need_id: body.need_id ?? null,
    vacancy_id: body.vacancy_id ?? null,
    created_at: at,
    viewed_at: null,
    responded_at: null,
    expires_at: addDays(at, INVITATION_TTL_DAYS).toISOString(),
    decline_reason: null,
    decline_comment: null,
    events: [{ status: "sent", at }],
    idempotency_key: idempotencyKey,
  };
  db.invitations.push(inv);
  persist();
  return sentDto(inv, true);
}

function ownSent(id: string): MockInvitation {
  const { company } = requireEmployer();
  refreshInvitationStatuses();
  const inv = getDb().invitations.find((i) => i.id === id && i.company_id === company.id);
  if (!inv) fail(404, "not_found", "Приглашение не найдено");
  return inv;
}

export async function getSentInvitation(id: T.Uuid): Promise<T.SentInvitation> {
  await delay();
  return sentDto(ownSent(id), true);
}

export async function withdrawInvitation(id: T.Uuid): Promise<void> {
  await delay();
  const inv = ownSent(id);
  assertActive(inv);
  inv.status = "withdrawn";
  inv.events.push({ status: "withdrawn", at: nowIso() });
  persist();
}

export async function getInvitationStats(): Promise<T.InvitationStats> {
  await delay();
  const { company } = requireEmployer();
  refreshInvitationStatuses();
  const list = getDb().invitations.filter((i) => i.company_id === company.id);
  const statuses: T.InvitationStatus[] = ["sent", "viewed", "accepted", "declined", "expired", "withdrawn"];
  const count = (s: T.InvitationStatus) => list.filter((i) => i.status === s).length;
  const answered = count("accepted") + count("declined");
  const considered = list.length - count("withdrawn");
  const reasons = new Map<T.DeclineReason, number>();
  for (const i of list) if (i.decline_reason) reasons.set(i.decline_reason, (reasons.get(i.decline_reason) ?? 0) + 1);
  const quota = quotaInfo(company.id);
  return {
    quota_limit: INVITATION_QUOTA,
    quota_max: INVITATION_QUOTA,
    trust_level: "high",
    quota_used: quota.used,
    quota_resets_in_days: quota.resetsInDays,
    by_status: statuses.map((status) => ({ status, count: count(status) })),
    response_rate: considered ? Math.round((answered / considered) * 100) / 100 : 0,
    decline_reasons: [...reasons.entries()].sort((a, b) => b[1] - a[1]).map(([reason, n]) => ({ reason, count: n })),
    hiring: [],
  };
}

export async function getTrustStatus(): Promise<T.TrustStatus> {
  await delay();
  const { company } = requireEmployer();
  const quota = quotaInfo(company.id);
  return {
    level: "high",
    frozen_until: null,
    moderation: [],
    quota_limit: INVITATION_QUOTA,
    quota_used: quota.used,
    quota_base: 10,
    quota_min: 5,
    quota_max: INVITATION_QUOTA,
    bonus: 20,
    penalty: 0,
    reasons: [
      { kind: "account_age", points: 10, text: "На платформе 120 дней" },
      { kind: "accepted", points: 10, text: "Кандидаты приняли 4 приглашения за полгода" },
    ],
    advice: [],
    domain: { domain: null, check: { status: "no_website", email_domain: "demo.fsp" }, hint: "Укажите сайт компании в профиле." },
    rule: ["Новая компания начинает с 10 приглашений в неделю."],
  };
}

export async function checkSalary(query: T.SalaryCheckQuery): Promise<T.SalaryCheck> {
  await delay(80, 160);
  requireEmployer();
  const wide = query.max >= query.min * 3;
  return {
    category: { specialization: query.specialization, grade: query.grade },
    market: null,
    warnings: wide ? [{ kind: "salary_wide_range", text: "Вилка очень широкая: так обычно описывают сразу несколько грейдов." }] : [],
  };
}

export async function findSimilarVacancies(body: T.SimilarInput): Promise<T.SimilarVacancy[]> {
  await delay(80, 160);
  const { company } = requireEmployer();
  const title = body.title.trim().toLowerCase();
  return getDb()
    .vacancies.filter((v) => v.company_id === company.id && v.id !== body.exclude_id && v.status !== "closed" && v.title.trim().toLowerCase() === title)
    .map((v) => ({ id: v.id, title: v.title, status: v.status, similarity: 1 }));
}

function vacancyDto(v: MockVacancy): T.Vacancy {
  const { company_id: _company, ...rest } = v;
  return { ...clone(rest), applications: getDb().applications.filter((a) => a.vacancy_id === v.id && a.status !== "withdrawn").length, warnings: [] };
}

function validateVacancy(body: T.VacancyInput) {
  const errors: FieldErrors = {};
  if (body.title.trim().length < 3) errors.title = ["Укажите название"];
  salaryErrors(body.salary, errors);
  if (body.description.trim().length < 20) errors.description = ["Опишите задачи — минимум 20 символов"];
  if (Object.keys(errors).length) validation(errors);
}

export async function listOwnVacancies(): Promise<T.Vacancy[]> {
  await delay();
  const { company } = requireEmployer();
  return getDb()
    .vacancies.filter((v) => v.company_id === company.id)
    .sort((a, b) => b.created_at.localeCompare(a.created_at))
    .map(vacancyDto);
}

export async function createVacancy(body: T.VacancyInput, idempotencyKey: string): Promise<T.Vacancy> {
  await delay(300, 500);
  const { company } = requireEmployer();
  validateVacancy(body);
  const existing = getDb().vacancies.find((v) => v.company_id === company.id && v.idempotency_key === idempotencyKey);
  if (existing) return vacancyDto(existing);
  const at = nowIso();
  const v: MockVacancy = {
    id: newId(),
    company_id: company.id,
    title: body.title,
    description: body.description,
    specialization: body.specialization,
    grade: body.grade,
    skills: body.skills ?? [],
    salary: clone(body.salary),
    work_format: body.work_format,
    city: body.city ?? null,
    status: body.status,
    published_at: body.status === "published" ? at : null,
    created_at: at,
    updated_at: at,
    idempotency_key: idempotencyKey,
  };
  getDb().vacancies.push(v);
  persist();
  return vacancyDto(v);
}

function ownVacancy(id: string): MockVacancy {
  const { company } = requireEmployer();
  const v = getDb().vacancies.find((x) => x.id === id && x.company_id === company.id);
  if (!v) fail(404, "not_found", "Вакансия не найдена");
  return v;
}

export async function getOwnVacancy(id: T.Uuid): Promise<T.Vacancy> {
  await delay();
  return vacancyDto(ownVacancy(id));
}

export async function updateVacancy(id: T.Uuid, body: T.VacancyInput): Promise<T.Vacancy> {
  await delay(250, 450);
  const v = ownVacancy(id);
  validateVacancy(body);
  const wasPublished = v.status === "published";
  Object.assign(v, { title: body.title, description: body.description, specialization: body.specialization, grade: body.grade, skills: body.skills ?? [], salary: clone(body.salary), work_format: body.work_format, city: body.city ?? null, status: body.status, updated_at: nowIso() });
  if (body.status === "published" && !wasPublished) v.published_at = nowIso();
  persist();
  return vacancyDto(v);
}

export async function listVacancyApplications(id: T.Uuid, cursor?: string | null): Promise<T.ApplicationPage> {
  await delay();
  ownVacancy(id);
  const db = getDb();
  const all = db.applications
    .filter((a) => a.vacancy_id === id)
    .sort((a, b) => b.created_at.localeCompare(a.created_at))
    .map((a): T.IncomingApplication => {
      const c = db.candidates.find((x) => x.id === a.candidate_id);
      return {
        id: a.id,
        candidate_id: a.candidate_id,
        candidate_code: c?.code ?? "—",
        candidate_category: c ? (currentAssignment(c)?.category ?? null) : null,
        cover_letter: a.cover_letter,
        status: a.status,
        created_at: a.created_at,
        updated_at: a.updated_at,
      };
    });
  return pageOf(all, cursor, 30);
}

export async function setApplicationStatus(id: T.Uuid, status: T.EmployerApplicationStatus): Promise<void> {
  await delay();
  const { company } = requireEmployer();
  const db = getDb();
  const a = db.applications.find((x) => x.id === id);
  const v = a ? db.vacancies.find((x) => x.id === a.vacancy_id && x.company_id === company.id) : undefined;
  if (!a || !v) fail(404, "not_found", "Отклик не найден");
  if (a.status === "withdrawn") fail(409, "application_withdrawn", "Кандидат отозвал отклик");
  a.status = status;
  a.updated_at = nowIso();
  persist();
}

function employerTaskDto(task: MockTask): T.EmployerTask {
  const subs = getDb().task_assignments.filter((a) => a.task_id === task.id && (a.status === "submitted" || a.status === "reviewed"));
  const { company_id: _company, ...rest } = task;
  return { ...clone(rest), submissions: subs.length, reviewed: subs.filter((s) => s.status === "reviewed").length };
}

export async function listEmployerTasks(): Promise<T.EmployerTask[]> {
  await delay();
  const { company } = requireEmployer();
  return getDb()
    .tasks.filter((t) => t.company_id === company.id)
    .sort((a, b) => b.created_at.localeCompare(a.created_at))
    .map(employerTaskDto);
}

export async function createEmployerTask(body: T.TaskInput): Promise<T.EmployerTask> {
  await delay(300, 500);
  const { company } = requireEmployer();
  const errors: FieldErrors = {};
  if (body.title.trim().length < 3) errors.title = ["Укажите название"];
  if (body.body.trim().length < 30) errors.body = ["Опишите задание — минимум 30 символов"];
  if (!body.grades.length) errors.grades = ["Выберите хотя бы один грейд"];
  if (Object.keys(errors).length) validation(errors);
  const task: MockTask = { title: body.title, body: body.body, kind: body.kind, specialization: body.specialization, grades: body.grades, skills: body.skills ?? [], id: newId(), company_id: company.id, active: true, created_at: nowIso() };
  getDb().tasks.push(task);
  persist();
  return employerTaskDto(task);
}

function ownTask(id: string): MockTask {
  const { company } = requireEmployer();
  const task = getDb().tasks.find((t) => t.id === id && t.company_id === company.id);
  if (!task) fail(404, "not_found", "Задание не найдено");
  return task;
}

export async function setTaskActive(id: T.Uuid, active: boolean): Promise<void> {
  await delay();
  ownTask(id).active = active;
  persist();
}

export async function listTaskSubmissions(id: T.Uuid, cursor?: string | null): Promise<T.SubmissionPage> {
  await delay();
  ownTask(id);
  const db = getDb();
  const all = db.task_assignments
    .filter((a) => a.task_id === id && (a.status === "submitted" || a.status === "reviewed"))
    .sort((a, b) => (b.submitted_at ?? "").localeCompare(a.submitted_at ?? ""))
    .map((a): T.TaskSubmission => {
      const c = db.candidates.find((x) => x.id === a.candidate_id);
      return {
        id: a.id,
        candidate_id: a.candidate_id,
        candidate_code: c?.code ?? "—",
        candidate_category: c ? (currentAssignment(c)?.category ?? null) : null,
        status: a.status,
        answer: a.answer,
        rating: a.rating,
        review_comment: a.review_comment,
        submitted_at: a.submitted_at,
      };
    });
  return pageOf(all, cursor, 30);
}

export async function reviewTaskSubmission(id: T.Uuid, submissionId: T.Uuid, body: T.ReviewInput): Promise<void> {
  await delay();
  ownTask(id);
  const a = getDb().task_assignments.find((x) => x.id === submissionId && x.task_id === id);
  if (!a || (a.status !== "submitted" && a.status !== "reviewed")) fail(404, "not_found", "Решение не найдено");
  if (!(body.rating >= 1 && body.rating <= 5)) validation({ rating: ["Оценка от 1 до 5"] });
  a.status = "reviewed";
  a.rating = body.rating;
  a.review_comment = body.comment?.trim() || null;
  a.reviewed_at = nowIso();
  const c = getDb().candidates.find((x) => x.id === a.candidate_id);
  if (c) c.last_active_at = nowIso();
  persist();
}

function publicDto(v: MockVacancy, detailed = false): T.PublicVacancy {
  const c = companyOf(v.company_id);
  return {
    id: v.id,
    company_id: v.company_id,
    company_name: c?.name ?? "Компания",
    company_city: c?.city ?? null,
    ...(detailed ? { company_description: c?.description ?? null } : {}),
    title: v.title,
    description: v.description,
    specialization: v.specialization,
    grade: v.grade,
    skills: [...v.skills],
    salary: clone(v.salary),
    work_format: v.work_format,
    city: v.city,
    published_at: v.published_at,
  };
}

export async function listVacancies(params: T.VacancyQuery = {}): Promise<T.Page<T.PublicVacancy>> {
  await delay(150, 350);
  const q = params.q?.trim().toLowerCase() ?? "";
  const list = getDb()
    .vacancies.filter((v) => v.status === "published")
    .filter((v) => {
      if (params.specialization && v.specialization !== params.specialization) return false;
      if (params.grade && v.grade !== params.grade) return false;
      if (params.work_format && v.work_format !== params.work_format) return false;
      if (params.salary_from && v.salary.max < params.salary_from) return false;
      if (q) {
        const hay = `${v.title} ${v.description} ${v.skills.map(skillName).join(" ")} ${companyOf(v.company_id)?.name ?? ""} ${v.city ?? ""}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      return true;
    })
    .sort((a, b) => (b.published_at ?? "").localeCompare(a.published_at ?? ""));
  const offset = params.offset ?? 0;
  const limit = Math.min(params.limit ?? 20, 100);
  return { items: list.slice(offset, offset + limit).map((v) => publicDto(v)), total: list.length };
}

export async function getVacancy(id: T.Uuid): Promise<T.PublicVacancy> {
  await delay();
  const v = getDb().vacancies.find((x) => x.id === id && x.status === "published");
  if (!v) fail(404, "not_found", "Вакансия не найдена или снята с публикации");
  return publicDto(v, true);
}

/** Mocks have no embeddings: the same specialization, most shared skills first, as the API without the embedder. */
export async function getSimilarVacancies(id: T.Uuid, limit = 4): Promise<T.RelatedVacancies> {
  await delay();
  const db = getDb();
  const v = db.vacancies.find((x) => x.id === id && x.status === "published");
  if (!v) fail(404, "not_found", "Вакансия не найдена или снята с публикации");
  const items = db.vacancies
    .filter((x) => x.status === "published" && x.id !== v.id && x.specialization === v.specialization)
    .map((x) => ({ vacancy: publicDto(x), similarity: x.skills.filter((s) => v.skills.includes(s)).length / Math.max(v.skills.length, 1) }))
    .sort((a, b) => b.similarity - a.similarity)
    .slice(0, limit);
  return { method: "skills", items };
}

export async function applyToVacancy(id: T.Uuid, body: T.ApplyInput): Promise<T.MyApplication> {
  await delay(300, 500);
  const { cand } = requireCandidate();
  const db = getDb();
  const v = db.vacancies.find((x) => x.id === id && x.status === "published");
  if (!v) fail(404, "not_found", "Вакансия не найдена");
  if (db.applications.some((a) => a.vacancy_id === id && a.candidate_id === cand.id && a.status !== "withdrawn")) fail(409, "already_applied", "Вы уже откликнулись на эту вакансию");
  const at = nowIso();
  const a: MockApplication = { id: newId(), vacancy_id: id, candidate_id: cand.id, status: "sent", cover_letter: (body.cover_letter ?? "").trim(), created_at: at, updated_at: at };
  db.applications.push(a);
  cand.last_active_at = at;
  persist();
  const dto = myApplicationDto(a);
  if (!dto) fail(404, "not_found", "Вакансия не найдена");
  return dto;
}

export async function createReport(body: T.ReportInput): Promise<T.ReportCreated> {
  await delay();
  const user = sessionUser();
  const reports = getDb().reports;
  const open = reports.find((r) => r.user_id === user.id && r.target === body.target_kind && r.target_id === body.target_id);
  if (open?.id) return { id: open.id, already_open: true };
  const id = newId();
  reports.push({ id, user_id: user.id, target: body.target_kind, target_id: body.target_id, reason: body.comment ? `${body.reason}: ${body.comment}` : body.reason, at: nowIso() });
  persist();
  return { id, already_open: false };
}

export * from "./integrations";
export * from "./notifications";
