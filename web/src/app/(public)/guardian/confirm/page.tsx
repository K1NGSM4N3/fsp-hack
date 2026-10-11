import type { Metadata } from "next";
import { Suspense } from "react";
import { Container } from "@/components/site/primitives";
import { GuardianConfirm } from "./GuardianConfirm";

export const metadata: Metadata = {
  title: { absolute: "Согласие законного представителя · ФСП Таланты" },
  description: "Подтверждение или отзыв согласия родителя на участие подростка 14–17 лет в платформе ФСП Таланты.",
  robots: { index: false, follow: false },
};

export default function GuardianConfirmPage() {
  return (
    <div className="flex flex-1 flex-col bg-white">
      <Container className="pt-28 pb-20 sm:pt-32 lg:pb-28">
        <Suspense>
          <GuardianConfirm />
        </Suspense>
      </Container>
    </div>
  );
}
