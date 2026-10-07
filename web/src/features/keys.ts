import type { CandidateSearchParams, SalaryCheckQuery, VacancyQuery } from "@/api/types";

/**
 * First page of the public vacancy list; the server renders the same page for crawlers and hydrates the client with
 * it. Six cards keep the `/vacancies` document inside the first TCP round trip (about 14 KB compressed) on phones.
 */
export const VACANCY_PAGE_SIZE = 6;

export const qk = {
  me: ["me"] as const,
  catalog: ["catalog"] as const,
  skills: (q: string) => ["skills", "search", q] as const,
  typicalSkills: (specialization: string) => ["skills", "typical", specialization] as const,
  consents: ["consents", "current"] as const,
  myConsents: ["consents", "mine"] as const,

  profile: ["candidate", "profile"] as const,
  privacy: ["candidate", "privacy"] as const,
  surveys: ["candidate", "surveys"] as const,
  category: ["candidate", "category"] as const,
  assessments: ["candidate", "assessments"] as const,
  assessment: (id: string) => ["candidate", "assessments", id] as const,
  received: ["candidate", "invitations"] as const,
  receivedLists: ["candidate", "invitations", "list"] as const,
  receivedPages: (status: string) => ["candidate", "invitations", "list", "pages", status] as const,
  receivedSummary: ["candidate", "invitations", "list", "summary"] as const,
  receivedOne: (id: string) => ["candidate", "invitations", id] as const,
  market: ["candidate", "market"] as const,
  myApplications: ["candidate", "applications"] as const,
  myTasks: ["candidate", "tasks"] as const,
  fsp: ["candidate", "fsp"] as const,
  contactAccess: ["candidate", "contact-access"] as const,
  guardian: ["candidate", "guardian"] as const,
  guardianConsent: (token: string) => ["guardian", "consent", token] as const,

  company: ["employer", "company"] as const,
  needs: ["employer", "needs"] as const,
  need: (id: string) => ["employer", "needs", id] as const,
  needCategories: (id: string) => ["employer", "needs", id, "categories"] as const,
  needTestPlan: (id: string) => ["employer", "needs", id, "test-plan"] as const,
  shortlist: (id: string) => ["employer", "needs", id, "shortlist"] as const,
  categories: ["employer", "categories"] as const,
  candidateSearch: (p: CandidateSearchParams) => ["employer", "candidates", p] as const,
  candidate: (id: string, needId?: string | null) => ["employer", "candidate", id, needId ?? null] as const,
  contacts: (id: string) => ["employer", "candidate", id, "contacts"] as const,
  sent: ["employer", "invitations"] as const,
  sentPages: (status: string) => ["employer", "invitations", "list", status] as const,
  sentOne: (id: string) => ["employer", "invitations", id] as const,
  stats: ["employer", "invitations", "stats"] as const,
  ownVacancies: ["employer", "vacancies"] as const,
  ownVacancy: (id: string) => ["employer", "vacancies", id] as const,
  applications: (vacancyId: string) => ["employer", "vacancies", vacancyId, "applications"] as const,
  tasks: ["employer", "tasks"] as const,
  trust: ["employer", "trust"] as const,
  salaryCheck: (q: SalaryCheckQuery) => ["employer", "salary-check", q] as const,
  submissions: (taskId: string) => ["employer", "tasks", taskId, "submissions"] as const,
  integrations: ["employer", "integrations"] as const,
  deliveries: ["employer", "integrations", "deliveries"] as const,
  deliveryPages: (status: string) => ["employer", "integrations", "deliveries", status] as const,
  apiKeys: ["employer", "integrations", "keys"] as const,

  notifications: ["notifications"] as const,
  notificationFeed: (unread: boolean) => ["notifications", "feed", unread] as const,
  notificationsLatest: ["notifications", "latest"] as const,
  unreadCount: ["notifications", "unread"] as const,
  notificationSettings: ["notification-settings"] as const,

  vacancies: (p: VacancyQuery) => ["vacancies", p] as const,
  vacancy: (id: string) => ["vacancies", "one", id] as const,
  similarVacancies: (id: string) => ["vacancies", "similar", id] as const,
};

/**
 * Freshness per kind of data. Reference data and documents change only with a deploy. Account settings change only
 * through this UI, where mutations write the new value into the cache. Lists the other side changes (invitations,
 * replies, applications) go stale quickly and also refresh when the tab regains focus.
 */
export const fresh = {
  reference: { staleTime: Infinity },
  account: { staleTime: 5 * 60_000 },
  detail: { staleTime: 60_000 },
  live: { staleTime: 20_000, refetchOnWindowFocus: true },
} as const;
