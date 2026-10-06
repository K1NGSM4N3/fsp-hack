"use client";

import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as api from "@/api/admin";
import type { AuditFilter, QueueFilter, ReviewDecision, TrustActionInput, TrustItemKind, TrustView } from "@/api/admin";
import { fresh } from "../keys";

/** Moderator data lives under its own root, so a decision refreshes every queue and counter at once. */
export const mk = {
  root: ["admin"] as const,
  overview: ["admin", "overview"] as const,
  queue: (filter: QueueFilter) => ["admin", "proctoring", filter] as const,
  session: (id: string) => ["admin", "session", id] as const,
  trust: (view: TrustView, kind: TrustItemKind | null) => ["admin", "antifake", view, kind] as const,
  log: (f: AuditFilter) => ["admin", "log", f] as const,
};

export function useOverview() {
  return useQuery({ queryKey: mk.overview, queryFn: api.getOverview, ...fresh.live });
}

export function useProctoringQueue(filter: QueueFilter) {
  return useInfiniteQuery({
    queryKey: mk.queue(filter),
    queryFn: ({ pageParam }) => api.listProctoringQueue(filter, pageParam),
    initialPageParam: null as string | null,
    getNextPageParam: (last) => last.next_cursor ?? null,
    ...fresh.live,
  });
}

export function useSessionReview(id: string) {
  return useQuery({ queryKey: mk.session(id), queryFn: () => api.getSessionReview(id), ...fresh.detail });
}

export function useDecide(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ decision, comment }: { decision: ReviewDecision; comment: string }) => api.decideSession(id, decision, comment),
    onSuccess: (review) => {
      qc.setQueryData(mk.session(id), review);
      void qc.invalidateQueries({ queryKey: mk.root, predicate: (q) => q.queryKey[1] !== "session" });
    },
  });
}

export function useTrustQueue(view: TrustView, kind: TrustItemKind | null) {
  return useInfiniteQuery({
    queryKey: mk.trust(view, kind),
    queryFn: ({ pageParam }) => api.listTrustQueue(view, kind, pageParam),
    initialPageParam: null as string | null,
    getNextPageParam: (last) => last.next_cursor ?? null,
    ...fresh.live,
  });
}

export function useTrustAction() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: TrustActionInput) => api.actOnTrustItem(body),
    onSuccess: () => void qc.invalidateQueries({ queryKey: mk.root }),
  });
}

export function useAuditLog(filter: AuditFilter) {
  return useInfiniteQuery({
    queryKey: mk.log(filter),
    queryFn: ({ pageParam }) => api.listAudit(filter, pageParam),
    initialPageParam: null as string | null,
    getNextPageParam: (last) => last.next_cursor ?? null,
    ...fresh.live,
  });
}
