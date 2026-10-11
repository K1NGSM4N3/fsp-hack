import { EmployerVacancyView } from "./EmployerVacancyView";

export default async function EmployerVacancyPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <EmployerVacancyView id={id} />;
}
