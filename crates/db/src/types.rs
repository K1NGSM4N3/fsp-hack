use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A keyset position: the sort time and the id that breaks ties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub at: DateTime<Utc>,
    pub id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    Candidate,
    Employer,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "search_status", rename_all = "snake_case")]
pub enum SearchStatus {
    Active,
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "assessment_status", rename_all = "snake_case")]
pub enum AssessmentStatus {
    InProgress,
    Passed,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "vacancy_status", rename_all = "snake_case")]
pub enum VacancyStatus {
    Draft,
    Published,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "application_status", rename_all = "snake_case")]
pub enum ApplicationStatus {
    Sent,
    Viewed,
    Accepted,
    Rejected,
    Withdrawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "invitation_status", rename_all = "snake_case")]
pub enum InvitationStatus {
    Sent,
    Viewed,
    Accepted,
    Declined,
    Expired,
    Withdrawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "task_assignment_status", rename_all = "snake_case")]
pub enum TaskAssignmentStatus {
    Assigned,
    Submitted,
    Reviewed,
    Skipped,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "integrity_event", rename_all = "snake_case")]
pub enum IntegrityEvent {
    TabHidden,
    WindowBlur,
    FullscreenExit,
    Paste,
    CameraLost,
    RecordingFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "age_bracket", rename_all = "snake_case")]
pub enum AgeBracket {
    Adult,
    Minor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "guardian_decision", rename_all = "snake_case")]
pub enum GuardianDecision {
    Granted,
    Declined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "integration_event_status", rename_all = "snake_case")]
pub enum IntegrationEventStatus {
    Pending,
    Delivered,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "hiring_stage", rename_all = "snake_case")]
pub enum HiringStage {
    Interview,
    Offer,
    Hired,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "review_decision", rename_all = "snake_case")]
pub enum ReviewDecision {
    Confirm,
    Annul,
    Retest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "moderation_action", rename_all = "snake_case")]
pub enum ModerationAction {
    ConfirmResult,
    AnnulResult,
    OrderRetest,
    ViewRecording,
    Dismiss,
    UnpublishVacancy,
    FreezeInvitations,
    Restore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "moderation_target", rename_all = "snake_case")]
pub enum ModerationTarget {
    Session,
    Vacancy,
    Invitation,
    Company,
    Complaint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "flag_review", rename_all = "snake_case")]
pub enum FlagReview {
    Dismissed,
    Upheld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "notification_email_status", rename_all = "snake_case")]
pub enum NotificationEmailStatus {
    Pending,
    Sent,
    Failed,
    Skipped,
}
