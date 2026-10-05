"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as api from "@/api/endpoints";
import type { ApiKeyInput, WebhookInput } from "@/api/types";
import { qk } from "../keys";

export function useIntegrations() {
  return useQuery({ queryKey: qk.integrations, queryFn: api.getIntegrations });
}

/** Pending deliveries are retried in the background, so the log refreshes while any are queued. */
export function useDeliveries() {
  return useQuery({
    queryKey: qk.deliveries,
    queryFn: () => api.listDeliveries(50),
    refetchInterval: (q) => (q.state.data?.some((d) => d.status === "pending") ? 5_000 : 30_000),
  });
}

export function useApiKeys() {
  return useQuery({ queryKey: qk.apiKeys, queryFn: api.listApiKeys });
}

function useInvalidateIntegrations() {
  const qc = useQueryClient();
  return () => void qc.invalidateQueries({ queryKey: qk.integrations });
}

export function useSaveWebhook() {
  const invalidate = useInvalidateIntegrations();
  return useMutation({ mutationFn: (body: WebhookInput) => api.saveWebhook(body), onSuccess: invalidate });
}

export function useDeleteWebhook() {
  const invalidate = useInvalidateIntegrations();
  return useMutation({ mutationFn: api.deleteWebhook, onSuccess: invalidate });
}

export function useRotateWebhookSecret() {
  const invalidate = useInvalidateIntegrations();
  return useMutation({ mutationFn: api.rotateWebhookSecret, onSuccess: invalidate });
}

export function useSendTestEvent() {
  const invalidate = useInvalidateIntegrations();
  return useMutation({ mutationFn: api.sendTestEvent, onSettled: invalidate });
}

export function useRetryDelivery() {
  const invalidate = useInvalidateIntegrations();
  return useMutation({ mutationFn: (eventId: string) => api.retryDelivery(eventId), onSuccess: invalidate });
}

export function useCreateApiKey() {
  const invalidate = useInvalidateIntegrations();
  return useMutation({ mutationFn: (body: ApiKeyInput) => api.createApiKey(body), onSuccess: invalidate });
}

export function useRevokeApiKey() {
  const invalidate = useInvalidateIntegrations();
  return useMutation({ mutationFn: (id: string) => api.revokeApiKey(id), onSuccess: invalidate });
}
