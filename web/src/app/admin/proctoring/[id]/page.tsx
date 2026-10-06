import { SessionReviewView } from "./SessionReviewView";

export default async function SessionReviewPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <SessionReviewView id={id} />;
}
