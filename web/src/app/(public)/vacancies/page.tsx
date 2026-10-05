import type { Metadata } from "next";
import { PublicVacancyBrowser } from "./PublicVacancyBrowser";

export const metadata: Metadata = { title: "Вакансии" };

export default function VacanciesPage() {
  return (
    <div className="mx-auto w-full max-w-6xl px-4 py-8 sm:px-6 sm:py-12">
      <header className="mb-8">
        <h1 className="font-mono text-2xl font-semibold tracking-tight text-ink sm:text-3xl">Вакансии</h1>
        <p className="mt-2 max-w-2xl text-ink-muted">Классический отклик — дополнительный путь. Основной — пройти тест и получать приглашения с вилкой.</p>
      </header>
      <PublicVacancyBrowser />
    </div>
  );
}
