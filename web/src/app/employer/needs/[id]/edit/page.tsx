import { EditNeed } from "./EditNeed";

export default async function EditNeedPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <EditNeed id={id} />;
}
