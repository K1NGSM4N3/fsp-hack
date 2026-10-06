"use client";

import { AlertTriangle, RotateCw } from "lucide-react";
import { Button, ButtonLink, Card, Skeleton } from "@/components/ui";

/** Route-level loading inside a cabinet: the page header and two rows of cards, in the shape most sections have. */
export function CabinetLoading() {
  return (
    <div role="status" aria-live="polite" className="animate-fade-in">
      <span className="sr-only">Загружаем раздел…</span>
      <div aria-hidden className="mb-9">
        <Skeleton className="h-3 w-24" />
        <Skeleton className="mt-4 h-8 w-72 max-w-full" />
        <Skeleton className="mt-3 h-4 w-[32rem] max-w-full" />
      </div>
      <div aria-hidden className="grid gap-4 md:grid-cols-3">
        {[0, 1, 2].map((i) => (
          <div key={i} className="rounded-card border border-line bg-white p-5">
            <Skeleton className="h-3 w-28" />
            <Skeleton className="mt-4 h-7 w-20" />
            <Skeleton className="mt-3 h-3 w-36" />
          </div>
        ))}
      </div>
      <div aria-hidden className="mt-4 grid gap-4 lg:grid-cols-[1.6fr_1fr]">
        {[5, 3].map((lines, i) => (
          <div key={i} className="rounded-card border border-line bg-white p-5">
            <Skeleton className="h-5 w-48" />
            <div className="mt-5 space-y-3">
              {Array.from({ length: lines }, (_, j) => (
                <Skeleton key={j} className={j === lines - 1 ? "h-3.5 w-3/5" : "h-3.5 w-full"} />
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

/** Error boundary of a cabinet section: the shell and navigation stay, only the page is replaced. */
export function CabinetError({ error, retry, home }: { error: Error & { digest?: string }; retry: () => void; home: string }) {
  return (
    <div className="mx-auto max-w-2xl pt-6">
      <Card padding="lg" className="text-center">
        <span className="mx-auto flex size-12 items-center justify-center rounded-2xl bg-accent-50 text-accent-strong ring-8 ring-accent-50/50">
          <AlertTriangle className="size-5" aria-hidden />
        </span>
        <h1 className="mt-5 text-2xl font-semibold tracking-[-0.03em] text-ink">Раздел не загрузился</h1>
        <p className="mx-auto mt-2 max-w-md text-sm leading-relaxed text-ink-muted">
          Данные никуда не делись — сломалось отображение этой страницы. Попробуйте ещё раз; если повторится, откройте другой раздел и вернитесь.
        </p>
        {error.digest && (
          <p className="mt-4 text-xs text-ink-subtle">
            Код для поддержки: <span className="font-mono text-ink">{error.digest}</span>
          </p>
        )}
        <div className="mt-7 flex flex-wrap justify-center gap-2">
          <Button icon={<RotateCw className="size-4" aria-hidden />} onClick={retry}>
            Попробовать снова
          </Button>
          <ButtonLink href={home} variant="secondary">
            На главную кабинета
          </ButtonLink>
        </div>
      </Card>
    </div>
  );
}
