"use client";

import { ChevronRight, TriangleAlert } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import type { QueueFilter, QueueSession } from "@/api/admin";
import { CategoryBadge } from "@/components/domain/Category";
import { Badge, Button, Card, EmptyState, ErrorAlert, PageHeader, SkeletonCard, Tabs, tabPanelProps } from "@/components/ui";
import { useOverview, useProctoringQueue } from "@/features/admin/hooks";
import { DECISION_DONE, QUEUE_FILTER_LABEL, VIDEO_LABEL, lz } from "@/features/admin/labels";
import { formatDateTime, pluralize } from "@/lib/format";
import { ATTEMPT_KIND_LABEL } from "@/lib/labels";

const FILTERS: QueueFilter[] = ["needs_review", "remarks", "video"];
const EMPTY: Record<QueueFilter, string> = {
  needs_review: "Тестов, которые требуют подтверждения, нет. Они появятся, когда нетипичная картина ответов совпадёт с уходами со вкладки или вставкой текста.",
  remarks: "Тестов с уходами со вкладки, вставкой текста или выходом из полноэкранного режима нет.",
  video: "Тестов с видеозаписью пока нет.",
};

function remarksText(s: QueueSession): string {
  const parts = [
    s.remarks.left_test ? pluralize(s.remarks.left_test, ["уход", "ухода", "уходов"]) : null,
    s.remarks.pastes ? pluralize(s.remarks.pastes, ["вставка", "вставки", "вставок"]) : null,
    s.remarks.fullscreen_exits ? `${s.remarks.fullscreen_exits} из полн. экрана` : null,
  ].filter(Boolean);
  return parts.length ? parts.join(", ") : "—";
}

function Status({ s }: { s: QueueSession }) {
  if (s.decision) {
    const d = DECISION_DONE[s.decision];
    return <Badge tone={d.tone}>{d.text}</Badge>;
  }
  if (s.needs_review)
    return (
      <Badge tone="warning" icon={<TriangleAlert className="size-3" aria-hidden />}>
        Требует подтверждения
      </Badge>
    );
  return <Badge tone="muted">Ждёт просмотра</Badge>;
}

function Row({ s }: { s: QueueSession }) {
  const href = `/admin/proctoring/${s.session_id}`;
  const video = VIDEO_LABEL[s.video];
  return (
    <tr className="group border-t border-line align-middle hover:bg-canvas">
      <td className="py-3 pr-3 pl-5">
        <Link href={href} className="font-mono font-medium text-primary hover:underline">
          {s.candidate_code}
        </Link>
        <p className="mt-0.5 text-xs text-ink-subtle">{ATTEMPT_KIND_LABEL[s.attempt_kind]}</p>
      </td>
      <td className="px-3 py-3">
        <CategoryBadge category={s.category} size="sm" />
        <p className="mt-1 text-xs text-ink-subtle">{s.status === "passed" ? (s.result_in_force ? "сдан, грейд действует" : "сдан, грейд не действует") : "не сдан"}</p>
      </td>
      <td className="px-3 py-3 text-sm whitespace-nowrap text-ink-muted">{formatDateTime(s.finished_at)}</td>
      <td className={s.aberrant ? "tabular px-3 py-3 font-mono text-sm font-medium text-accent-strong" : "tabular px-3 py-3 font-mono text-sm text-ink-muted"}>{lz(s.person_fit_lz)}</td>
      <td className="px-3 py-3 text-sm text-ink">{remarksText(s)}</td>
      <td className="px-3 py-3">
        <Badge tone={video.tone}>{video.text}</Badge>
      </td>
      <td className="py-3 pr-5 pl-3">
        <div className="flex items-center justify-between gap-2">
          <Status s={s} />
          <ChevronRight className="size-4 text-ink-faint" aria-hidden />
        </div>
      </td>
    </tr>
  );
}

function MobileItem({ s }: { s: QueueSession }) {
  const video = VIDEO_LABEL[s.video];
  return (
    <li className="border-t border-line first:border-t-0">
      <Link href={`/admin/proctoring/${s.session_id}`} className="block px-4 py-4 hover:bg-canvas">
        <div className="flex items-center justify-between gap-3">
          <span className="font-mono font-medium text-primary">{s.candidate_code}</span>
          <Status s={s} />
        </div>
        <div className="mt-2 flex flex-wrap items-center gap-2">
          <CategoryBadge category={s.category} size="sm" />
          <Badge tone={video.tone}>{video.text}</Badge>
        </div>
        <dl className="mt-3 grid grid-cols-2 gap-x-3 gap-y-1 text-xs">
          <dt className="text-ink-subtle">Завершён</dt>
          <dd className="text-ink">{formatDateTime(s.finished_at)}</dd>
          <dt className="text-ink-subtle" title="Статистика person-fit l_z: ниже −2 — нетипичная картина ответов">Согласованность ответов</dt>
          <dd className={s.aberrant ? "font-mono font-medium text-accent-strong" : "font-mono text-ink"}>{lz(s.person_fit_lz)}</dd>
          <dt className="text-ink-subtle">Замечания</dt>
          <dd className="text-ink">{remarksText(s)}</dd>
        </dl>
      </Link>
    </li>
  );
}

function Queue({ filter }: { filter: QueueFilter }) {
  const q = useProctoringQueue(filter);
  if (q.isPending) return <SkeletonCard lines={5} />;
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  const items = q.data.pages.flatMap((p) => p.items);
  if (!items.length) return <EmptyState title="Очередь пуста" description={EMPTY[filter]} />;
  return (
    <Card padding="none">
      <div className="hidden md:block">
        <table className="w-full text-left">
          <caption className="sr-only">{QUEUE_FILTER_LABEL[filter]}, сначала самые старые</caption>
          <thead>
            <tr className="text-xs text-ink-subtle">
              <th scope="col" className="py-3 pr-3 pl-5 font-medium">
                Кандидат
              </th>
              <th scope="col" className="px-3 py-3 font-medium">
                Категория
              </th>
              <th scope="col" className="px-3 py-3 font-medium">
                Завершён
              </th>
              <th scope="col" className="px-3 py-3 font-medium" title="Статистика person-fit l_z; ниже −2 — нетипичная картина ответов">
                Согласованность
              </th>
              <th scope="col" className="px-3 py-3 font-medium">
                Замечания
              </th>
              <th scope="col" className="px-3 py-3 font-medium">
                Видео
              </th>
              <th scope="col" className="py-3 pr-5 pl-3 font-medium">
                Статус
              </th>
            </tr>
          </thead>
          <tbody>
            {items.map((s) => (
              <Row key={s.session_id} s={s} />
            ))}
          </tbody>
        </table>
      </div>
      <ul className="md:hidden">
        {items.map((s) => (
          <MobileItem key={s.session_id} s={s} />
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
  );
}

export default function ProctoringQueuePage() {
  const [filter, setFilter] = useState<QueueFilter>("needs_review");
  const counts = useOverview().data?.proctoring;
  return (
    <div>
      <PageHeader
        title="Прокторинг"
        description="Разбор тестов, где картина ответов и поведение во время теста расходятся. Сначала самые старые. Имени кандидата здесь нет: для решения хватает ответов, событий и записи."
      />
      <Tabs
        idPrefix="queue"
        ariaLabel="Очередь тестов"
        value={filter}
        onChange={setFilter}
        items={FILTERS.map((f) => ({
          value: f,
          label: QUEUE_FILTER_LABEL[f],
          count: f === "needs_review" ? counts?.needs_review : f === "remarks" ? counts?.remarks : counts?.with_video,
        }))}
        className="mb-5"
      />
      <div {...tabPanelProps("queue", filter)} className="focus-visible:outline-none">
        <Queue filter={filter} />
      </div>
    </div>
  );
}
