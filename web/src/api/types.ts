/**
 * Friendly names over the generated OpenAPI schema (src/api/schema.d.ts).
 * Regenerate with `make gen-api` (or `npm run gen:api`); never describe response shapes by hand.
 */
import type { components, operations } from "./schema";

type S = components["schemas"];

export type Uuid = string;

export interface Page<T> {
  items: T[];
  total: number;
}

export type Role = S["Role"];
export type Direction = S["Direction"];
export type Specialization = S["Specialization"];
export type Grade = S["Grade"];
export type WorkFormat = S["WorkFormat"];
export type Industry = S["Industry"];
export type Category = S["Category"];
export type SalaryRange = S["SalaryRange"];
export type SearchStatus = S["SearchStatus"];
export type SkillSource = S["SkillSource"];

export type Catalog = S["CatalogResponse"];
export type DirectionInfo = S["DirectionInfo"];
export type SkillView = S["SkillView"];
export type ConsentDocument = S["ConsentDocument"];
export type UserConsent = S["UserConsent"];
export type GrantConsent = S["GrantConsent"];
export type ContactAccessEntry = S["ContactAccessEntry"];

export type SessionUser = S["SessionUser"];
export type Me = S["MeResponse"];
export type MeCandidate = S["CandidateSummary"];
export type OnboardingStep = S["OnboardingStep"];
export type RegisterRequest = S["RegisterRequest"];
export type RegisterResponse = S["RegisterResponse"];
export type LoginRequest = S["LoginRequest"];
export type LoginResponse = S["LoginResponse"];

export type Contacts = S["Contacts"];
export type ExperienceEntry = S["ExperienceEntry"];
export type ProfileInput = S["ProfileInput"];
export type ProfileSkill = S["ProfileSkill"];
export type Profile = S["ProfileResponse"];
export type ResumeDraft = S["ResumeDraft"];
export type ParsedResume = S["ParsedResume"];
export type Privacy = S["Privacy"];
export type SurveyInput = S["SurveyInput"];
export type SurveyResponse = S["SurveyResponse"];

export type AssessmentTopic = S["Topic"];
export type AttemptKind = S["AttemptKind"];
export type Offer = S["Offer"];
export type GradeHistoryEntry = S["GradeHistoryEntry"];
export type CategoryStatus = S["CategoryStatus"];
export type StartInput = S["StartInput"];
export type AssessmentStatus = S["AssessmentStatus"];
export type AssessmentSession = S["AssessmentSession"];
export type AssessmentResult = S["AssessmentResult"];
export type Decision = S["Decision"];
export type TopicResult = S["TopicResult"];
export type CurrentItem = S["CurrentItem"];
export type ItemView = S["ItemView"];
export type ItemKind = S["ItemKind"];
export type CodeBlock = S["CodeBlock"];
export type Answer = S["Answer"];
export type AnswerInput = S["AnswerInput"];
export type MarketStats = S["MarketStats"];

export type ProctoringMode = S["ProctoringMode"];
export type ProctoringOptions = S["ProctoringOptions"];
export type ProctoringChoice = S["ProctoringChoice"];
export type ProctoringStatus = S["ProctoringStatus"];
export type ProctoringSummary = S["ProctoringSummary"];
export type ProctoringBadge = S["ProctoringBadge"];
export type IntegrityEvent = S["IntegrityEvent"];
export type IntegrityEventInput = S["EventInput"];
export type EventCounts = S["EventCountsView"];
export type TestPlanDemo = S["TestPlanDemo"];
export type TestPlan = S["TestPlan"];

export type InvitationStatus = S["InvitationStatus"];
export type DeclineReason = S["DeclineReason"];
export type StatusEvent = S["StatusEvent"];
export type ReceivedInvitation = S["ReceivedInvitation"];
export type DeclineRequest = S["DeclineRequest"];
export type SendInvitation = S["SendInvitation"];
export type SentInvitation = S["SentInvitation"];
export type InvitationStats = S["InvitationStats"];

export type ApplicationStatus = S["ApplicationStatus"];
export type MyApplication = S["MyApplication"];
export type ApplyInput = S["ApplyInput"];
export type IncomingApplication = S["IncomingApplication"];
export type EmployerApplicationStatus = Exclude<ApplicationStatus, "sent" | "withdrawn">;

export type TaskKind = S["TaskKind"];
export type TaskStatus = S["TaskStatus"];
export type MyTask = S["MyTask"];
export type SubmitInput = S["SubmitInput"];
export type TaskInput = S["TaskInput"];
export type EmployerTask = S["EmployerTask"];
export type TaskSubmission = S["Submission"];
export type ReviewInput = S["ReviewInput"];

export type FspDiscipline = S["Discipline"];
export type FspRank = S["Rank"];
export type FspEventLevel = S["EventLevel"];
export type FspPlace = S["Place"];
export type FspAchievement = S["AchievementView"];
export type FspScore = S["FspScore"];
export type FspStatus = S["FspStatus"];

export type CompanyProfile = S["CompanyProfile"];
export type Company = S["CompanyResponse"];

export type NeedStatus = S["NeedStatus"];
export type NeedInput = S["NeedInput"];
export type HiringNeed = S["HiringNeed"];
export type ParsedNeed = S["ParsedNeed"];
export type NeedCategory = S["CategoryStats"];
export type CategoryCard = S["CategoryCard"];
export type ShortlistFilters = S["ShortlistFilters"];
export type ShortlistItem = S["ShortlistItem"];
export type Shortlist = S["Shortlist"];
export type ShortlistPatch = S["ShortlistPatch"];
export type FeatureKey = S["FeatureKey"];
export type FeatureScore = S["FeatureScore"];
export type ScoreBreakdown = S["ScoreBreakdown"];
export type SkillMatch = S["SkillMatch"];

export type CandidateCard = S["CandidateCard"];
export type CardSkill = S["CardSkill"];
export type CardFsp = S["CardFsp"];
export type CardAchievement = S["CardAchievement"];
export type CardExperience = S["CardExperience"];
export type Relation = S["Relation"];
export type BankItem = S["BankItem"];
export type CandidateProfileView = S["CandidateProfileView"];
export type CandidateContacts = S["CandidateContacts"];
export type ContactBasis = S["ContactBasis"];

export type VacancyStatus = S["VacancyStatus"];
export type VacancyInput = S["VacancyInput"];
export type Vacancy = S["Vacancy"];
export type PublicVacancy = S["PublicVacancy"];

export type ReportInput = S["ReportInput"];
export type ReportReason = S["ReportReason"];

export type CandidateSearchParams = NonNullable<operations["search"]["parameters"]["query"]>;
export type VacancyQuery = NonNullable<operations["list_published"]["parameters"]["query"]>;

export type AccessChannel = S["AccessChannel"];
export type HiringStage = S["HiringStage"];
export type HiringOutcome = S["HiringOutcome"];
export type StageCount = S["StageCount"];
export type IntegrationsOverview = S["IntegrationsOverview"];
export type WebhookView = S["WebhookView"];
export type WebhookInput = S["WebhookInput"];
export type WebhookSaved = S["WebhookSaved"];
export type TestDelivery = S["TestDelivery"];
export type DeliveryView = S["DeliveryView"];
export type DeliveryStatus = S["DeliveryStatus"];
export type DeliveryAttempt = S["AttemptView"];
export type IntegrationEventType = S["EventType"];
export type ApiScope = S["Scope"];
export type ApiKeyView = S["ApiKeyView"];
export type ApiKeyInput = S["ApiKeyInput"];
export type ApiKeyCreated = S["ApiKeyCreated"];
