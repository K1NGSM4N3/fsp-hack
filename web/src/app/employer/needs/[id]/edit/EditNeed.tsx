"use client";

import { useRouter } from "next/navigation";
import { isApiError } from "@/api/client";
import type { HiringNeed } from "@/api/types";
import { ErrorAlert, PageHeader, PageSkeleton, useToast } from "@/components/ui";
import { useNeed, useUpdateNeed } from "@/features/employer/hooks";
import { NeedForm, type NeedDraft } from "@/features/employer/components/NeedForm";

function toDraft(n: HiringNeed): NeedDraft {
  return {
    title: n.title,
    specialization: n.specialization,
    grades: n.grades,
    allow_adjacent_grades: n.allow_adjacent_grades,
    must_skills: n.must_skills,
    nice_skills: n.nice_skills,
    min: n.salary.min,
    max: n.salary.max,
    work_formats: n.work_formats,
    city: n.city ?? "",
    description: n.description,
    industries: n.industries,
  };
}

export function EditNeed({ id }: { id: string }) {
  const router = useRouter();
  const toast = useToast();
  const q = useNeed(id);
  const update = useUpdateNeed(id);
  if (q.isPending) return <PageSkeleton cards={2} />;
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  return (
    <div>
      <PageHeader back={{ href: `/employer/needs/${id}`, label: q.data.title }} title="Изменить потребность" description="После сохранения подборка пересчитается, закреплённые кандидаты останутся." />
      <NeedForm
        initial={toDraft(q.data)}
        submitLabel="Сохранить"
        pending={update.isPending}
        serverError={isApiError(update.error) ? update.error : null}
        onCancel={() => router.back()}
        onSubmit={(input) =>
          update.mutate(input, {
            onSuccess: () => {
              toast.success("Потребность обновлена");
              router.push(`/employer/needs/${id}`);
            },
          })
        }
      />
    </div>
  );
}
