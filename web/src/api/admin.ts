/**
 * Moderator's cabinet (`/api/v1/admin/*`, tag «Модерация»). Types come from the generated schema;
 * the mock API has no moderator, so in mock mode every call fails with a clear message.
 */
import { API_BASE, ApiError, http, USE_MOCKS } from "./client";
import type { components, operations } from "./schema";

type S = components["schemas"];

export type ModerationOverview = S["ModerationOverview"];
export type QueuePage = S["QueuePage"];
export type QueueSession = S["QueueSession"];
export type QueueFilter = NonNullable<operations["proctoring_queue"]["parameters"]["query"]>["filter"] & string;
export type SessionReview = S["SessionReview"];
export type ReviewAnswer = S["ReviewAnswer"];
export type ReviewEvent = S["ReviewEvent"];
export type ReviewDecision = S["ReviewDecision"];
export type VideoState = S["VideoState"];
export type TrustPage = S["TrustPage"];
export type TrustItem = S["TrustItem"];
export type CompanyRef = S["CompanyRef"];
export type TrustItemKind = S["TrustItemKind"];
export type TrustAction = S["TrustAction"];
export type TrustView = NonNullable<operations["trust_queue"]["parameters"]["query"]>["view"] & string;
export type TrustActionInput = S["TrustActionInput"];
export type TrustActionResult = S["TrustActionResult"];
export type AuditPage = S["AuditPage"];
export type AuditEntry = S["AuditEntry"];
export type ModerationAction = S["ModerationAction"];
export type ModerationTarget = S["ModerationTarget"];

function mockless(): never {
  throw new ApiError({
    status: 503,
    code: "mock_unsupported",
    title: "Кабинет модератора работает только с настоящим API",
    detail: "Запустите API с SEED_DEMO=true и войдите как moderator@demo.fsp",
  });
}

export async function getOverview(): Promise<ModerationOverview> {
  if (USE_MOCKS) mockless();
  return http("GET", "/admin/overview");
}

export async function listProctoringQueue(filter: QueueFilter, cursor?: string | null): Promise<QueuePage> {
  if (USE_MOCKS) mockless();
  return http("GET", "/admin/proctoring", { query: { filter, cursor, limit: 30 } });
}

export async function getSessionReview(sessionId: string): Promise<SessionReview> {
  if (USE_MOCKS) mockless();
  return http("GET", `/admin/proctoring/${sessionId}`);
}

/** Playing it is logged and the candidate sees the date; `HEAD` only checks the file is there. */
export function adminRecordingUrl(sessionId: string): string {
  return `${API_BASE}/admin/proctoring/${sessionId}/recording`;
}

export async function decideSession(sessionId: string, decision: ReviewDecision, comment: string): Promise<SessionReview> {
  if (USE_MOCKS) mockless();
  return http("POST", `/admin/proctoring/${sessionId}/decision`, { body: { decision, comment } });
}

export async function listTrustQueue(view: TrustView, kind: TrustItemKind | null, cursor?: string | null): Promise<TrustPage> {
  if (USE_MOCKS) mockless();
  return http("GET", "/admin/antifake", { query: { view, kind, cursor, limit: 30 } });
}

export async function actOnTrustItem(body: TrustActionInput): Promise<TrustActionResult> {
  if (USE_MOCKS) mockless();
  return http("POST", "/admin/antifake/actions", { body });
}

export interface AuditFilter {
  action?: ModerationAction | null;
  target_kind?: ModerationTarget | null;
  subject?: string | null;
}

export async function listAudit(filter: AuditFilter, cursor?: string | null): Promise<AuditPage> {
  if (USE_MOCKS) mockless();
  return http("GET", "/admin/log", { query: { ...filter, cursor, limit: 30 } });
}
