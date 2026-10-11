"use client";

import { useMutation } from "@tanstack/react-query";
import * as api from "@/api/endpoints";
import type { ReportInput } from "@/api/types";

export function useCreateReport() {
  return useMutation({ mutationFn: (body: ReportInput) => api.createReport(body) });
}
