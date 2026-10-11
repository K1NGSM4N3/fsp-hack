"use client";

import { useRouter } from "next/navigation";
import { PageHeader } from "@/components/ui";
import { VacancyForm } from "@/features/employer/components/VacancyForm";

export default function NewVacancyPage() {
  const router = useRouter();
  return (
    <div className="max-w-6xl">
      <PageHeader
        back={{ href: "/employer/vacancies", label: "Вакансии" }}
        eyebrow="Найм"
        title="Новая вакансия"
        description="Вакансия попадёт в общий список: соискатели откликнутся сами, а их контакты станут доступны сразу после отклика."
      />
      <VacancyForm onSaved={(v) => router.replace(`/employer/vacancies/${v.id}`)} onCancel={() => router.push("/employer/vacancies")} />
    </div>
  );
}
