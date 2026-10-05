"use client";

import { useRouter } from "next/navigation";
import { PageHeader } from "@/components/ui";
import { VacancyForm } from "@/features/employer/components/VacancyForm";

export default function NewVacancyPage() {
  const router = useRouter();
  return (
    <div className="max-w-4xl">
      <PageHeader back={{ href: "/employer/vacancies", label: "Вакансии" }} title="Новая вакансия" />
      <VacancyForm onSaved={(v) => router.replace(`/employer/vacancies/${v.id}`)} />
    </div>
  );
}
