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

/**
 * The server matches case- and space-insensitively, so «Go», «go» and «go » share one key and one request. An empty query
 * asks nothing: without a specialization the API has no list to give. Falls back to the bundled dictionary when the
 * API is unreachable.
 */
export function useSkillSearch(q: string, enabled = true) {
  const key = q.trim().toLowerCase();
  return useQuery({
    queryKey: qk.skills(key),
    queryFn: () => search(key),
    enabled: enabled && key.length > 0,
    staleTime: 5 * 60_000,
    placeholderData: keepPreviousData,
  });
}

const typical = (s: Specialization) => ({ queryKey: qk.typicalSkills(s), queryFn: () => search("", s), staleTime: Infinity });

export function useTypicalSkills(specialization: Specialization | null) {
  return useQuery({
    queryKey: qk.typicalSkills(specialization ?? "none"),
    queryFn: () => search("", specialization ?? undefined),
    enabled: Boolean(specialization),
    staleTime: Infinity,
  });
}

/** Warms the slug → name cache so stored slugs render with API display names; the same keys as `useTypicalSkills`. */
export function useSkillNamePreload() {
  useQueries({ queries: SPECIALIZATIONS.map(typical) });
}
