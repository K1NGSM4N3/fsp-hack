"use client";

import { useState } from "react";
import type { AuditEntry, AuditFilter, ModerationAction, ModerationTarget } from "@/api/admin";
import { Badge, Button, Card, EmptyState, ErrorAlert, Field, Input, PageHeader, Select, SkeletonCard } from "@/components/ui";
import { useAuditLog } from "@/features/admin/hooks";
import { ACTION_LABEL, ACTION_TONE, TARGET_LABEL } from "@/features/admin/labels";
import { formatDateTime } from "@/lib/format";

const ACTIONS = Object.keys(ACTION_LABEL) as ModerationAction[];
const TARGETS = Object.keys(TARGET_LABEL) as ModerationTarget[];
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const CODE = /^k-?[0-9a-z]{6}$/i;

function Target({ e }: { e: AuditEntry }) {
  const label = e.target_label ?? e.company_name ?? null;
  return (
    <div className="min-w-0">
      <p className="text-sm text-ink">
        <span className="text-ink-subtle">{TARGET_LABEL[e.target_kind]}</span>
        {e.candidate_code && <span className="ml-1.5 font-mono">{e.candidate_code}</span>}
        {label && <span className="ml-1.5">{label}</span>}
      </p>
      {e.company_name && e.target_kind !== "company" && <p className="text-xs text-ink-subtle">{e.company_name}</p>}
    </div>
  );
}

function Details({ e }: { e: AuditEntry }) {
  return (
    <div className="min-w-0 space-y-1">
      {e.comment && <p className="text-sm whitespace-pre-line text-ink">{e.comment}</p>}
      {e.details.map((d) => (
        <p key={d} className="text-xs text-ink-muted">
          {d}
        </p>
      ))}
      {!e.comment && !e.details.length && <span className="text-ink-faint">—</span>}
    </div>
  );
}

export default function AuditLogPage() {
  const [action, setAction] = useState<ModerationAction | "">("");
  const [target, setTarget] = useState<ModerationTarget | "">("");
  const [subject, setSubject] = useState("");
  const subjectValid = !subject.trim() || UUID.test(subject.trim()) || CODE.test(subject.trim());
  const filter: AuditFilter = {
    action: action || null,
    target_kind: target || null,
    subject: subjectValid && subject.trim() ? subject.trim() : null,
  };
  const q = useAuditLog(filter);
  const items = q.data?.pages.flatMap((p) => p.items) ?? [];
  return (
    <div>
      <PageHeader title="Журнал" description="Кто, когда и что сделал: решения по тестам, действия антифейка и каждый просмотр видеозаписи теста. Записи не удаляются и переживают аккаунты." />
      <Card className="mb-5">
        <div className="grid gap-4 sm:grid-cols-3">
          <Field label="Действие">
            <Select value={action} onChange={(e) => setAction(e.target.value as ModerationAction | "")}>
              <option value="">Все действия</option>
              {ACTIONS.map((a) => (
                <option key={a} value={a}>
                  {ACTION_LABEL[a]}
                </option>
              ))}
            </Select>
          </Field>
          <Field label="Объект">
            <Select value={target} onChange={(e) => setTarget(e.target.value as ModerationTarget | "")}>
              <option value="">Все объекты</option>
              {TARGETS.map((t) => (
                <option key={t} value={t}>
                  {TARGET_LABEL[t]}
                </option>
              ))}
            </Select>
          </Field>
          <Field label="Код кандидата или ID" hint="K-7Q3M2X — всё о кандидате; ID — теста, вакансии, компании или жалобы" error={subjectValid ? undefined : "Нужен код вида K-7Q3M2X или ID"} optional>
            <Input value={subject} onChange={(e) => setSubject(e.target.value)} placeholder="K-7Q3M2X" spellCheck={false} autoCapitalize="characters" />
          </Field>
        </div>
      </Card>
      {q.isPending ? (
        <SkeletonCard lines={6} />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : !items.length ? (
        <EmptyState title="Записей нет" description="Под эти фильтры ничего не нашлось." />
      ) : (
        <Card padding="none">
          <div className="hidden md:block">
            <table className="w-full text-left">
              <caption className="sr-only">Журнал модерации, новые сверху</caption>
              <thead>
                <tr className="text-xs text-ink-subtle">
                  <th scope="col" className="py-3 pr-3 pl-5 font-medium">
                    Когда
                  </th>
                  <th scope="col" className="px-3 py-3 font-medium">
                    Кто
                  </th>
                  <th scope="col" className="px-3 py-3 font-medium">
                    Действие
                  </th>
                  <th scope="col" className="px-3 py-3 font-medium">
                    Объект
                  </th>
                  <th scope="col" className="py-3 pr-5 pl-3 font-medium">
                    Комментарий
                  </th>
                </tr>
              </thead>
              <tbody>
                {items.map((e) => (
                  <tr key={e.id} className="border-t border-line align-top">
                    <td className="py-3 pr-3 pl-5 text-sm whitespace-nowrap text-ink-muted">{formatDateTime(e.at)}</td>
                    <td className="px-3 py-3 text-sm whitespace-nowrap text-ink">{e.moderator ?? "удалённый аккаунт"}</td>
                    <td className="px-3 py-3">
                      <Badge tone={ACTION_TONE[e.action]}>{ACTION_LABEL[e.action]}</Badge>
                    </td>
                    <td className="px-3 py-3">
                      <Target e={e} />
                    </td>
                    <td className="max-w-md py-3 pr-5 pl-3">
                      <Details e={e} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <ul className="md:hidden">
            {items.map((e) => (
              <li key={e.id} className="border-t border-line px-4 py-4 first:border-t-0">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <Badge tone={ACTION_TONE[e.action]}>{ACTION_LABEL[e.action]}</Badge>
                  <span className="text-xs text-ink-subtle">{formatDateTime(e.at)}</span>
                </div>
                <div className="mt-2">
                  <Target e={e} />
                </div>
                <div className="mt-2">
                  <Details e={e} />
                </div>
                <p className="mt-2 text-xs break-all text-ink-subtle">{e.moderator ?? "удалённый аккаунт"}</p>
              </li>
            ))}
          </ul>
          {q.hasNextPage && (
            <div className="border-t border-line p-4 text-center">
              <Button variant="secondary" loading={q.isFetchingNextPage} onClick={() => void q.fetchNextPage()}>
                Показать ещё
              </Button>
            </div>
          )}
        </Card>
      )}
    </div>
  );
}
