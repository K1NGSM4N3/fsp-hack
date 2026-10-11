import { VacancyDetail } from "@/components/domain/VacancyDetail";

export default async function CandidateVacancyPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <VacancyDetail id={id} backHref="/candidate/vacancies" backLabel="Вакансии и отклики" />;
}
