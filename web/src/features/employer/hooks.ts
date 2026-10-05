"use client";

import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as api from "@/api/endpoints";
import type {
  CandidateSearchParams,
  CompanyProfile,
  EmployerApplicationStatus,
  NeedInput,
  ReviewInput,
  SalaryCheckQuery,
  SendInvitation,
  ShortlistFilters,
  ShortlistPatch,
  SimilarInput,
  TaskInput,
  VacancyInput,
} from "@/api/types";
import { qk } from "../keys";

export function useTrustStatus() {
  return useQuery({ queryKey: qk.trust, queryFn: api.getTrustStatus });
}

/** `null` while the range is incomplete; callers debounce the inputs. */
export function useSalaryCheck(query: SalaryCheckQuery | null) {
  return useQuery({
    queryKey: query ? qk.salaryCheck(query) : ["employer", "salary-check", "idle"],
    queryFn: () => api.checkSalary(query as SalaryCheckQuery),
    enabled: query !== null,
    staleTime: 5 * 60_000,
    placeholderData: keepPreviousData,
    retry: false,
  });
}

export function useSimilarVacancies() {
  return useMutation({ mutationFn: (body: SimilarInput) => api.findSimilarVacancies(body) });
}

export function useCompany() {
  return useQuery({ queryKey: qk.company, queryFn: api.getCompany });
}

export function useUpdateCompany() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: CompanyProfile) => api.updateCompany(body),
    onSuccess: (data) => {
      qc.setQueryData(qk.company, data);
      void qc.invalidateQueries({ queryKey: qk.me });
      void qc.invalidateQueries({ queryKey: qk.trust });
      void qc.invalidateQueries({ queryKey: qk.stats });
    },
  });
}

export function useNeeds() {
  return useQuery({ queryKey: qk.needs, queryFn: api.listNeeds });
}

export function useNeed(id: string, enabled = true) {
  return useQuery({ queryKey: qk.need(id), queryFn: () => api.getNeed(id), enabled: enabled && Boolean(id) });
}

export function useCreateNeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: NeedInput) => api.createNeed(body),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.needs }),
  });
}

export function useUpdateNeed(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: NeedInput) => api.updateNeed(id, body),
    onSuccess: (data) => {
      qc.setQueryData(qk.need(id), data);
      void qc.invalidateQueries({ queryKey: qk.needs });
      void qc.invalidateQueries({ queryKey: qk.needCategories(id) });
    },
  });
}

export function useDeleteNeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteNeed(id),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.needs }),
  });
}

export function useParseNeed() {
  return useMutation({ mutationFn: (text: string) => api.parseNeed(text) });
}

export function useNeedCategories(id: string) {
  return useQuery({ queryKey: qk.needCategories(id), queryFn: () => api.getNeedCategories(id) });
}

export function useNeedTestPlan(id: string, enabled = true) {
  return useQuery({ queryKey: qk.needTestPlan(id), queryFn: () => api.getNeedTestPlan(id), enabled, staleTime: 10 * 60_000 });
}

export function useShortlist(id: string) {
  return useQuery({ queryKey: qk.shortlist(id), queryFn: () => api.getShortlist(id) });
}

export function useRefineShortlist(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (filters: ShortlistFilters) => api.refineShortlist(id, filters),
    onSuccess: (data) => {
      qc.setQueryData(qk.shortlist(id), data);
      qc.setQueryData(qk.need(id), data.need);
      void qc.invalidateQueries({ queryKey: qk.needs });
    },
  });
}

export function usePatchShortlist(needId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ candidateId, patch }: { candidateId: string; patch: ShortlistPatch }) => api.patchShortlistItem(needId, candidateId, patch),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.shortlist(needId) }),
  });
}

export function useCategoryStats() {
  return useQuery({ queryKey: qk.categories, queryFn: api.listCategories, staleTime: 5 * 60_000 });
}

export function useCandidateSearch(params: CandidateSearchParams) {
  return useQuery({ queryKey: qk.candidateSearch(params), queryFn: () => api.searchCandidates(params), placeholderData: keepPreviousData });
}

export function useEmployerCandidate(id: string, needId?: string | null) {
  return useQuery({ queryKey: qk.candidate(id, needId), queryFn: () => api.getCandidateForEmployer(id, needId) });
}

export function useCandidateContacts(id: string, enabled: boolean) {
  return useQuery({ queryKey: qk.contacts(id), queryFn: () => api.getCandidateContacts(id), enabled, staleTime: Infinity, retry: false });
}

export function useSentInvitations() {
  return useQuery({ queryKey: qk.sent, queryFn: api.listSentInvitations });
}

export function useSentInvitation(id: string) {
  return useQuery({ queryKey: qk.sentOne(id), queryFn: () => api.getSentInvitation(id) });
}

export function useSendInvitation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ body, key }: { body: SendInvitation; key: string }) => api.sendInvitation(body, key),
    onSuccess: (inv) => {
      void qc.invalidateQueries({ queryKey: qk.sent });
      void qc.invalidateQueries({ queryKey: qk.stats });
      void qc.invalidateQueries({ queryKey: qk.trust });
      void qc.invalidateQueries({ queryKey: ["employer", "candidate", inv.candidate_id] });
      if (inv.need_id) void qc.invalidateQueries({ queryKey: qk.shortlist(inv.need_id) });
    },
  });
}

export function useWithdrawInvitation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.withdrawInvitation(id),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: qk.sent });
      void qc.invalidateQueries({ queryKey: qk.stats });
    },
  });
}

export function useInvitationStats() {
  return useQuery({ queryKey: qk.stats, queryFn: api.getInvitationStats });
}

export function useOwnVacancies() {
  return useQuery({ queryKey: qk.ownVacancies, queryFn: api.listOwnVacancies });
}

export function useOwnVacancy(id: string) {
  return useQuery({ queryKey: qk.ownVacancy(id), queryFn: () => api.getOwnVacancy(id) });
}

export function useSaveVacancy(id?: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: VacancyInput) => (id ? api.updateVacancy(id, body) : api.createVacancy(body)),
    onSuccess: (v) => {
      qc.setQueryData(qk.ownVacancy(v.id), v);
      void qc.invalidateQueries({ queryKey: qk.ownVacancies });
      void qc.invalidateQueries({ queryKey: ["vacancies"] });
      void qc.invalidateQueries({ queryKey: qk.trust });
    },
  });
}

export function useVacancyApplications(vacancyId: string) {
  return useQuery({ queryKey: qk.applications(vacancyId), queryFn: () => api.listVacancyApplications(vacancyId) });
}

export function useSetApplicationStatus(vacancyId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, status }: { id: string; status: EmployerApplicationStatus }) => api.setApplicationStatus(id, status),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.applications(vacancyId) }),
  });
}

export function useEmployerTasks() {
  return useQuery({ queryKey: qk.tasks, queryFn: api.listEmployerTasks });
}

export function useCreateTask() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: TaskInput) => api.createEmployerTask(body),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.tasks }),
  });
}

export function useSetTaskActive() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, active }: { id: string; active: boolean }) => api.setTaskActive(id, active),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.tasks }),
  });
}

export function useTaskSubmissions(taskId: string, enabled = true) {
  return useQuery({ queryKey: qk.submissions(taskId), queryFn: () => api.listTaskSubmissions(taskId), enabled });
}

export function useReviewSubmission(taskId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ submissionId, body }: { submissionId: string; body: ReviewInput }) => api.reviewTaskSubmission(taskId, submissionId, body),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: qk.submissions(taskId) });
      void qc.invalidateQueries({ queryKey: qk.tasks });
    },
  });
}
