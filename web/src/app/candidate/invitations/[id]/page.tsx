import { InvitationDetail } from "./InvitationDetail";

export default async function InvitationPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <InvitationDetail id={id} />;
}
