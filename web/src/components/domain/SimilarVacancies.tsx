"use client";

import { Building2 } from "lucide-react";
import Link from "next/link";
import { Card, CardHeader, Skeleton } from "@/components/ui";
import { useSimilarPublished } from "@/features/vacancies/hooks";
import { CategoryBadge, SalaryRange } from "./Category";

const CAPTION = {
  meaning: "По смыслу описания — сравнивает наша модель на нашем сервере. Сначала та же специализация.",
  skills: "Той же специализации, с общим стеком.",
} as const;

/** Nearest published vacancies; hidden while there are none. `hrefBase` is where a vacancy page lives in this part of the site. */
export function SimilarVacancies({ id, hrefBase }: { id: string; hrefBase: string }) {
  const query = useSimilarPublished(id);
  if (query.isPending) {
    return (
      <Card aria-busy>
        <Skeleton className="h-5 w-48" />
        <div className="mt-4 space-y-3">
          <Skeleton className="h-12 w-full" />
          <Skeleton className="h-12 w-full" />
        </div>
      </Card>
    );
  }
  if (query.isError || query.data.items.length === 0) return null;
  const { method, items } = query.data;
  return (
    <Card>
      <CardHeader title="Похожие вакансии" description={CAPTION[method]} />
      <ul className="divide-y divide-line" data-testid="similar-vacancies" data-method={method}>
        {items.map(({ vacancy: v }) => (
          <li key={v.id} className="flex flex-col gap-2 py-3 first:pt-0 last:pb-0 sm:flex-row sm:items-center sm:justify-between">
            <div className="min-w-0">
              <Link href={`${hrefBase}/${v.id}`} className="font-medium text-ink hover:text-primary hover:underline">
                {v.title}
              </Link>
              <p className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-ink-muted">
                <span className="inline-flex items-center gap-1.5">
                  <Building2 className="size-4 text-ink-subtle" aria-hidden />
                  {v.company_name}
                </span>
                <CategoryBadge category={{ specialization: v.specialization, grade: v.grade }} size="sm" />
              </p>
            </div>
            <SalaryRange salary={v.salary} size="sm" label="Зарплата" className="shrink-0" />
          </li>
        ))}
      </ul>
    </Card>
  );
}
