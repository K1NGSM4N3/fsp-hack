import type {
  Answer,
  AssessmentSession,
  AssessmentTopic,
  Category,
  CodeBlock,
  Company,
  DeclineReason,
  Grade,
  Industry,
  InvitationStatus,
  ApplicationStatus,
  ItemKind,
  NeedStatus,
  Privacy,
  ProfileInput,
  Role,
  SalaryRange,
  ShortlistFilters,
  SkillSource,
  Specialization,
  StatusEvent,
  SurveyResponse,
  TaskKind,
  TaskStatus,
  UserConsent,
  VacancyStatus,
  WorkFormat,
} from "@/api/types";
import type { RawAchievement } from "./fsp";
import { buildSeed, SEED_VERSION } from "./seed";

export interface MockUser {
  id: string;
  email: string;
  password: string;
  role: Exclude<Role, "admin">;
  email_verified: boolean;
  verify_token: string | null;
  candidate_id: string | null;
  company_id: string | null;
  created_at: string;
  consents: UserConsent[];
}

export interface MockAssignment {
  id: string;
  category: Category;
  theta: number;
  /** Position of θ inside the grade band, 0..1. */
  strength: number;
  reason: "placement" | "upgrade" | "downgrade";
  session_id: string | null;
  assigned_at: string;
}

export interface MockCandidate {
  id: string;
  user_id: string | null;
  code: string;
  profile: ProfileInput;
  skill_sources: Record<string, SkillSource[]>;
  profile_saved_at: string | null;
  privacy: Privacy;
  surveys: SurveyResponse[];
  assignments: MockAssignment[];
  placement: { open: boolean; offer: "step_down" | "upgrade" | null; offer_grade: Grade | null };
  retry_after: Partial<Record<Grade, string>>;
  no_category_retry_after: string | null;
  fsp: { fsp_id: string | null; linked_at: string | null; last_sync_at: string | null; achievements: RawAchievement[] };
  last_active_at: string;
}

export interface MockCompany extends Company {
  owner_user_id: string | null;
}

export interface MockNeed {
  id: string;
  company_id: string;
  title: string;
  description: string;
  specialization: Specialization;
  grades: Grade[];
  allow_adjacent_grades: boolean;
  must_skills: string[];
  nice_skills: string[];
  salary: SalaryRange;
  work_formats: WorkFormat[];
  city: string | null;
  industries: Industry[];
  status: NeedStatus;
  filters: ShortlistFilters;
  shortlist_generated_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface MockShortlistEntry {
  pinned: boolean;
  dismissed: boolean;
  is_new: boolean;
  first_seen_at: string;
}

export interface MockShortlist {
  need_id: string;
  entries: Record<string, MockShortlistEntry>;
}

export interface MockInvitation {
  id: string;
  company_id: string;
  candidate_id: string;
  status: InvitationStatus;
  title: string;
  message: string;
  salary: SalaryRange;
  work_format: WorkFormat | null;
  contact: string;
  need_id: string | null;
  vacancy_id: string | null;
  created_at: string;
  viewed_at: string | null;
  responded_at: string | null;
  expires_at: string;
  decline_reason: DeclineReason | null;
  decline_comment: string | null;
  events: StatusEvent[];
  idempotency_key: string | null;
}

export interface MockVacancy {
  id: string;
  company_id: string;
  title: string;
  description: string;
  specialization: Specialization;
  grade: Grade;
  skills: string[];
  salary: SalaryRange;
  work_format: WorkFormat;
  city: string | null;
  status: VacancyStatus;
  published_at: string | null;
  created_at: string;
  updated_at: string;
  idempotency_key?: string;
}

export interface MockApplication {
  id: string;
  vacancy_id: string;
  candidate_id: string;
  status: ApplicationStatus;
  cover_letter: string;
  created_at: string;
  updated_at: string;
}

export type AnswerKey = { type: "choice"; value: number } | { type: "number"; value: number; tolerance: number } | { type: "text"; value: string[] };

export interface MockItem {
  seq: number;
  template_id: string;
  level: number;
  topic: AssessmentTopic;
  title: string;
  kind: ItemKind;
  prompt: string;
  code: CodeBlock | null;
  options: string[];
  input_hint: string | null;
  key: AnswerKey;
  a: number;
  b: number;
  c: number;
  time_limit_secs: number;
  shown_at: string;
  deadline_at: string;
  answered: boolean;
  answer: Answer | null;
  correct: boolean | null;
  answered_at: string | null;
}

export interface MockAssessment {
  session: Omit<AssessmentSession, "answered" | "current">;
  candidate_id: string;
  items: MockItem[];
  posterior: number[];
  last_activity_at: string;
}

export interface MockTask {
  id: string;
  company_id: string;
  title: string;
  body: string;
  kind: TaskKind;
  specialization: Specialization;
  grades: Grade[];
  skills: string[];
  active: boolean;
  created_at: string;
}

export interface MockTaskAssignment {
  id: string;
  task_id: string;
  candidate_id: string;
  status: TaskStatus;
  answer: string | null;
  rating: number | null;
  review_comment: string | null;
  assigned_at: string;
  due_at: string;
  submitted_at: string | null;
  reviewed_at: string | null;
}

export interface MockDb {
  version: number;
  seeded_at: string;
  session: { user_id: string } | null;
  users: MockUser[];
  candidates: MockCandidate[];
  companies: MockCompany[];
  needs: MockNeed[];
  shortlists: Record<string, MockShortlist>;
  invitations: MockInvitation[];
  vacancies: MockVacancy[];
  applications: MockApplication[];
  assessments: MockAssessment[];
  tasks: MockTask[];
  task_assignments: MockTaskAssignment[];
  contact_log: { candidate_id: string; company_id: string; basis: "invitation" | "application"; at: string }[];
  reports: { id?: string; user_id?: string; target: string; target_id: string; reason: string; at: string }[];
}

const STORAGE_KEY = "fsp-mock-db";

let db: MockDb | null = null;

function load(): MockDb {
  if (typeof window !== "undefined") {
    try {
      const raw = window.localStorage.getItem(STORAGE_KEY);
      if (raw) {
        const parsed = JSON.parse(raw) as MockDb;
        if (parsed.version === SEED_VERSION) return parsed;
      }
    } catch {
      // Private mode or corrupted JSON: fall back to a fresh seed.
    }
  }
  return buildSeed(new Date());
}

export function getDb(): MockDb {
  db ??= load();
  return db;
}

export function persist(): void {
  if (typeof window === "undefined" || !db) return;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(db));
  } catch {
    // Storage full or blocked — the in-memory state still works for this tab.
  }
}

export function resetDb(): void {
  db = buildSeed(new Date());
  persist();
}
