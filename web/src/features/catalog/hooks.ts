"use client";

import { keepPreviousData, useQueries, useQuery } from "@tanstack/react-query";
import * as api from "@/api/endpoints";
import type { SkillView, Specialization } from "@/api/types";
import { SPECIALIZATIONS } from "@/lib/labels";
import { rememberSkillNames, searchSkillsLocal } from "@/lib/skills";
import { qk } from "../keys";

export function useCatalog() {
  return useQuery({ queryKey: qk.catalog, queryFn: api.getCatalog, staleTime: Infinity });
}

async function search(q: string, specialization?: Specialization): Promise<SkillView[]> {
  try {
    const list = await api.searchSkills(q, specialization);
    rememberSkillNames(list);
    return list;
  } catch {
    return searchSkillsLocal(q, specialization);
  }
}

/** Falls back to the bundled dictionary when the API is unreachable. */
export function useSkillSearch(q: string, enabled = true) {
  return useQuery({
    queryKey: qk.skills(q),
    queryFn: () => search(q),
    enabled,
    staleTime: 5 * 60_000,
    placeholderData: keepPreviousData,
  });
}

export function useTypicalSkills(specialization: Specialization | null) {
  return useQuery({
    queryKey: ["skills", "typical", specialization],
    queryFn: () => search("", specialization ?? undefined),
    enabled: Boolean(specialization),
    staleTime: Infinity,
  });
}

/** Warms the slug → name cache so stored slugs render with API display names. */
export function useSkillNamePreload() {
  useQueries({
    queries: SPECIALIZATIONS.map((s) => ({ queryKey: ["skills", "typical", s], queryFn: () => search("", s), staleTime: Infinity })),
  });
}
