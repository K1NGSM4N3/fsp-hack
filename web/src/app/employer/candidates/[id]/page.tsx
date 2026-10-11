import { EmployerCandidateView } from "./EmployerCandidateView";

export default async function EmployerCandidatePage({ params, searchParams }: { params: Promise<{ id: string }>; searchParams: Promise<Record<string, string | string[] | undefined>> }) {
  const { id } = await params;
  const sp = await searchParams;
  const need = typeof sp.need === "string" ? sp.need : null;
  const vacancy = typeof sp.vacancy === "string" ? sp.vacancy : null;
  return <EmployerCandidateView id={id} needId={need} vacancyId={vacancy} />;
}
