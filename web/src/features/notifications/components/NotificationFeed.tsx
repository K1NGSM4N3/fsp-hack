"use client";

import { BellOff, CheckCheck, Settings2 } from "lucide-react";
import { useState } from "react";
import { Button, ButtonLink, EmptyState, ErrorAlert, PageHeader, SkeletonCard, Tabs, tabPanelProps } from "@/components/ui";
import { useMarkRead, useNotificationFeed, useUnreadCount } from "../hooks";
import { NotificationRow } from "./NotificationRow";

type CabinetRole = "candidate" | "employer";
type View = "all" | "unread";

const SETTINGS: Record<CabinetRole, string> = {
  candidate: "/candidate/settings#notifications",
  employer: "/employer/company#notifications",
};

const DESCRIPTION: Record<CabinetRole, string> = {
  candidate: "Приглашения, ответы компаний, этапы найма и итоги тестов. Важное приходит и на почту — какие письма получать, решаете вы.",
  employer: "Ответы кандидатов, отклики, сбои доставки в ATS и изменения квоты. В письмах нет имён и контактов кандидатов — только анонимный код.",
};

export function NotificationFeed({ role }: { role: CabinetRole }) {
  const [view, setView] = useState<View>("all");
  const feed = useNotificationFeed(view === "unread");
  const unread = useUnreadCount();
  const markRead = useMarkRead();
  const count = unread.data?.count ?? 0;
  const items = feed.data?.pages.flatMap((p) => p.items) ?? [];

  return (
    <div className="max-w-3xl">
      <PageHeader
        title="Уведомления"
        description={DESCRIPTION[role]}
        actions={
          <ButtonLink href={SETTINGS[role]} variant="secondary" size="sm" icon={<Settings2 className="size-4" aria-hidden />}>
            Настроить письма
          </ButtonLink>
        }
      />
      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <Tabs<View>
          idPrefix="notifications"
          ariaLabel="Какие уведомления показать"
          value={view}
          onChange={setView}
          items={[
            { value: "all", label: "Все" },
            { value: "unread", label: "Непрочитанные", count },
          ]}
        />
        {count > 0 && (
          <Button variant="ghost" size="sm" icon={<CheckCheck className="size-4" aria-hidden />} loading={markRead.isPending} onClick={() => markRead.mutate({ all: true })}>
            Отметить всё прочитанным
          </Button>
        )}
      </div>

      <div {...tabPanelProps("notifications", view)}>
        {feed.isPending ? (
          <SkeletonCard lines={4} />
        ) : feed.isError ? (
          <ErrorAlert error={feed.error} onRetry={() => void feed.refetch()} />
        ) : items.length === 0 ? (
          <EmptyState
            icon={<BellOff className="size-5" aria-hidden />}
            title={view === "unread" ? "Всё прочитано" : "Уведомлений пока нет"}
            description={view === "unread" ? "Новые события появятся здесь и на значке колокольчика." : "Когда что-то произойдёт с вашими приглашениями и откликами, мы напишем сюда."}
          />
        ) : (
          <div className="overflow-hidden rounded-card border border-line bg-white shadow-card">
            <ul className="divide-y divide-line" data-testid="notification-feed">
              {items.map((n) => (
                <li key={n.id}>
                  <NotificationRow n={n} onOpen={() => !n.read_at && markRead.mutate({ ids: [n.id] })} />
                </li>
              ))}
            </ul>
          </div>
        )}
        {feed.hasNextPage && (
          <div className="mt-4 flex justify-center">
            <Button variant="secondary" loading={feed.isFetchingNextPage} onClick={() => void feed.fetchNextPage()}>
              Показать ещё
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}
