import { VacancyDetail } from "@/components/domain/VacancyDetail";
import { Container } from "@/components/site/primitives";

export default async function VacancyPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return (
    <div className="flex flex-1 flex-col bg-canvas">
      <Container className="pt-28 pb-14 sm:pt-32">
        <VacancyDetail id={id} backHref="/vacancies" backLabel="Все вакансии" />
      </Container>
    </div>
  );
}
