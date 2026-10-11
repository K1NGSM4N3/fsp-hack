"use client";

import { useQueryClient } from "@tanstack/react-query";
import { Bell, CheckCheck } from "lucide-react";
import Link from "next/link";
import { useEffect, useRef, useState } from "react";
import type { NotificationView } from "@/api/types";
import { IconButton, Spinner, useToast } from "@/components/ui";
import { qk } from "@/features/keys";
import { useLatestNotifications, useMarkRead, useUnreadCount } from "../hooks";
import { NotificationRow } from "./NotificationRow";

type CabinetRole = "candidate" | "employer";

const EMPTY: Record<CabinetRole, string> = {
  candidate: "Пока ничего нового. Здесь появятся приглашения компаний, ответы на отклики и итоги тестов.",
  employer: "Пока ничего нового. Здесь появятся ответы кандидатов, новые отклики и сбои интеграции.",
};

/**
 * Mounted once per cabinet: polls the badge. When it grows, the cabinet's own lists refresh too, and a brand-new
 * invitation gets one quiet toast. Items already in the first loaded list are never toasted, so a reload does not
 * replay old news.
 */
export function NotificationWatcher({ role }: { role: CabinetRole }) {
  const unread = useUnreadCount(true);
  const latest = useLatestNotifications();
  const qc = useQueryClient();
  const toast = useToast();
  const previous = useRef<number | null>(null);
  const known = useRef<Set<string> | null>(null);
  const count = unread.data?.count;

  useEffect(() => {
    if (count === undefined) return;
    if (previous.current !== null && count > previous.current) {
      void qc.invalidateQueries({ queryKey: qk.notificationsLatest });
      void qc.invalidateQueries({ queryKey: [...qk.notifications, "feed"] });
      void qc.invalidateQueries({ queryKey: role === "candidate" ? qk.received : qk.sent });
    }
    previous.current = count;
  }, [count, qc, role]);

  useEffect(() => {
    const items = latest.data?.items;
    if (!items) return;
    if (known.current === null) {
      known.current = new Set(items.map((n) => n.id));
      return;
    }
    for (const n of items) {
      if (known.current.has(n.id)) continue;
      known.current.add(n.id);
      if (n.kind === "invitation_received" && !n.read_at) toast.info(n.title, n.body);
    }
  }, [latest.data, toast]);

  return null;
}

export function NotificationBell({ role }: { role: CabinetRole }) {
  const count = useUnreadCount().data?.count ?? 0;
  const latest = useLatestNotifications();
  const markRead = useMarkRead();
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onPointer = (e: PointerEvent) => {
      if (root.current && !root.current.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", onPointer);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onPointer);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  const toggle = () => {
    if (!open) void latest.refetch();
    setOpen(!open);
  };
  const openItem = (n: NotificationView) => {
    if (!n.read_at) markRead.mutate({ ids: [n.id] });
    setOpen(false);
  };
  const items = latest.data?.items ?? [];
  const badge = count > 99 ? "99+" : String(count);

  return (
    <div ref={root} className="relative">
      <IconButton
        label={count ? `Уведомления, непрочитанных: ${count}` : "Уведомления"}
        aria-expanded={open}
        aria-haspopup="dialog"
        onClick={toggle}
        className="relative"
        data-testid="notification-bell"
      >
        <Bell className="size-5" aria-hidden />
        {count > 0 && (
          <span
            data-testid="notification-badge"
            className="tabular absolute top-1 right-1 flex h-4 min-w-4 items-center justify-center rounded-full bg-accent px-1 font-mono text-[10px] leading-none font-semibold text-white ring-2 ring-white"
            aria-hidden
          >
            {badge}
          </span>
        )}
      </IconButton>

      {open && (
        <div
          role="dialog"
          aria-label="Уведомления"
          className="fixed inset-x-3 top-16 z-50 flex max-h-[min(34rem,calc(100dvh-5rem))] animate-[toast-in_260ms_var(--ease-out-expo)_both] flex-col overflow-hidden rounded-card border border-line bg-white shadow-pop sm:absolute sm:inset-x-auto sm:top-[calc(100%+0.5rem)] sm:right-0 sm:w-[25rem]"
        >
          <div className="flex items-center justify-between gap-3 border-b border-line px-4 py-3">
            <p className="text-sm font-semibold text-ink">Уведомления</p>
            {count > 0 && (
              <button
                type="button"
                onClick={() => markRead.mutate({ all: true })}
                disabled={markRead.isPending}
                className="tap inline-flex items-center gap-1.5 rounded text-xs font-medium text-primary hover:underline disabled:opacity-50"
              >
                <CheckCheck className="size-3.5" aria-hidden />
                Отметить всё прочитанным
              </button>
            )}
          </div>
          <div className="scrollbar-thin min-h-0 flex-1 overflow-y-auto">
            {latest.isPending ? (
              <div className="flex justify-center py-10">
                <Spinner className="size-5 text-primary" />
              </div>
            ) : items.length === 0 ? (
              <p className="px-6 py-10 text-center text-sm text-ink-muted">{EMPTY[role]}</p>
            ) : (
              <ul className="divide-y divide-line">
                {items.map((n) => (
                  <li key={n.id}>
                    <NotificationRow n={n} compact onOpen={() => openItem(n)} />
                  </li>
                ))}
              </ul>
            )}
          </div>
          <div className="border-t border-line px-4 py-2.5">
            <Link href={`/${role}/notifications`} onClick={() => setOpen(false)} className="tap inline-flex rounded text-sm font-medium text-primary hover:underline">
              Все уведомления
            </Link>
          </div>
        </div>
      )}
    </div>
  );
}
