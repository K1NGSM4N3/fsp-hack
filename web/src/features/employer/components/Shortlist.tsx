"use client";

import { EyeOff, Pin, PinOff, Send, SlidersHorizontal } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import type { Grade, HiringNeed, ShortlistFilters, ShortlistItem, WorkFormat } from "@/api/types";
import { CandidateCardView } from "@/components/domain/CandidateCard";
import { ExplanationPanel, ScoreRing } from "@/components/domain/Explanation";
import { SkillInput } from "@/components/domain/Skills";
import { InvitationStatusBadge } from "@/components/domain/StatusBadges";
import { Badge, Button, Card, Checkbox, ChipSelect, Field, Input, MoneyInput, Switch } from "@/components/ui";
import { ProctoringBadge } from "./ProctoringBadge";
import { cn } from "@/lib/cn";
import { GRADE_LABEL, GRADES, WORK_FORMAT_LABEL, WORK_FORMATS } from "@/lib/labels";

interface ShortlistCardProps {
  item: ShortlistItem;
  need: HiringNeed;
  onPin: (pinned: boolean) => void;
  onDismiss: () => void;
  onInvite: () => void;
  busy?: boolean;
}

export function ShortlistCard({ item, need, onPin, onDismiss, onInvite, busy }: ShortlistCardProps) {
  const c = item.candidate;
  const status = c.relation.invitation_status ?? null;
  const activeInvite = status === "sent" || status === "viewed";
  return (
    <CandidateCardView
      candidate={c}
      href={`/employer/candidates/${c.id}?need=${need.id}`}
      highlightSkills={[...need.must_skills, ...need.nice_skills]}
      className={cn(item.is_new && "border-primary-300 ring-1 ring-primary-100", item.pinned && "border-primary-200")}
      aside={<ScoreRing score={item.score} />}
      badges={
        <>
          {item.is_new && <Badge tone="solid">Новый</Badge>}
          {item.pinned && (
            <Badge tone="primary" icon={<Pin className="size-3" aria-hidden />}>
              Закреплён
            </Badge>
          )}
          {!item.in_current && <Badge tone="muted">Вне текущих условий</Badge>}
          {status && <InvitationStatusBadge status={status} />}
          {c.relation.applied && <Badge tone="success">Откликнулся</Badge>}
          <ProctoringBadge badge={c.proctoring} />
        </>
      }
      footer={
        <>
          <Button size="sm" variant={item.pinned ? "soft" : "secondary"} icon={item.pinned ? <PinOff className="size-4" aria-hidden /> : <Pin className="size-4" aria-hidden />} onClick={() => onPin(!item.pinned)} disabled={busy}>
            {item.pinned ? "Открепить" : "Закрепить"}
          </Button>
          <Button size="sm" variant="ghost" icon={<EyeOff className="size-4" aria-hidden />} onClick={onDismiss} disabled={busy}>
            Скрыть
          </Button>
          <span className="flex-1" />
          <Link href={`/employer/candidates/${c.id}?need=${need.id}`} className="rounded px-2 text-sm font-medium text-primary hover:underline">
            Профиль
          </Link>
          <Button size="sm" icon={<Send className="size-4" aria-hidden />} onClick={onInvite} disabled={activeInvite || status === "accepted"}>
            {activeInvite ? "Приглашение отправлено" : status === "accepted" ? "Принял приглашение" : "Пригласить"}
          </Button>
        </>
      }
    >
      <div className="mt-4 border-t border-line pt-4">
        <ExplanationPanel breakdown={item.breakdown} />
      </div>
    </CandidateCardView>
  );
}

interface Draft {
  grades: Grade[];
  must_skills: string[];
  nice_skills: string[];
  work_formats: WorkFormat[];
  allow_adjacent_grades: boolean;
  city: string;
  fsp_only: boolean;
  max_expectation: number | null;
}

function draftOf(need: HiringNeed): Draft {
  const f = need.filters;
  return {
    grades: f.grades ?? need.grades,
    allow_adjacent_grades: f.allow_adjacent_grades ?? need.allow_adjacent_grades,
    must_skills: f.must_skills ?? need.must_skills,
    nice_skills: f.nice_skills ?? need.nice_skills,
    work_formats: f.work_formats ?? need.work_formats,
    city: f.city ?? need.city ?? "",
    fsp_only: f.fsp_only ?? false,
    max_expectation: f.max_expectation ?? null,
  };
}

interface RefinePanelProps {
  need: HiringNeed;
  pending: boolean;
  onApply: (f: ShortlistFilters) => void;
}

/** Edits the shortlist conditions on top of the need; the API keeps pinned and dismissed marks. */
export function RefinePanel({ need, pending, onApply }: RefinePanelProps) {
  const [d, setD] = useState<Draft>(() => draftOf(need));
  const [open, setOpen] = useState(false);
  const set = (patch: Partial<Draft>) => setD((x) => ({ ...x, ...patch }));
  return (
    <Card padding="none" className="h-fit lg:sticky lg:top-20">
      <button type="button" aria-expanded={open} onClick={() => setOpen((v) => !v)} className="flex w-full items-center justify-between gap-2 px-5 py-4 text-left lg:pointer-events-none lg:cursor-default">
        <span className="flex items-center gap-2 text-base font-semibold tracking-[-0.01em] text-ink">
          <SlidersHorizontal className="size-4 text-primary" aria-hidden />
          Уточнить подборку
        </span>
        <span className="text-sm text-primary lg:hidden">{open ? "Свернуть" : "Открыть"}</span>
      </button>
      <form
        noValidate
        className={cn("space-y-4 border-t border-line px-5 py-4 lg:block", !open && "hidden")}
        onSubmit={(e) => {
          e.preventDefault();
          onApply({
            grades: d.grades,
            allow_adjacent_grades: d.allow_adjacent_grades,
            must_skills: d.must_skills,
            nice_skills: d.nice_skills,
            work_formats: d.work_formats,
            city: d.city.trim() || null,
            fsp_only: d.fsp_only,
            max_expectation: d.max_expectation,
          });
        }}
      >
        <p className="text-xs text-ink-subtle">Закреплённые кандидаты останутся, новые будут отмечены. Сама потребность не меняется.</p>
        <Field label="Грейды">
          <ChipSelect<Grade> ariaLabel="Грейды" options={GRADES.map((g) => ({ value: g, label: GRADE_LABEL[g] }))} value={d.grades} onChange={(grades) => set({ grades })} />
        </Field>
        <Checkbox checked={d.allow_adjacent_grades} onChange={(e) => set({ allow_adjacent_grades: e.target.checked })} label="И соседние грейды" />
        <Field label="Обязательные навыки">
          <SkillInput value={d.must_skills} onChange={(must_skills) => set({ must_skills })} tone="primary" placeholder="Go, Kafka…" />
        </Field>
        <Field label="Желательные навыки">
          <SkillInput value={d.nice_skills} onChange={(nice_skills) => set({ nice_skills })} placeholder="Kubernetes…" />
        </Field>
        <Field label="Формат">
          <ChipSelect<WorkFormat> ariaLabel="Формат" options={WORK_FORMATS.map((w) => ({ value: w, label: WORK_FORMAT_LABEL[w] }))} value={d.work_formats} onChange={(work_formats) => set({ work_formats })} />
        </Field>
        <Field label="Город" optional>
          <Input value={d.city} onChange={(e) => set({ city: e.target.value })} placeholder="Любой" />
        </Field>
        <Field label="Ожидания не выше" optional>
          <MoneyInput value={d.max_expectation} onChange={(max_expectation) => set({ max_expectation })} placeholder={need.salary.max.toLocaleString("ru-RU")} />
        </Field>
        <Switch checked={d.fsp_only} onChange={(fsp_only) => set({ fsp_only })} label="Только с достижениями ФСП" />
        <div className="flex flex-col gap-2 pt-2">
          <Button type="submit" loading={pending}>
            Применить
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              setD(draftOf({ ...need, filters: {} }));
              onApply({});
            }}
            disabled={pending}
          >
            Вернуть условия потребности
          </Button>
        </div>
      </form>
    </Card>
  );
}
