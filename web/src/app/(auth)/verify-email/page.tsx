import type { Metadata } from "next";
import { Suspense } from "react";
import { Spinner } from "@/components/ui";
import { VerifyEmail } from "./VerifyEmail";

export const metadata: Metadata = { title: "Подтверждение email" };

export default function VerifyEmailPage() {
  return (
    <Suspense fallback={<Spinner label="Загрузка" />}>
      <VerifyEmail />
    </Suspense>
  );
}
