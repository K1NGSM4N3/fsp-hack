"use client";

import { Building2, Check, CheckCircle2, Globe, Lock, MapPin, MessageSquare, Phone, X } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { DeclineReason, ReceivedInvitation } from "@/api/types";
import { SalaryRange } from "@/components/domain/Category";
import { DomainBadge } from "@/components/domain/DomainBadge";
import { HiringOutcomes } from "@/components/domain/HiringOutcomes";
import { ReportButton } from "@/components/domain/ReportButton";
import { InvitationStatusBadge } from "@/components/domain/StatusBadges";
import { Alert, Button, Card, CardHeader, ChipSelect, Dialog, ErrorAlert, Field, PageHeader, PageSkeleton, Textarea, useToast } from "@/components/ui";
import { useAcceptInvitation, useDeclineInvitation, useReceivedInvitation } from "@/features/candidate/hooks";
import { formatDate, formatDateTime, formatRelative } from "@/lib/format";
import { DECLINE_REASON_LABEL, DECLINE_REASONS, WORK_FORMAT_LABEL } from "@/lib/labels";

function Timeline({ inv }: { inv: ReceivedInvitation }) {
  const rows = [
    { label: "Получено", at: inv.created_at },
    { label: "Вы открыли", at: inv.viewed_at },
    { label: inv.status === "accepted" ? "Вы приняли" : inv.status === "declined" ? "Вы отклонили" : "Ответ", at: inv.responded_at },
  ];
  return (
    <ol className="space-y-3 text-sm">
      {rows.map((r) => (
        <li key={r.label} className="flex items-center justify-between gap-3">
          <span className={r.at ? "text-ink" : "text-ink-subtle"}>{r.label}</span>
          <span className="tabular text-ink-subtle">{r.at ? formatDateTime(r.at) : "—"}</span>
        </li>
      ))}
      {inv.outcomes.length > 0 && (
        <li className="border-t border-line pt-3">
          <HiringOutcomes outcomes={inv.outcomes} />
        </li>
      )}
      {(inv.status === "sent" || inv.status === "viewed") && (
        <li className="flex items-center justify-between gap-3 border-t border-line pt-3">
          <span className="text-ink">Действует до</span>
          <span className="text-ink-muted">
            {formatDate(inv.expires_at)} ({formatRelative(inv.expires_at)})
          </span>
        </li>
      )}
    </ol>
  );
}

function DeclineDialog({ inv, open, onClose }: { inv: ReceivedInvitation; open: boolean; onClose: () => void }) {
  const toast = useToast();
  const decline = useDeclineInvitation();
  const [reason, setReason] = useState<DeclineReason[]>([]);
  const [comment, setComment] = useState("");
  const picked = reason[0];
  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Отклонить приглашение"
      description="Причина анонимно попадёт в статистику компании — так приглашения станут точнее."
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            Отмена
          </Button>
          <Button
            variant="danger"
            disabled={!picked}
            loading={decline.isPending}
            onClick={() =>
              picked &&
              decline.mutate(
                { id: inv.id, body: { reason: picked, comment: comment.trim() || null } },
                {
                  onSuccess: () => {
                    onClose();
                    toast.success("Приглашение отклонено", picked === "not_looking" ? "Статус поиска изменён на «Не ищу работу»" : undefined);
                  },
                  onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось отклонить"),
                },
              )
            }
          >
            Отклонить
          </Button>
        </>
      }
    >
      <div className="space-y-5">
        <Field label="Главная причина" required>
          <ChipSelect<DeclineReason> single ariaLabel="Причина отказа" options={DECLINE_REASONS.map((r) => ({ value: r, label: DECLINE_REASON_LABEL[r] }))} value={reason} onChange={setReason} />
        </Field>
        {picked === "not_looking" && <Alert tone="info">Мы переключим статус поиска на «Не ищу работу» — новые приглашения приходить не будут.</Alert>}
        <Field label="Комментарий" optional hint="Например: «рассматриваю от 320 000 ₽» или «интересен только удалённый формат»">
          <Textarea rows={3} value={comment} onChange={(e) => setComment(e.target.value)} maxLength={500} />
        </Field>
      </div>
    </Dialog>
  );
}

function AcceptDialog({ inv, open, onClose }: { inv: ReceivedInvitation; open: boolean; onClose: () => void }) {
  const toast = useToast();
  const accept = useAcceptInvitation();
  return (
    <Dialog
      open={open}
      onClose={onClose}
      size="sm"
      title="Принять приглашение?"
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            Отмена
          </Button>
          <Button
            loading={accept.isPending}
            icon={<Check className="size-4" aria-hidden />}
            onClick={() =>
              accept.mutate(inv.id, {
                onSuccess: () => {
                  onClose();
                  toast.success("Приглашение принято", `${inv.company_name} получила ваши контакты`);
                },
                onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось принять", isApiError(e) ? (e.detail ?? undefined) : undefined),
              })
            }
          >
            Принять
          </Button>
        </>
      }
    >
      <div className="space-y-3 text-sm text-ink-muted">
        <p>
          Компания <span className="font-medium text-ink">{inv.company_name}</span> увидит ваше имя и контакты из профиля: email, телефон и Telegram.
        </p>
        <p className="flex items-start gap-2">
          <Lock className="mt-0.5 size-4 shrink-0 text-ink-subtle" aria-hidden />
          Каждый просмотр контактов записывается в журнал доступа.
        </p>
        <p>Если у компании подключена своя система подбора (ATS), эти же данные попадут туда — это тоже отразится в журнале.</p>
      </div>
    </Dialog>
  );
}

export function InvitationDetail({ id }: { id: string }) {
  const q = useReceivedInvitation(id);
  const [dialog, setDialog] = useState<"accept" | "decline" | null>(null);

  if (q.isPending) return <PageSkeleton cards={2} />;
  if (q.isError) {
    return (
      <div>
        <PageHeader back={{ href: "/candidate/invitations", label: "Приглашения" }} title="Приглашение недоступно" />
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      </div>
    );
  }
  const inv = q.data;
  const active = inv.status === "sent" || inv.status === "viewed";

  return (
    <div>
      <PageHeader
        back={{ href: "/candidate/invitations", label: "Приглашения" }}
        eyebrow={<InvitationStatusBadge status={inv.status} audience="candidate" />}
        title={inv.title}
        description={
          <span className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
            <span className="inline-flex items-center gap-1.5">
              <Building2 className="size-4" aria-hidden />
              {inv.company_name}
              {inv.company_industry ? ` · ${inv.company_industry}` : ""}
            </span>
            <DomainBadge domain={inv.company_verified_domain} />
          </span>
        }
      />

      <div className="grid gap-6 lg:grid-cols-[1fr_20rem]">
        <div className="min-w-0 space-y-6">
          <Card padding="lg">
            <p className="text-sm text-ink-subtle">Зарплата</p>
            <SalaryRange salary={inv.salary} size="xl" className="mt-1 block" label="Зарплата" />
            <dl className="mt-6 grid gap-4 text-sm sm:grid-cols-2">
              <div className="flex items-start gap-2.5">
                <MapPin className="mt-0.5 size-4 shrink-0 text-ink-subtle" aria-hidden />
                <div>
                  <dt className="text-ink-subtle">Формат</dt>
                  <dd className="text-ink">
                    {inv.work_format ? WORK_FORMAT_LABEL[inv.work_format] : "Обсуждается"}
                    {inv.company_city && inv.work_format !== "remote" ? `, ${inv.company_city}` : ""}
                  </dd>
                </div>
              </div>
              <div className="flex items-start gap-2.5">
                <Phone className="mt-0.5 size-4 shrink-0 text-ink-subtle" aria-hidden />
                <div>
                  <dt className="text-ink-subtle">Как связаться с компанией</dt>
                  <dd className="font-medium break-words text-ink">{inv.contact}</dd>
                </div>
              </div>
            </dl>

            {active && (
              <div className="mt-8 flex flex-col gap-3 border-t border-line pt-6 sm:flex-row">
                <Button size="lg" icon={<Check className="size-5" aria-hidden />} onClick={() => setDialog("accept")} className="sm:flex-1">
                  Принять
                </Button>
                <Button size="lg" variant="secondary" icon={<X className="size-5" aria-hidden />} onClick={() => setDialog("decline")} className="sm:flex-1">
                  Отклонить
                </Button>
              </div>
            )}
            {inv.status === "accepted" && (
              <Alert tone="success" title="Вы приняли приглашение" className="mt-6" icon={<CheckCircle2 className="size-5 text-success" aria-hidden />}>
                Компания получила ваши контакты и свяжется с вами. Можно написать первыми: {inv.contact}.
              </Alert>
            )}
            {inv.status === "declined" && (
              <Alert tone="info" title="Вы отклонили приглашение" className="mt-6">
                {inv.decline_reason ? `Причина: ${DECLINE_REASON_LABEL[inv.decline_reason]}.` : null} Компания видит только статус и причину, без ваших контактов.
              </Alert>
            )}
            {inv.status === "expired" && <Alert title="Срок приглашения истёк" className="mt-6">Приглашения действуют 14 дней.</Alert>}
            <div className="mt-6 flex justify-end">
              <ReportButton targetKind="invitation" targetId={inv.id} />
            </div>
            {inv.status === "withdrawn" && <Alert title="Компания отозвала приглашение" className="mt-6" />}
          </Card>

          <Card>
            <CardHeader title="Сообщение компании" action={<MessageSquare className="size-5 text-ink-subtle" aria-hidden />} />
            <p className="text-sm leading-relaxed whitespace-pre-line text-ink sm:text-base">{inv.message}</p>
            {inv.vacancy_id && (
              <Link href={`/candidate/vacancies/${inv.vacancy_id}`} className="tap mt-4 inline-block text-sm font-medium text-primary hover:underline">
                Открыть связанную вакансию
              </Link>
            )}
          </Card>
        </div>

        <div className="space-y-6">
          <Card>
            <CardHeader title="О компании" />
            <p className="font-medium text-ink">{inv.company_name}</p>
            <DomainBadge domain={inv.company_verified_domain} className="mt-2" />
            {inv.company_description && <p className="mt-2 text-sm leading-relaxed text-ink-muted">{inv.company_description}</p>}
            <ul className="mt-3 space-y-1.5 text-sm text-ink-muted">
              {inv.company_city && (
                <li className="flex items-center gap-2">
                  <MapPin className="size-4 text-ink-subtle" aria-hidden />
                  {inv.company_city}
                </li>
              )}
              {inv.company_website && (
                <li className="flex items-center gap-2">
                  <Globe className="size-4 text-ink-subtle" aria-hidden />
                  <a href={inv.company_website} target="_blank" rel="noreferrer" className="text-primary hover:underline">
                    {inv.company_website.replace(/^https?:\/\//, "")}
                  </a>
                </li>
              )}
            </ul>
          </Card>
          <Card>
            <CardHeader title="Статус" />
            <Timeline inv={inv} />
          </Card>
        </div>
      </div>

      <AcceptDialog inv={inv} open={dialog === "accept"} onClose={() => setDialog(null)} />
      <DeclineDialog inv={inv} open={dialog === "decline"} onClose={() => setDialog(null)} />
    </div>
  );
}
