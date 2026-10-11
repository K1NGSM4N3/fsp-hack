"use client";

import { BadgeCheck, Ban, Clock3, EyeOff, FileText, Flag, Gauge, GraduationCap, Inbox, Mail, MailX, RotateCcw, Scale, ShieldCheck, Snowflake, Trophy, UserCheck, UserX, Video, Webhook, type LucideIcon } from "lucide-react";
import Link from "next/link";
import type { NotificationKind, NotificationView } from "@/api/types";
import { cn } from "@/lib/cn";
import { formatDateShort, formatDateTime } from "@/lib/format";

const ICON: Record<NotificationKind, LucideIcon> = {
  invitation_received: Mail,
  invitation_expiring: Clock3,
  invitation_withdrawn: MailX,
  hiring_stage: Flag,
  application_status: FileText,
  assessment_result: GraduationCap,
  guardian_decision: ShieldCheck,
  invitation_accepted: UserCheck,
  invitation_declined: UserX,
  application_received: Inbox,
  webhook_failing: Webhook,
  trust_changed: Gauge,
  result_confirmed: BadgeCheck,
  result_annulled: Ban,
  retest_ordered: Video,
  vacancy_unpublished: EyeOff,
  invitations_frozen: Snowflake,
  sanction_lifted: RotateCcw,
  complaint_resolved: Scale,
  fsp_achievements: Trophy,
};

const MINUTE = 60_000;

/** «только что», «12 мин назад», «3 ч назад», «вчера», then the date. */
export function formatAgo(iso: string, now: number = Date.now()): string {
  const diff = now - new Date(iso).getTime();
  if (diff < MINUTE) return "только что";
  if (diff < 60 * MINUTE) return `${Math.floor(diff / MINUTE)} мин назад`;
  if (diff < 24 * 60 * MINUTE) return `${Math.floor(diff / (60 * MINUTE))} ч назад`;
  if (diff < 48 * 60 * MINUTE) return "вчера";
  return formatDateShort(iso);
}

interface RowProps {
  n: NotificationView;
  onOpen: () => void;
  /** The bell's dropdown: two lines of text at most. */
  compact?: boolean;
}

export function NotificationRow({ n, onOpen, compact = false }: RowProps) {
  const unread = !n.read_at;
  const Icon = ICON[n.kind];
  const body = (
    <>
      <span className={cn("relative mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-full", unread ? "bg-primary-50 text-primary" : "bg-canvas text-ink-subtle")} aria-hidden>
        <Icon className="size-4" />
        {unread && <span className="absolute -top-0.5 -right-0.5 size-2.5 rounded-full border-2 border-white bg-accent" />}
      </span>
      <span className="min-w-0 flex-1">
        <span className={cn("block text-sm leading-snug", unread ? "font-semibold text-ink" : "font-medium text-ink-muted")}>{n.title}</span>
        <span className={cn("mt-0.5 block text-sm leading-relaxed text-ink-muted", compact && "line-clamp-2")}>{n.body}</span>
        <time dateTime={n.created_at} title={formatDateTime(n.created_at)} className="mt-1 block text-xs text-ink-subtle">
          {formatAgo(n.created_at)}
        </time>
      </span>
      {unread && <span className="sr-only">Не прочитано</span>}
    </>
  );
  const cls = cn("flex w-full gap-3 text-left transition-colors hover:bg-canvas", compact ? "px-4 py-3" : "px-4 py-4 sm:px-5", unread && "bg-primary-50/40");
  return n.link ? (
    <Link href={n.link} onClick={onOpen} className={cls}>
      {body}
    </Link>
  ) : (
    <button type="button" onClick={onOpen} className={cls}>
      {body}
    </button>
  );
}
