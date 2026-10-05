import { VacancyDetail } from "@/components/domain/VacancyDetail";

export default async function VacancyPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return (
    <div className="mx-auto w-full max-w-6xl px-4 py-8 sm:px-6 sm:py-12">
      <VacancyDetail id={id} backHref="/vacancies" backLabel="Все вакансии" />
    </div>
  );
}
