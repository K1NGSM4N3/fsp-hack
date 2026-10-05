"use client";

import type { UseQueryResult } from "@tanstack/react-query";
import { RotateCw } from "lucide-react";
import type { ReactNode } from "react";
import { isApiError } from "@/api/client";
import { Button } from "./Button";
import { Alert, PageSkeleton } from "./Feedback";

export function errorText(error: unknown): { title: string; detail: string | null } {
  if (isApiError(error)) return { title: error.title, detail: error.detail };
  return { title: "Что-то пошло не так", detail: error instanceof Error ? error.message : null };
}

export function ErrorAlert({ error, onRetry, className }: { error: unknown; onRetry?: () => void; className?: string }) {
  const { title, detail } = errorText(error);
  return (
    <Alert
      tone="danger"
      title={title}
      className={className}
      action={
        onRetry ? (
          <Button variant="secondary" size="sm" icon={<RotateCw className="size-4" aria-hidden />} onClick={onRetry}>
            Повторить
          </Button>
        ) : undefined
      }
    >
      {detail}
    </Alert>
  );
}

interface QueryStateProps<T> {
  query: UseQueryResult<T>;
  loading?: ReactNode;
  children: (data: T) => ReactNode;
}

/** Uniform loading / error handling for a single query. */
export function QueryState<T>({ query, loading, children }: QueryStateProps<T>) {
  if (query.isPending) return <>{loading ?? <PageSkeleton />}</>;
  if (query.isError) return <ErrorAlert error={query.error} onRetry={() => void query.refetch()} />;
  return <>{children(query.data)}</>;
}
