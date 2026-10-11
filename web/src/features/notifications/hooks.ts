"use client";

import { useInfiniteQuery, useMutation, useQuery, useQueryClient, type InfiniteData } from "@tanstack/react-query";
import * as api from "@/api/endpoints";
import type { MarkRead, NotificationPage, NotificationSettingsInput } from "@/api/types";
import { fresh, qk } from "../keys";

/** Half a minute is quick enough for an invitation; TanStack pauses the interval while the tab is hidden. */
export const UNREAD_POLL_MS = 30_000;
const LATEST_LIMIT = 8;
const PAGE = 20;

/** Only the cabinet's single watcher polls (`poll`); the bells read the same cache. */
export function useUnreadCount(poll = false) {
  return useQuery({
    queryKey: qk.unreadCount,
    queryFn: api.getUnreadCount,
    refetchInterval: poll ? UNREAD_POLL_MS : false,
    refetchIntervalInBackground: false,
    refetchOnWindowFocus: poll,
    staleTime: 10_000,
  });
}

/** The bell's list. Refetched when the badge grows, not on a timer of its own. */
export function useLatestNotifications() {
  return useQuery({ queryKey: qk.notificationsLatest, queryFn: () => api.listNotifications({ limit: LATEST_LIMIT }), staleTime: Infinity });
}

export function useNotificationFeed(unread: boolean) {
  return useInfiniteQuery({
    queryKey: qk.notificationFeed(unread),
    queryFn: ({ pageParam }) => api.listNotifications({ unread: unread || undefined, cursor: pageParam, limit: PAGE }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.next_cursor ?? undefined,
    ...fresh.live,
  });
}

function markPage(page: NotificationPage, body: MarkRead, at: string): NotificationPage {
  const ids = new Set(body.ids ?? []);
  return { ...page, items: page.items.map((n) => (!n.read_at && (body.all || ids.has(n.id)) ? { ...n, read_at: at } : n)) };
}

/** Optimistic: the dot goes away at once, the server count follows. */
export function useMarkRead() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: MarkRead) => api.markNotificationsRead(body),
    onMutate: (body) => {
      const at = new Date().toISOString();
      qc.setQueryData<NotificationPage>(qk.notificationsLatest, (page) => page && markPage(page, body, at));
      qc.setQueriesData<InfiniteData<NotificationPage>>({ queryKey: [...qk.notifications, "feed"] }, (data) =>
        data ? { ...data, pages: data.pages.map((p) => markPage(p, body, at)) } : data,
      );
    },
    onSuccess: (count) => qc.setQueryData(qk.unreadCount, count),
    onSettled: () => void qc.invalidateQueries({ queryKey: qk.notifications }),
  });
}

export function useNotificationSettings() {
  return useQuery({ queryKey: qk.notificationSettings, queryFn: api.getNotificationSettings, ...fresh.account });
}

export function useUpdateNotificationSettings() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: NotificationSettingsInput) => api.updateNotificationSettings(body),
    onSuccess: (data) => qc.setQueryData(qk.notificationSettings, data),
  });
}

export function useUnsubscribe() {
  return useMutation({ mutationFn: (token: string) => api.unsubscribe(token) });
}
