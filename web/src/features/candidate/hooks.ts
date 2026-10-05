"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as api from "@/api/endpoints";
import type { AnswerInput, DeclineRequest, Privacy, ProfileInput, StartInput, SurveyInput } from "@/api/types";
import { qk } from "../keys";

export function useProfile() {
  return useQuery({ queryKey: qk.profile, queryFn: api.getProfile });
}

export function useUpdateProfile() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: ProfileInput) => api.updateProfile(body),
    onSuccess: (data) => {
      qc.setQueryData(qk.profile, data);
      void qc.invalidateQueries({ queryKey: qk.me });
    },
  });
}

export function useUploadResume() {
  return useMutation({ mutationFn: (file: File) => api.uploadResume(file) });
}

export function usePrivacy() {
  return useQuery({ queryKey: qk.privacy, queryFn: api.getPrivacy });
}

export function useUpdatePrivacy() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: Privacy) => api.updatePrivacy(body),
    onSuccess: (data) => qc.setQueryData(qk.privacy, data),
  });
}

export function useSurveyHistory() {
  return useQuery({ queryKey: qk.surveys, queryFn: api.getSurveyHistory });
}

export function useSubmitSurvey() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: SurveyInput) => api.submitSurvey(body),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: qk.surveys });
      void qc.invalidateQueries({ queryKey: qk.me });
      void qc.invalidateQueries({ queryKey: qk.category });
      void qc.invalidateQueries({ queryKey: qk.profile });
    },
  });
}

export function useCategory() {
  return useQuery({ queryKey: qk.category, queryFn: api.getCandidateCategory });
}

export function useAssessments() {
  return useQuery({ queryKey: qk.assessments, queryFn: api.listAssessments });
}

/** Reads through `current-item`, which also advances a timed-out item. */
export function useAssessment(id: string) {
  return useQuery({ queryKey: qk.assessment(id), queryFn: () => api.getCurrentItem(id), staleTime: Infinity, retry: false });
}

export function useStartAssessment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: StartInput) => api.startAssessment(body),
    onSuccess: (session) => {
      qc.setQueryData(qk.assessment(session.id), session);
      void qc.invalidateQueries({ queryKey: qk.assessments });
      void qc.invalidateQueries({ queryKey: qk.me });
    },
  });
}

export function useAnswerItem(sessionId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: AnswerInput) => api.answerItem(sessionId, body),
    onSuccess: (session) => {
      qc.setQueryData(qk.assessment(sessionId), session);
      if (session.status !== "in_progress") {
        void qc.invalidateQueries({ queryKey: qk.category });
        void qc.invalidateQueries({ queryKey: qk.assessments });
        void qc.invalidateQueries({ queryKey: qk.me });
        void qc.invalidateQueries({ queryKey: qk.profile });
      }
    },
  });
}

export function useReceivedInvitations() {
  return useQuery({ queryKey: qk.received, queryFn: api.listReceivedInvitations });
}

export function useReceivedInvitation(id: string) {
  const qc = useQueryClient();
  return useQuery({
    queryKey: qk.receivedOne(id),
    queryFn: async () => {
      const inv = await api.getReceivedInvitation(id);
      void qc.invalidateQueries({ queryKey: qk.received });
      return inv;
    },
  });
}

export function useAcceptInvitation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.acceptInvitation(id),
    onSuccess: (inv) => {
      qc.setQueryData(qk.receivedOne(inv.id), inv);
      void qc.invalidateQueries({ queryKey: qk.received });
    },
  });
}

export function useDeclineInvitation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, body }: { id: string; body: DeclineRequest }) => api.declineInvitation(id, body),
    onSuccess: (inv) => {
      qc.setQueryData(qk.receivedOne(inv.id), inv);
      void qc.invalidateQueries({ queryKey: qk.received });
      void qc.invalidateQueries({ queryKey: qk.privacy });
    },
  });
}

export function useMarket(enabled = true) {
  return useQuery({ queryKey: qk.market, queryFn: api.getCandidateMarket, enabled });
}

export function useMyApplications(enabled = true) {
  return useQuery({ queryKey: qk.myApplications, queryFn: api.listMyApplications, enabled });
}

export function useWithdrawApplication() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.withdrawApplication(id),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.myApplications }),
  });
}

export function useMyTasks() {
  return useQuery({ queryKey: qk.myTasks, queryFn: api.listMyTasks });
}

export function useSubmitTask() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, answer }: { id: string; answer: string }) => api.submitTask(id, { answer }),
    onSuccess: (tasks) => qc.setQueryData(qk.myTasks, tasks),
  });
}

export function useSkipTask() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.skipTask(id),
    onSuccess: () => void qc.invalidateQueries({ queryKey: qk.myTasks }),
  });
}

export function useFspStatus() {
  return useQuery({ queryKey: qk.fsp, queryFn: api.getFspStatus });
}

function useFspMutation<A, R>(fn: (arg: A) => Promise<R>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: fn,
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: qk.fsp });
      void qc.invalidateQueries({ queryKey: qk.profile });
    },
  });
}

export function useFspLinkDev() {
  return useFspMutation((fspId: string) => api.fspLinkDev(fspId));
}

export function useFspSync() {
  return useFspMutation(() => api.syncFsp());
}

export function useFspUnlink() {
  return useFspMutation(() => api.unlinkFsp());
}
