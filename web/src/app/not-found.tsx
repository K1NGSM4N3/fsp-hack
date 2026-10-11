import type { Metadata } from "next";
import Link from "next/link";
import { SiteState, stateButton } from "@/components/states/SiteState";

export const metadata: Metadata = { title: "Страница не найдена", robots: { index: false } };

export default function NotFound() {
  return (
    <SiteState
      code="404"
      title="Такой страницы нет"
      text="Возможно, ссылка устарела или в адресе опечатка. Вакансию могли снять с публикации — остальные открытые вакансии в общем списке."
      actions={
        <>
          <Link href="/" className={`${stateButton} bg-primary hover:bg-primary-hover`}>
            На главную
          </Link>
          <Link href="/vacancies" className={`${stateButton} border border-white/25 hover:border-white hover:bg-white/5`}>
            Все вакансии
          </Link>
        </>
      }
    />
  );
}
