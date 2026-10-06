"use client";

import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as api from "@/api/endpoints";
import type { VacancyQuery } from "@/api/types";
import { fresh, qk } from "../keys";

export function useVacancies(params: VacancyQuery) {
  return useQuery({ queryKey: qk.vacancies(params), queryFn: () => api.listVacancies(params), placeholderData: keepPreviousData, ...fresh.detail });
}

export function useVacancy(id: string) {
  return useQuery({ queryKey: qk.vacancy(id), queryFn: () => api.getVacancy(id), ...fresh.detail });
}

export function useApply() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, coverLetter }: { id: string; coverLetter: string }) => api.applyToVacancy(id, { cover_letter: coverLetter }),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.myApplications }),
  });
}
