"use client";

import { CabinetError } from "@/components/states/CabinetStates";

export default function CabinetErrorPage({ error, retry }: { error: Error & { digest?: string }; retry: () => void }) {
  return <CabinetError error={error} retry={retry} home="/employer" />;
}
