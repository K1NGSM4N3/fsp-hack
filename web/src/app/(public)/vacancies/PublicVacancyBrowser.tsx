"use client";

import { VacancyBrowser } from "@/components/domain/VacancyBrowser";

export function PublicVacancyBrowser() {
  return <VacancyBrowser hrefFor={(id) => `/vacancies/${id}`} />;
}
