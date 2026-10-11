import { AssessmentSessionView } from "./AssessmentSessionView";

export default async function AssessmentSessionPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <AssessmentSessionView id={id} />;
}
