"use client";

import { Building2, FileText, MailWarning, MessageSquareWarning, PauseCircle } from "lucide-react";
import { useState, type ReactNode } from "react";
import { isApiError } from "@/api/client";
import type { TrustAction, TrustItem, TrustItemKind, TrustView } from "@/api/admin";
import { SalaryRange } from "@/components/domain/Category";
import { Alert, Badge, Button, Card, Dialog, EmptyState, ErrorAlert, Field, Input, PageHeader, SkeletonCard, Tabs, Textarea, ToggleChip, tabPanelProps, useToast } from "@/components/ui";
import { useOverview, useTrustAction, useTrustQueue } from "@/features/admin/hooks";
import { ITEM_KIND_LABEL, TRUST_ACTION_LABEL, trustActionEffect } from "@/features/admin/labels";
import { formatDate, formatDateTime } from "@/lib/format";
import { TRUST_LEVEL_LABEL } from "@/lib/labels";

const KINDS: TrustItemKind[] = ["complaint", "vacancy", "invitation", "company"];
const ICON: Record<TrustItemKind, ReactNode> = {
  company: <Building2 className="size-4" aria-hidden />,
  vacancy: <FileText className="size-4" aria-hidden />,
  invitation: <MailWarning className="size-4" aria-hidden />,
  complaint: <MessageSquareWarning className="size-4" aria-hidden />,
};
const VACANCY_STATUS: Record<string, string> = { draft: "черновик", published: "опубликована", closed: "закрыта" };
const TARGET_ACCUSATIVE: Record<TrustItemKind, string> = { company: "на компанию", vacancy: "на вакансию", invitation: "на приглашение", complaint: "на жалобу" };

interface Pending {
  item: TrustItem;
  action: TrustAction;
}

function ActionDialog({ pending, onClose }: { pending: Pending | null; onClose: () => void }) {
  const toast = useToast();
  const act = useTrustAction();
  const [comment, setComment] = useState("");
  const [days, setDays] = useState("14");
  const [touched, setTouched] = useState(false);
  const close = () => {
    setComment("");
    setDays("14");
    setTouched(false);
    act.reset();
    onClose();
  };
  if (!pending) return <Dialog open={false} onClose={close} title="">{null}</Dialog>;
  const { item, action } = pending;
  const freeze = action === "freeze";
  const n = Number(days);
  const daysError = freeze && touched && (!Number.isInteger(n) || n < 1 || n > 90) ? "От 1 до 90 дней" : undefined;
  const commentError = touched && !comment.trim() ? "Комментарий обязателен: его увидит компания" : undefined;
  const submit = () => {
    setTouched(true);
    if (!comment.trim() || (freeze && (!Number.isInteger(n) || n < 1 || n > 90))) return;
    act.mutate(
      { kind: item.kind, id: item.id, action, comment: comment.trim(), days: freeze ? n : null },
      {
        onSuccess: (r) => {
          toast.success("Готово", r.complaints_resolved ? `Закрыто жалоб: ${r.complaints_resolved}` : undefined);
          close();
        },
      },
    );
  };
  return (
    <Dialog
      open
      onClose={close}
      title={TRUST_ACTION_LABEL[action]}
      description={`${ITEM_KIND_LABEL[item.kind]}: ${item.title}${item.company && item.kind !== "company" ? ` · ${item.company.name}` : ""}`}
      footer={
        <>
          <Button variant="secondary" onClick={close}>
            Отмена
          </Button>
          <Button variant={action === "dismiss" || action === "restore" ? "primary" : "danger"} loading={act.isPending} onClick={submit}>
            {TRUST_ACTION_LABEL[action]}
          </Button>
        </>
      }
    >
      <p className="text-sm text-ink-muted">{trustActionEffect(action, item.kind)}</p>
      {freeze && (
        <Field label="Срок, дней" hint="Квота компании будет 0 до конца срока" error={daysError} required className="mt-4">
          <Input type="number" inputMode="numeric" min={1} max={90} value={days} onChange={(e) => setDays(e.target.value)} className="max-w-32" />
        </Field>
      )}
      <Field label="Комментарий" hint="Компания увидит его на странице «Доверие и квота». Автора жалобы не называйте." error={commentError} required className="mt-4">
        <Textarea value={comment} maxLength={2000} rows={4} onChange={(e) => setComment(e.target.value)} />
      </Field>
      {act.isError && (
        <Alert tone="danger" title={isApiError(act.error) ? act.error.title : "Не удалось выполнить действие"} className="mt-4">
          {isApiError(act.error) ? act.error.detail : null}
        </Alert>
      )}
    </Dialog>
  );
}

function Item({ item, onAct }: { item: TrustItem; onAct: (a: TrustAction) => void }) {
  const c = item.company;
  return (
    <Card>
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <Badge tone={item.kind === "complaint" ? "warning" : "neutral"} icon={ICON[item.kind]}>
              {ITEM_KIND_LABEL[item.kind]}
            </Badge>
            {item.vacancy_status && <Badge tone="muted">{VACANCY_STATUS[item.vacancy_status] ?? item.vacancy_status}</Badge>}
            {item.open_complaints > 0 && <Badge tone="warning">жалоб: {item.open_complaints}</Badge>}
            <span className="text-xs text-ink-subtle">{item.until ? `с ${formatDate(item.at)} до ${formatDate(item.until)}` : formatDateTime(item.at)}</span>
          </div>
          <h2 className="mt-2 text-base font-semibold text-ink">{item.title}</h2>
          {c && (
            <p className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-ink-muted">
              {[item.kind !== "company" ? c.name : null, c.level ? TRUST_LEVEL_LABEL[c.level] : null, typeof c.quota === "number" ? `квота ${c.quota} в неделю` : null]
                .filter(Boolean)
                .join(" · ")}
              {c.frozen_until && (
                <span className="inline-flex items-center gap-1 text-accent-strong">
                  <PauseCircle className="size-3.5" aria-hidden />
                  приглашения на паузе до {formatDate(c.frozen_until)}
                </span>
              )}
            </p>
          )}
        </div>
        {item.salary && <SalaryRange salary={item.salary} size="sm" label="Вилка" />}
      </div>
      {item.complaint ? (
        <div className="mt-3 rounded-control bg-canvas p-3 text-sm">
          <p className="font-medium text-ink">
            {item.complaint.reason_ru} · жалоба {TARGET_ACCUSATIVE[item.complaint.target_kind]}
          </p>
          {item.complaint.comment ? <p className="mt-1 whitespace-pre-line text-ink-muted">«{item.complaint.comment}»</p> : <p className="mt-1 text-ink-subtle">Без комментария</p>}
        </div>
      ) : (
        item.reasons.length > 0 && (
          <ul className="mt-3 list-disc space-y-1 pl-5 text-sm text-ink">
            {item.reasons.map((r) => (
              <li key={r}>{r}</li>
            ))}
          </ul>
        )
      )}
      <div className="mt-4 flex flex-wrap gap-2">
        {item.actions.map((a) => (
          <Button key={a} size="sm" variant={a === "dismiss" || a === "restore" ? "secondary" : "ghost"} className={a === "unpublish" || a === "freeze" ? "text-accent-strong hover:bg-accent-50 hover:text-accent-strong" : undefined} onClick={() => onAct(a)}>
            {TRUST_ACTION_LABEL[a]}
          </Button>
        ))}
      </div>
    </Card>
  );
}

function Queue({ view, kind }: { view: TrustView; kind: TrustItemKind | null }) {
  const q = useTrustQueue(view, kind);
  const [pending, setPending] = useState<Pending | null>(null);
  if (q.isPending) return <SkeletonCard lines={4} />;
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  const items = q.data.pages.flatMap((p) => p.items);
  if (!items.length)
    return (
      <EmptyState
        title={view === "open" ? "Всё разобрано" : "Действующих мер нет"}
        description={view === "open" ? "Новые жалобы и отметки антифейка появятся здесь, старые — сверху." : "Снятые вакансии и компании на паузе появятся здесь — их можно вернуть."}
      />
    );
  return (
    <div className="space-y-4">
      {items.map((item) => (
        <Item key={`${item.kind}-${item.id}`} item={item} onAct={(action) => setPending({ item, action })} />
      ))}
      {q.hasNextPage && (
        <div className="text-center">
          <Button variant="secondary" loading={q.isFetchingNextPage} onClick={() => void q.fetchNextPage()}>
            Показать ещё
          </Button>
        </div>
      )}
      <ActionDialog pending={pending} onClose={() => setPending(null)} />
    </div>
  );
}

export default function AntifakePage() {
  const [view, setView] = useState<TrustView>("open");
  const [kind, setKind] = useState<TrustItemKind | null>(null);
  const a = useOverview().data?.antifake;
  return (
    <div>
      <PageHeader
        title="Антифейк и жалобы"
        description="Отметки антифейка — вилка выше рынка, слишком широкая вилка, копии вакансий — и жалобы кандидатов в одной очереди, сначала самые старые. Каждое действие — с комментарием: компания увидит его на странице «Доверие и квота»."
      />
      <Tabs
        idPrefix="antifake"
        ariaLabel="Очередь антифейка"
        value={view}
        onChange={setView}
        items={[
          { value: "open", label: "Ждут решения", count: a ? a.companies + a.vacancies + a.invitations + a.complaints : undefined },
          { value: "sanctions", label: "Действующие меры", count: a ? a.unpublished_vacancies + a.frozen_companies : undefined },
        ]}
        className="mb-4"
      />
      <div {...tabPanelProps("antifake", view)} className="focus-visible:outline-none">
        <div className="mb-5 flex flex-wrap gap-2" role="group" aria-label="Что показывать">
          <ToggleChip selected={kind === null} onToggle={() => setKind(null)}>
            Всё
          </ToggleChip>
          {KINDS.filter((k) => view === "open" || k === "vacancy" || k === "company").map((k) => (
            <ToggleChip key={k} selected={kind === k} onToggle={() => setKind(kind === k ? null : k)} icon={ICON[k]}>
              {k === "complaint" ? "Жалобы" : k === "vacancy" ? "Вакансии" : k === "invitation" ? "Приглашения" : "Компании"}
            </ToggleChip>
          ))}
        </div>
        <Queue view={view} kind={kind} />
      </div>
    </div>
  );
}
