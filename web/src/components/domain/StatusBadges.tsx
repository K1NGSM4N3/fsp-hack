import type { ApplicationStatus, AssessmentStatus, InvitationStatus, TaskStatus, VacancyStatus } from "@/api/types";
import { Badge, type BadgeTone } from "@/components/ui";
import {
  APPLICATION_STATUS_LABEL,
  ASSESSMENT_STATUS_LABEL,
  INVITATION_STATUS_LABEL,
  RECEIVED_INVITATION_STATUS_LABEL,
  TASK_STATUS_LABEL,
  VACANCY_STATUS_LABEL,
} from "@/lib/labels";

const INVITATION_TONE: Record<InvitationStatus, BadgeTone> = {
  sent: "primary",
  viewed: "warning",
  accepted: "success",
  declined: "danger",
  expired: "muted",
  withdrawn: "muted",
};

export function InvitationStatusBadge({ status, audience = "employer" }: { status: InvitationStatus; audience?: "employer" | "candidate" }) {
  const label = audience === "candidate" ? RECEIVED_INVITATION_STATUS_LABEL[status] : INVITATION_STATUS_LABEL[status];
  const tone = audience === "candidate" && status === "sent" ? "solid" : INVITATION_TONE[status];
  return (
    <Badge tone={tone} dot={tone !== "solid"}>
      {label}
    </Badge>
  );
}

const APPLICATION_TONE: Record<ApplicationStatus, BadgeTone> = { sent: "primary", viewed: "warning", accepted: "success", rejected: "danger", withdrawn: "muted" };

export function ApplicationStatusBadge({ status }: { status: ApplicationStatus }) {
  return (
    <Badge tone={APPLICATION_TONE[status]} dot>
      {APPLICATION_STATUS_LABEL[status]}
    </Badge>
  );
}

const ASSESSMENT_TONE: Record<AssessmentStatus, BadgeTone> = { in_progress: "primary", passed: "success", failed: "danger", expired: "muted" };

export function AssessmentStatusBadge({ status }: { status: AssessmentStatus }) {
  return (
    <Badge tone={ASSESSMENT_TONE[status]} dot>
      {ASSESSMENT_STATUS_LABEL[status]}
    </Badge>
  );
}

const VACANCY_TONE: Record<VacancyStatus, BadgeTone> = { draft: "muted", published: "success", closed: "neutral" };

export function VacancyStatusBadge({ status }: { status: VacancyStatus }) {
  return (
    <Badge tone={VACANCY_TONE[status]} dot>
      {VACANCY_STATUS_LABEL[status]}
    </Badge>
  );
}

const TASK_TONE: Record<TaskStatus, BadgeTone> = { assigned: "primary", submitted: "warning", reviewed: "success", skipped: "muted", expired: "muted" };

export function TaskStatusBadge({ status }: { status: TaskStatus }) {
  return (
    <Badge tone={TASK_TONE[status]} dot>
      {TASK_STATUS_LABEL[status]}
    </Badge>
  );
}
