/**
 * With NEXT_PUBLIC_USE_MOCKS=1 calls go to src/mocks/handlers.ts (same names and
 * signatures). The import is lazy so mock code stays out of real-API bundles.
 */
import { API_BASE, http, USE_MOCKS } from "./client";
import type * as T from "./types";

const mocks = () => import("@/mocks/handlers");

export async function getCatalog(): Promise<T.Catalog> {
  if (USE_MOCKS) return (await mocks()).getCatalog();
  return http("GET", "/catalog");
}

export async function searchSkills(q: string, specialization?: T.Specialization): Promise<T.SkillView[]> {
  if (USE_MOCKS) return (await mocks()).searchSkills(q, specialization);
  return http("GET", "/catalog/skills", { query: { q, specialization } });
}

export async function getCurrentConsents(): Promise<T.ConsentDocument[]> {
  if (USE_MOCKS) return (await mocks()).getCurrentConsents();
  return http("GET", "/consents/current");
}

export async function register(body: T.RegisterRequest): Promise<T.RegisterResponse> {
  if (USE_MOCKS) return (await mocks()).register(body);
  return http("POST", "/auth/register", { body });
}

export async function verifyEmail(token: string): Promise<void> {
  if (USE_MOCKS) return (await mocks()).verifyEmail(token);
  return http("POST", "/auth/verify-email", { body: { token } });
}

export async function resendVerification(email: string): Promise<void> {
  if (USE_MOCKS) return (await mocks()).resendVerification(email);
  return http("POST", "/auth/resend-verification", { body: { email } });
}

export async function login(body: T.LoginRequest): Promise<T.LoginResponse> {
  if (USE_MOCKS) return (await mocks()).login(body);
  return http("POST", "/auth/login", { body });
}

export async function logout(): Promise<void> {
  if (USE_MOCKS) return (await mocks()).logout();
  return http("POST", "/auth/logout");
}

export async function getMe(): Promise<T.Me> {
  if (USE_MOCKS) return (await mocks()).getMe();
  return http("GET", "/me");
}

export async function deleteMe(): Promise<void> {
  if (USE_MOCKS) return (await mocks()).deleteMe();
  return http("DELETE", "/me");
}

export async function getMyConsents(): Promise<T.UserConsent[]> {
  if (USE_MOCKS) return (await mocks()).getMyConsents();
  return http("GET", "/me/consents");
}

export async function revokeConsent(documentId: T.Uuid): Promise<void> {
  if (USE_MOCKS) return (await mocks()).revokeConsent(documentId);
  return http("POST", `/me/consents/${documentId}/revoke`);
}

/** Grants the current version; an already active grant comes back unchanged. */
export async function grantConsent(documentId: T.Uuid): Promise<T.UserConsent> {
  if (USE_MOCKS) return (await mocks()).grantConsent(documentId);
  const body: T.GrantConsent = { document_id: documentId };
  return http("POST", "/me/consents", { body });
}

/** Everything stored about the account as a JSON file (152-ФЗ, ст. 14). */
export async function exportMyData(): Promise<Blob> {
  if (USE_MOCKS) return (await mocks()).exportMyData();
  return http("GET", "/me/export", { blob: true });
}

export async function getContactAccess(): Promise<T.ContactAccessEntry[]> {
  if (USE_MOCKS) return (await mocks()).getContactAccess();
  return http("GET", "/candidate/contact-access");
}

export async function getProfile(): Promise<T.Profile> {
  if (USE_MOCKS) return (await mocks()).getProfile();
  return http("GET", "/candidate/profile");
}

export async function updateProfile(body: T.ProfileInput): Promise<T.Profile> {
  if (USE_MOCKS) return (await mocks()).updateProfile(body);
  return http("PUT", "/candidate/profile", { body });
}

export async function uploadResume(file: File): Promise<T.ResumeDraft> {
  if (USE_MOCKS) return (await mocks()).uploadResume(file);
  const form = new FormData();
  form.append("file", file);
  return http("POST", "/candidate/resume", { body: form });
}

export async function getProfilePdf(): Promise<Blob> {
  if (USE_MOCKS) return (await mocks()).getProfilePdf();
  return http("GET", "/candidate/profile.pdf", { blob: true, headers: { Accept: "application/pdf" } });
}

export async function getPrivacy(): Promise<T.Privacy> {
  if (USE_MOCKS) return (await mocks()).getPrivacy();
  return http("GET", "/candidate/privacy");
}

export async function updatePrivacy(body: T.Privacy): Promise<T.Privacy> {
  if (USE_MOCKS) return (await mocks()).updatePrivacy(body);
  return http("PUT", "/candidate/privacy", { body });
}

export async function submitSurvey(body: T.SurveyInput): Promise<T.SurveyResponse> {
  if (USE_MOCKS) return (await mocks()).submitSurvey(body);
  return http("POST", "/candidate/survey", { body });
}

export async function getSurveyHistory(): Promise<T.SurveyResponse[]> {
  if (USE_MOCKS) return (await mocks()).getSurveyHistory();
  return http("GET", "/candidate/survey/history");
}

export async function getCandidateCategory(): Promise<T.CategoryStatus> {
  if (USE_MOCKS) return (await mocks()).getCandidateCategory();
  return http("GET", "/candidate/category");
}

export async function startAssessment(body: T.StartInput): Promise<T.AssessmentSession> {
  if (USE_MOCKS) return (await mocks()).startAssessment(body);
  return http("POST", "/candidate/assessments", { body });
}

export async function listAssessments(): Promise<T.AssessmentSession[]> {
  if (USE_MOCKS) return (await mocks()).listAssessments();
  return http("GET", "/candidate/assessments");
}

export async function getAssessment(id: T.Uuid): Promise<T.AssessmentSession> {
  if (USE_MOCKS) return (await mocks()).getAssessment(id);
  return http("GET", `/candidate/assessments/${id}`);
}

/** Also advances the session when the current item timed out. */
export async function getCurrentItem(id: T.Uuid): Promise<T.AssessmentSession> {
  if (USE_MOCKS) return (await mocks()).getCurrentItem(id);
  return http("GET", `/candidate/assessments/${id}/current-item`);
}

export async function answerItem(id: T.Uuid, body: T.AnswerInput): Promise<T.AssessmentSession> {
  if (USE_MOCKS) return (await mocks()).answerItem(id, body);
  return http("POST", `/candidate/assessments/${id}/answer`, { body });
}

/** Mock mode has no camera backend: the proctoring step is skipped as with `PROCTORING=off`. */
export async function getProctoringOptions(): Promise<T.ProctoringOptions> {
  if (USE_MOCKS) return { mode: "off", consent_granted: false, retention_days: 90, max_chunk_bytes: 1 << 20, max_events_per_batch: 50 };
  return http("GET", "/candidate/proctoring");
}

export async function uploadProctoringChunk(id: T.Uuid, seq: number, chunk: Blob): Promise<void> {
  if (USE_MOCKS) return;
  return http("PUT", `/candidate/assessments/${id}/proctoring/chunks/${seq}`, {
    body: chunk,
    headers: { "Content-Type": chunk.type || "video/webm" },
  });
}

export async function postProctoringEvents(id: T.Uuid, events: T.IntegrityEventInput[], keepalive = false): Promise<void> {
  if (USE_MOCKS) return;
  return http("POST", `/candidate/assessments/${id}/proctoring/events`, { body: { events }, keepalive });
}

/** Played by <video src>, which sends the session cookie itself. */
export function proctoringRecordingUrl(id: T.Uuid): string {
  return `${API_BASE}/candidate/assessments/${id}/proctoring/recording`;
}

export async function listReceivedInvitations(): Promise<T.ReceivedInvitation[]> {
  if (USE_MOCKS) return (await mocks()).listReceivedInvitations();
  return http("GET", "/candidate/invitations");
}

export async function getReceivedInvitation(id: T.Uuid): Promise<T.ReceivedInvitation> {
  if (USE_MOCKS) return (await mocks()).getReceivedInvitation(id);
  return http("GET", `/candidate/invitations/${id}`);
}

export async function acceptInvitation(id: T.Uuid): Promise<T.ReceivedInvitation> {
  if (USE_MOCKS) return (await mocks()).acceptInvitation(id);
  return http("POST", `/candidate/invitations/${id}/accept`);
}

export async function declineInvitation(id: T.Uuid, body: T.DeclineRequest): Promise<T.ReceivedInvitation> {
  if (USE_MOCKS) return (await mocks()).declineInvitation(id, body);
  return http("POST", `/candidate/invitations/${id}/decline`, { body });
}

export async function getCandidateMarket(): Promise<T.MarketStats> {
  if (USE_MOCKS) return (await mocks()).getCandidateMarket();
  return http("GET", "/candidate/market");
}

export async function listMyApplications(): Promise<T.MyApplication[]> {
  if (USE_MOCKS) return (await mocks()).listMyApplications();
  return http("GET", "/candidate/applications");
}

export async function withdrawApplication(id: T.Uuid): Promise<void> {
  if (USE_MOCKS) return (await mocks()).withdrawApplication(id);
  return http("POST", `/candidate/applications/${id}/withdraw`);
}

export async function listMyTasks(): Promise<T.MyTask[]> {
  if (USE_MOCKS) return (await mocks()).listMyTasks();
  return http("GET", "/candidate/tasks");
}

export async function submitTask(id: T.Uuid, body: T.SubmitInput): Promise<T.MyTask[]> {
  if (USE_MOCKS) return (await mocks()).submitTask(id, body);
  return http("POST", `/candidate/tasks/${id}/submit`, { body });
}

export async function skipTask(id: T.Uuid): Promise<void> {
  if (USE_MOCKS) return (await mocks()).skipTask(id);
  return http("POST", `/candidate/tasks/${id}/skip`);
}

/** Navigated to, not fetched: the API answers with a redirect to Keycloak. */
export const FSP_LINK_START_URL = "/api/v1/fsp/link/start";

export async function fspLinkDev(fspId: string): Promise<T.FspStatus> {
  if (USE_MOCKS) return (await mocks()).fspLinkDev(fspId);
  return http("POST", "/fsp/link/dev", { body: { fsp_id: fspId } });
}

export async function getFspStatus(): Promise<T.FspStatus> {
  if (USE_MOCKS) return (await mocks()).getFspStatus();
  return http("GET", "/fsp/achievements");
}

export async function syncFsp(): Promise<T.FspStatus> {
  if (USE_MOCKS) return (await mocks()).syncFsp();
  return http("POST", "/fsp/sync");
}

export async function unlinkFsp(): Promise<void> {
  if (USE_MOCKS) return (await mocks()).unlinkFsp();
  return http("DELETE", "/fsp/link");
}

export async function getCompany(): Promise<T.Company> {
  if (USE_MOCKS) return (await mocks()).getCompany();
  return http("GET", "/employer/company");
}

export async function updateCompany(body: T.CompanyProfile): Promise<T.Company> {
  if (USE_MOCKS) return (await mocks()).updateCompany(body);
  return http("PUT", "/employer/company", { body });
}

export async function listNeeds(): Promise<T.HiringNeed[]> {
  if (USE_MOCKS) return (await mocks()).listNeeds();
  return http("GET", "/employer/needs");
}

export async function createNeed(body: T.NeedInput): Promise<T.HiringNeed> {
  if (USE_MOCKS) return (await mocks()).createNeed(body);
  return http("POST", "/employer/needs", { body });
}

export async function getNeed(id: T.Uuid): Promise<T.HiringNeed> {
  if (USE_MOCKS) return (await mocks()).getNeed(id);
  return http("GET", `/employer/needs/${id}`);
}

export async function updateNeed(id: T.Uuid, body: T.NeedInput): Promise<T.HiringNeed> {
  if (USE_MOCKS) return (await mocks()).updateNeed(id, body);
  return http("PUT", `/employer/needs/${id}`, { body });
}

export async function deleteNeed(id: T.Uuid): Promise<void> {
  if (USE_MOCKS) return (await mocks()).deleteNeed(id);
  return http("DELETE", `/employer/needs/${id}`);
}

export async function parseNeed(text: string): Promise<T.ParsedNeed> {
  if (USE_MOCKS) return (await mocks()).parseNeed(text);
  return http("POST", "/employer/needs/parse", { body: { text } });
}

export async function getNeedCategories(id: T.Uuid): Promise<T.NeedCategory[]> {
  if (USE_MOCKS) return (await mocks()).getNeedCategories(id);
  return http("GET", `/employer/needs/${id}/categories`);
}

export async function refineShortlist(id: T.Uuid, filters: T.ShortlistFilters): Promise<T.Shortlist> {
  if (USE_MOCKS) return (await mocks()).refineShortlist(id, filters);
  return http("POST", `/employer/needs/${id}/shortlist`, { body: filters });
}

export async function getShortlist(id: T.Uuid): Promise<T.Shortlist> {
  if (USE_MOCKS) return (await mocks()).getShortlist(id);
  return http("GET", `/employer/needs/${id}/shortlist`);
}

export async function patchShortlistItem(id: T.Uuid, candidateId: T.Uuid, body: T.ShortlistPatch): Promise<void> {
  if (USE_MOCKS) return (await mocks()).patchShortlistItem(id, candidateId, body);
  return http("PATCH", `/employer/needs/${id}/shortlist/${candidateId}`, { body });
}

export async function getNeedTestPlan(id: T.Uuid): Promise<T.TestPlanDemo> {
  if (USE_MOCKS) return (await mocks()).getNeedTestPlan(id);
  return http("GET", `/employer/needs/${id}/test-plan`);
}

export async function getTestPlan(params: { specialization: T.Specialization; grade: T.Grade; skills?: string }): Promise<T.TestPlanDemo> {
  if (USE_MOCKS) return (await mocks()).getTestPlan(params);
  return http("GET", "/catalog/test-plan", { query: params });
}

export async function listCategories(): Promise<T.CategoryCard[]> {
  if (USE_MOCKS) return (await mocks()).listCategories();
  return http("GET", "/employer/categories");
}

export async function searchCandidates(params: T.CandidateSearchParams): Promise<T.Page<T.BankItem>> {
  if (USE_MOCKS) return (await mocks()).searchCandidates(params);
  return http("GET", "/employer/candidates", { query: { ...params } });
}

export async function getCandidateForEmployer(id: T.Uuid, needId?: T.Uuid | null): Promise<T.CandidateProfileView> {
  if (USE_MOCKS) return (await mocks()).getCandidateForEmployer(id, needId);
  return http("GET", `/employer/candidates/${id}`, { query: { need_id: needId ?? undefined } });
}

export async function getCandidateContacts(id: T.Uuid): Promise<T.CandidateContacts> {
  if (USE_MOCKS) return (await mocks()).getCandidateContacts(id);
  return http("GET", `/employer/candidates/${id}/contacts`);
}

export async function listSentInvitations(): Promise<T.SentInvitation[]> {
  if (USE_MOCKS) return (await mocks()).listSentInvitations();
  return http("GET", "/employer/invitations");
}

export async function sendInvitation(body: T.SendInvitation, idempotencyKey: string): Promise<T.SentInvitation> {
  if (USE_MOCKS) return (await mocks()).sendInvitation(body, idempotencyKey);
  return http("POST", "/employer/invitations", { body, headers: { "Idempotency-Key": idempotencyKey } });
}

export async function getSentInvitation(id: T.Uuid): Promise<T.SentInvitation> {
  if (USE_MOCKS) return (await mocks()).getSentInvitation(id);
  return http("GET", `/employer/invitations/${id}`);
}

export async function withdrawInvitation(id: T.Uuid): Promise<void> {
  if (USE_MOCKS) return (await mocks()).withdrawInvitation(id);
  return http("POST", `/employer/invitations/${id}/withdraw`);
}

export async function getInvitationStats(): Promise<T.InvitationStats> {
  if (USE_MOCKS) return (await mocks()).getInvitationStats();
  return http("GET", "/employer/invitations/stats");
}

export async function listOwnVacancies(): Promise<T.Vacancy[]> {
  if (USE_MOCKS) return (await mocks()).listOwnVacancies();
  return http("GET", "/employer/vacancies");
}

export async function createVacancy(body: T.VacancyInput): Promise<T.Vacancy> {
  if (USE_MOCKS) return (await mocks()).createVacancy(body);
  return http("POST", "/employer/vacancies", { body });
}

export async function getOwnVacancy(id: T.Uuid): Promise<T.Vacancy> {
  if (USE_MOCKS) return (await mocks()).getOwnVacancy(id);
  return http("GET", `/employer/vacancies/${id}`);
}

export async function updateVacancy(id: T.Uuid, body: T.VacancyInput): Promise<T.Vacancy> {
  if (USE_MOCKS) return (await mocks()).updateVacancy(id, body);
  return http("PUT", `/employer/vacancies/${id}`, { body });
}

export async function listVacancyApplications(id: T.Uuid): Promise<T.IncomingApplication[]> {
  if (USE_MOCKS) return (await mocks()).listVacancyApplications(id);
  return http("GET", `/employer/vacancies/${id}/applications`);
}

export async function setApplicationStatus(id: T.Uuid, status: T.EmployerApplicationStatus): Promise<void> {
  if (USE_MOCKS) return (await mocks()).setApplicationStatus(id, status);
  return http("POST", `/employer/applications/${id}/status`, { body: { status } });
}

export async function listEmployerTasks(): Promise<T.EmployerTask[]> {
  if (USE_MOCKS) return (await mocks()).listEmployerTasks();
  return http("GET", "/employer/tasks");
}

export async function createEmployerTask(body: T.TaskInput): Promise<T.EmployerTask> {
  if (USE_MOCKS) return (await mocks()).createEmployerTask(body);
  return http("POST", "/employer/tasks", { body });
}

export async function setTaskActive(id: T.Uuid, active: boolean): Promise<void> {
  if (USE_MOCKS) return (await mocks()).setTaskActive(id, active);
  return http("POST", `/employer/tasks/${id}/active`, { body: { active } });
}

export async function listTaskSubmissions(id: T.Uuid): Promise<T.TaskSubmission[]> {
  if (USE_MOCKS) return (await mocks()).listTaskSubmissions(id);
  return http("GET", `/employer/tasks/${id}/submissions`);
}

export async function reviewTaskSubmission(id: T.Uuid, submissionId: T.Uuid, body: T.ReviewInput): Promise<void> {
  if (USE_MOCKS) return (await mocks()).reviewTaskSubmission(id, submissionId, body);
  return http("POST", `/employer/tasks/${id}/submissions/${submissionId}/review`, { body });
}

export async function listVacancies(params: T.VacancyQuery = {}): Promise<T.Page<T.PublicVacancy>> {
  if (USE_MOCKS) return (await mocks()).listVacancies(params);
  return http("GET", "/vacancies", { query: { ...params } });
}

export async function getVacancy(id: T.Uuid): Promise<T.PublicVacancy> {
  if (USE_MOCKS) return (await mocks()).getVacancy(id);
  return http("GET", `/vacancies/${id}`);
}

export async function applyToVacancy(id: T.Uuid, body: T.ApplyInput): Promise<T.MyApplication> {
  if (USE_MOCKS) return (await mocks()).applyToVacancy(id, body);
  return http("POST", `/vacancies/${id}/applications`, { body });
}

export async function createReport(body: T.ReportInput): Promise<void> {
  if (USE_MOCKS) return (await mocks()).createReport(body);
  return http("POST", "/reports", { body });
}
