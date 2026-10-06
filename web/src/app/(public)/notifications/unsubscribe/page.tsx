import type { Metadata } from "next";
import { Suspense } from "react";
import { Container } from "@/components/site/primitives";
import { Unsubscribe } from "./Unsubscribe";

export const metadata: Metadata = {
  title: { absolute: "Отписка от писем · ФСП Таланты" },
  description: "Отключить письма с уведомлениями ФСП Таланты по ссылке из письма.",
  robots: { index: false, follow: false },
};

export default function UnsubscribePage() {
  return (
    <div className="flex flex-1 flex-col bg-white">
      <Container className="pt-28 pb-20 sm:pt-32 lg:pb-28">
        <Suspense>
          <Unsubscribe />
        </Suspense>
      </Container>
    </div>
  );
}
