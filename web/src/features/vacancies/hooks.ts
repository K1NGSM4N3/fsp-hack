"use client";

import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as api from "@/api/endpoints";
import type { Page, PublicVacancy, VacancyQuery } from "@/api/types";
import { fresh, qk } from "../keys";

/** Server-rendered data the client starts from, with the time it was fetched so staleness is judged correctly. */
export interface Seed<T> {
  data: T;
  at: number;
}

export function useVacancies(params: VacancyQuery, seed?: Seed<Page<PublicVacancy>> | null) {
  return useQuery({
    queryKey: qk.vacancies(params),
    queryFn: () => api.listVacancies(params),
    placeholderData: keepPreviousData,
    initialData: seed?.data,
    initialDataUpdatedAt: seed?.at,
    ...fresh.detail,
  });
}

export function useVacancy(id: string, seed?: Seed<PublicVacancy> | null) {
  return useQuery({ queryKey: qk.vacancy(id), queryFn: () => api.getVacancy(id), initialData: seed?.data, initialDataUpdatedAt: seed?.at, ...fresh.detail });
}

/** «Похожие вакансии»: by meaning when the texts' embeddings are ready, otherwise by shared skills. */
export function useSimilarPublished(id: string) {
  return useQuery({ queryKey: qk.similarVacancies(id), queryFn: () => api.getSimilarVacancies(id, 4), ...fresh.detail });
}

export function useApply() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, coverLetter }: { id: string; coverLetter: string }) => api.applyToVacancy(id, { cover_letter: coverLetter }),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.myApplications }),
  });
}
