"use client";

import type { Page, PublicVacancy } from "@/api/types";
import { VacancyBrowser } from "@/components/domain/VacancyBrowser";
import type { Seed } from "@/features/vacancies/hooks";

export function PublicVacancyBrowser({ seed }: { seed: Seed<Page<PublicVacancy>> | null }) {
  return <VacancyBrowser hrefFor={(id) => `/vacancies/${id}`} seed={seed} />;
}
