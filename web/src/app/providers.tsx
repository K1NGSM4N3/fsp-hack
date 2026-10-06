"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { isApiError } from "@/api/client";
import { ToastProvider } from "@/components/ui";

let browserQueryClient: QueryClient | undefined;

/** Per-hook freshness lives in `fresh` (features/keys.ts); this default covers searches and anything unlabelled. */
function makeQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 30_000,
        gcTime: 10 * 60_000,
        refetchOnWindowFocus: false,
        retry: (count, error) => !(isApiError(error) && error.status >= 400 && error.status < 500) && count < 2,
      },
      mutations: { retry: false },
    },
  });
}

function getQueryClient() {
  if (typeof window === "undefined") return makeQueryClient();
  browserQueryClient ??= makeQueryClient();
  return browserQueryClient;
}

/**
 * Mounted by the segments that talk to the API (cabinets, auth, public vacancies, guardian consent), not by the root:
 * marketing pages ship neither the query client nor toasts. The browser client is a singleton, so the cache survives
 * moving between those segments.
 */
export function Providers({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <ToastProvider>{children}</ToastProvider>
    </QueryClientProvider>
  );
}
