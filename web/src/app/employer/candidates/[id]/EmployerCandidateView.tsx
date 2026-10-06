"use client";

import { Eye, Lock, Mail, MapPin, Phone, Send, ShieldCheck, Star, Trophy, UserRound } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import type { CandidateCard } from "@/api/types";
import { MinorBadge, candidateTitle, strengthPoints } from "@/components/domain/CandidateCard";
import { CategoryBadge } from "@/components/domain/Category";
import { ExplanationPanel, ScoreRing } from "@/components/domain/Explanation";
import { FspEmpty } from "@/components/domain/Fsp";
import { SkillLegend, SkillList } from "@/components/domain/Skills";
import { InvitationStatusBadge } from "@/components/domain/StatusBadges";
import { ProctoringBadge } from "@/features/employer/components/ProctoringBadge";
import { Badge, Button, Card, CardHeader, ErrorAlert, PageHeader, PageSkeleton, ProgressBar } from "@/components/ui";
import { useCandidateContacts, useEmployerCandidate, useNeed } from "@/features/employer/hooks";
import { InviteDialog } from "@/features/employer/components/InviteDialog";
import { formatDate, formatRelative, formatRub, pluralize } from "@/lib/format";
import { INDUSTRY_LABEL, SEARCH_STATUS_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";

function ContactsCard({ c, onInvite }: { c: CandidateCard; onInvite: () => void }) {
  const [reveal, setReveal] = useState(false);
  const contacts = useCandidateContacts(c.id, reveal);
  const status = c.relation.invitation_status ?? null;
  if (!c.relation.contacts_available) {
    return (
      <Card className="border-dashed">
        <div className="flex items-start gap-3">
          <Lock className="mt-0.5 size-5 shrink-0 text-ink-subtle" aria-hidden />
          <div>
            <p className="font-medium text-ink">Контакты скрыты</p>
            <p className="mt-1 text-sm text-ink-muted">Откроются, когда кандидат примет ваше приглашение или сам откликнется на вашу вакансию.</p>
          </div>
        </div>
        {status === "sent" || status === "viewed" ? (
          <div className="mt-4 flex items-center justify-between gap-2 rounded-control bg-canvas p-3 text-sm">
            <span className="text-ink-muted">Ваше приглашение</span>
            <InvitationStatusBadge status={status} />
          </div>
        ) : (
          <Button className="mt-4 w-full" icon={<Send className="size-4" aria-hidden />} onClick={onInvite}>
            Пригласить
          </Button>
        )}
      </Card>
    );
  }
  return (
    <Card className="border-success-200">
      <CardHeader title="Контакты" description={c.relation.applied ? "Кандидат откликнулся на вашу вакансию" : "Кандидат принял приглашение"} action={<ShieldCheck className="size-5 text-success" aria-hidden />} />
      {!reveal ? (
        <Button variant="secondary" className="w-full" icon={<Eye className="size-4" aria-hidden />} onClick={() => setReveal(true)}>
          Показать контакты
        </Button>
      ) : contacts.isPending ? (
        <PageSkeleton cards={1} />
      ) : contacts.isError ? (
        <ErrorAlert error={contacts.error} />
      ) : (
        <ul className="space-y-2.5 text-sm">
          {contacts.data.full_name && (
            <li className="flex items-center gap-2 font-medium text-ink">
              <UserRound className="size-4 text-ink-subtle" aria-hidden />
              {contacts.data.full_name}
            </li>
          )}
          {contacts.data.email && (
            <li className="flex items-center gap-2">
              <Mail className="size-4 text-ink-subtle" aria-hidden />
              <a href={`mailto:${contacts.data.email}`} className="link-inline">
                {contacts.data.email}
              </a>
            </li>
          )}
          {contacts.data.phone && (
            <li className="flex items-center gap-2">
              <Phone className="size-4 text-ink-subtle" aria-hidden />
              <a href={`tel:${contacts.data.phone.replace(/[^\d+]/g, "")}`} className="text-ink underline decoration-line-strong underline-offset-[3px] hover:decoration-ink">
                {contacts.data.phone}
              </a>
            </li>
          )}
          {contacts.data.telegram && (
            <li className="flex items-center gap-2">
              <Send className="size-4 text-ink-subtle" aria-hidden />
              <span className="text-ink">{contacts.data.telegram}</span>
            </li>
          )}
        </ul>
      )}
      <p className="mt-4 flex items-start gap-2 text-xs text-ink-subtle">
        <Lock className="mt-0.5 size-3.5 shrink-0" aria-hidden />
        Каждый просмотр контактов записывается в журнал доступа: кто, когда и на каком основании.
      </p>
    </Card>
  );
}

export function EmployerCandidateView({ id, needId }: { id: string; needId: string | null }) {
  const q = useEmployerCandidate(id, needId);
  const need = useNeed(needId ?? "", Boolean(needId));
  const [invite, setInvite] = useState(false);
  const back = needId ? { href: `/employer/needs/${needId}`, label: need.data?.title ?? "К подборке" } : { href: "/employer/candidates", label: "Поиск по базе" };

  if (q.isPending) return <PageSkeleton cards={3} />;
  if (q.isError) {
    return (
      <div>
        <PageHeader back={back} title="Профиль недоступен" />
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      </div>
    );
  }
  const c = q.data.candidate;
  const breakdown = q.data.breakdown ?? null;
  const status = c.relation.invitation_status ?? null;
  const active = status === "sent" || status === "viewed";
  const strength = strengthPoints(c.test_strength);

  return (
    <div>
      <PageHeader
        back={back}
        eyebrow={
          <span className="flex flex-wrap items-center gap-2">
            {c.category && <CategoryBadge category={c.category} long />}
            {c.minor && <MinorBadge />}
            {c.search_status === "active" && <Badge tone="success">{SEARCH_STATUS_LABEL.active}</Badge>}
            {status && <InvitationStatusBadge status={status} />}
            <ProctoringBadge badge={c.proctoring} />
          </span>
        }
        title={candidateTitle(c)}
        description={`Анонимный профиль · активность ${formatRelative(c.last_activity_at)}`}
        actions={
          <Button icon={<Send className="size-4" aria-hidden />} onClick={() => setInvite(true)} disabled={active || c.relation.contacts_available}>
            {active ? "Приглашение отправлено" : c.relation.contacts_available ? "Контакты открыты" : "Пригласить"}
          </Button>
        }
      />

      <div className="grid gap-6 lg:grid-cols-[1fr_20rem]">
        <div className="min-w-0 space-y-6">
          {breakdown && (
            <Card className="border-primary-200">
              <div className="flex items-start gap-4">
                <ScoreRing score={breakdown.total} size={64} />
                <div className="min-w-0 flex-1">
                  <h2 className="text-base font-semibold text-ink">Почему в подборке</h2>
                  <p className="mt-0.5 text-sm text-ink-muted">{need.data ? `Под потребность «${need.data.title}»` : "Под вашу потребность"}</p>
                  <ExplanationPanel breakdown={breakdown} defaultOpen className="mt-4" />
                </div>
              </div>
            </Card>
          )}

          <Card>
            <CardHeader title="Результат теста" description="Категория и сила результата — только по адаптивному тесту, не по резюме" />
            <div className="grid gap-6 sm:grid-cols-2">
              <div>
                <p className="text-xs text-ink-subtle">Сила результата в категории</p>
                <p className="tabular font-mono text-2xl font-semibold text-ink">{strength !== null ? `${strength}/100` : "—"}</p>
                {strength !== null && <ProgressBar value={strength} className="mt-2" label="Сила результата" />}
                {c.category_assigned_at && <p className="mt-1.5 text-xs text-ink-subtle">Категория подтверждена {formatDate(c.category_assigned_at)}</p>}
              </div>
              <div>
                <p className="text-xs text-ink-subtle">Регулярные задания</p>
                <p className="tabular font-mono text-2xl font-semibold text-ink">{c.tasks_done}</p>
                {typeof c.task_rating === "number" && (
                  <p className="mt-1 inline-flex items-center gap-1 text-sm text-ink-muted">
                    <Star className="size-4 fill-primary text-primary" aria-hidden />
                    средняя оценка {c.task_rating.toFixed(1)} из 5
                  </p>
                )}
              </div>
            </div>
          </Card>

          <Card>
            <CardHeader title="Навыки" />
            <SkillList skills={c.skills} highlight={need.data ? [...need.data.must_skills, ...need.data.nice_skills] : undefined} />
            <div className="mt-3">
              <SkillLegend />
            </div>
            {c.soft_skills.length > 0 && <p className="mt-4 text-sm text-ink-muted">Софт-скиллы: {c.soft_skills.join(", ")}</p>}
          </Card>

          <Card>
            <CardHeader title="Достижения ФСП" />
            {c.fsp && c.fsp.achievements.length > 0 ? (
              <ul className="space-y-2.5">
                {c.fsp.achievements.map((a, i) => (
                  <li key={`${a.date}-${i}`} className="flex gap-3 text-sm">
                    <Trophy className="mt-0.5 size-4 shrink-0 text-primary" aria-hidden />
                    <span className="min-w-0">
                      <span className="text-ink">{a.text}</span>
                      <span className="block text-xs text-ink-subtle">
                        {formatDate(a.date)}
                        {a.protocol_url && (
                          <>
                            {" · "}
                            <a href={a.protocol_url} target="_blank" rel="noreferrer" className="link-inline">
                              протокол
                            </a>
                          </>
                        )}
                      </span>
                    </span>
                  </li>
                ))}
              </ul>
            ) : (
              <FspEmpty />
            )}
          </Card>

          <Card>
            <CardHeader title="Опыт" description={c.experience_years !== null && c.experience_years !== undefined ? `Всего ${pluralize(Math.round(c.experience_years), ["год", "года", "лет"])}` : undefined} />
            {c.about && <p className="mb-4 text-sm leading-relaxed text-ink">{c.about}</p>}
            {c.experience.length === 0 ? (
              <p className="text-sm text-ink-subtle">Кандидат не указал места работы.</p>
            ) : (
              <ol className="space-y-4">
                {c.experience.map((e, i) => (
                  <li key={i} className="border-l-2 border-primary-200 pl-4">
                    <p className="font-medium text-ink">{e.position}</p>
                    <p className="text-sm text-ink-muted">
                      {e.company ?? "Компания скрыта кандидатом"} · {e.start} — {e.end ?? "сейчас"}
                    </p>
                    {e.description && <p className="mt-1 text-sm text-ink-muted">{e.description}</p>}
                  </li>
                ))}
              </ol>
            )}
          </Card>
        </div>

        <div className="space-y-6">
          <ContactsCard c={c} onInvite={() => setInvite(true)} />
          <Card>
            <CardHeader title="Ожидания" />
            <dl className="space-y-3 text-sm">
              <div>
                <dt className="text-ink-subtle">Зарплата</dt>
                <dd className="tabular font-mono font-semibold text-ink">{c.salary_expectation_min ? `от ${formatRub(c.salary_expectation_min)}` : "Не указана"}</dd>
              </div>
              <div>
                <dt className="text-ink-subtle">Формат</dt>
                <dd className="text-ink">{c.work_formats.map((f) => WORK_FORMAT_LABEL[f]).join(", ") || "—"}</dd>
              </div>
              <div>
                <dt className="text-ink-subtle">Город</dt>
                <dd className="flex items-center gap-1.5 text-ink">
                  <MapPin className="size-3.5 text-ink-subtle" aria-hidden />
                  {c.city ?? "Не указан"}
                  {c.relocation ? ", готов к переезду" : ""}
                </dd>
              </div>
              {c.roles.length > 0 && (
                <div>
                  <dt className="text-ink-subtle">Роли</dt>
                  <dd className="text-ink">{c.roles.join(", ")}</dd>
                </div>
              )}
              {c.industries.length > 0 && (
                <div>
                  <dt className="text-ink-subtle">Отрасли</dt>
                  <dd className="text-ink">{c.industries.map((i) => INDUSTRY_LABEL[i]).join(", ")}</dd>
                </div>
              )}
            </dl>
          </Card>
          {needId && need.data && (
            <Link href={`/employer/needs/${needId}`} className="tap block text-sm font-medium text-primary hover:underline">
              ← Вернуться к подборке «{need.data.title}»
            </Link>
          )}
        </div>
      </div>

      <InviteDialog open={invite} onClose={() => setInvite(false)} target={{ id: c.id, code: c.code, display_name: c.display_name, category: c.category, minor: c.minor }} need={need.data ?? null} />
    </div>
  );
}
