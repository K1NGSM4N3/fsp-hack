"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as api from "@/api/endpoints";
import type { LoginRequest, RegisterRequest } from "@/api/types";
import { qk } from "../keys";

export function useMe(enabled = true) {
  return useQuery({ queryKey: qk.me, queryFn: api.getMe, enabled, retry: false, staleTime: 60_000 });
}

export function useConsentDocuments() {
  return useQuery({ queryKey: qk.consents, queryFn: api.getCurrentConsents, staleTime: 10 * 60_000 });
}

export function useLogin() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: LoginRequest) => api.login(body),
    onSuccess: async () => {
      qc.clear();
      await qc.prefetchQuery({ queryKey: qk.me, queryFn: api.getMe });
    },
  });
}

export function useRegister() {
  return useMutation({ mutationFn: (body: RegisterRequest) => api.register(body) });
}

export function useVerifyEmail() {
  return useMutation({ mutationFn: (token: string) => api.verifyEmail(token) });
}

export function useResendVerification() {
  return useMutation({ mutationFn: (email: string) => api.resendVerification(email) });
}

export function useLogout() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: api.logout,
    onSettled: () => qc.clear(),
  });
}

export function useMyConsents() {
  return useQuery({ queryKey: qk.myConsents, queryFn: api.getMyConsents });
}

export function useRevokeConsent() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (documentId: string) => api.revokeConsent(documentId),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: qk.myConsents });
      void qc.invalidateQueries({ queryKey: qk.privacy });
    },
  });
}

export function useGrantConsent() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (documentId: string) => api.grantConsent(documentId),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.myConsents }),
  });
}

function saveFile(blob: Blob, name: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  URL.revokeObjectURL(url);
}

/** Downloads `fsp-talents-<date>.json` with everything the platform stores about the account. */
export function useExportMyData() {
  return useMutation({
    mutationFn: api.exportMyData,
    onSuccess: (blob) => saveFile(blob, `fsp-talents-${new Date().toISOString().slice(0, 10)}.json`),
  });
}

export function useDeleteAccount() {
  const qc = useQueryClient();
  return useMutation({ mutationFn: api.deleteMe, onSuccess: () => qc.clear() });
}
