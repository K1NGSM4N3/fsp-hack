"use client";

import { BadgeCheck, Plus, Trophy } from "lucide-react";
import { useId, useMemo, useRef, useState, type KeyboardEvent } from "react";
import type { ProfileSkill, SkillSource, SkillView } from "@/api/types";
import { Chip } from "@/components/ui";
import { useFieldControl } from "@/components/ui/Field";
import { controlBase } from "@/components/ui/Field";
import { useSkillSearch } from "@/features/catalog/hooks";
import { cn } from "@/lib/cn";
import { SKILL_SOURCE_LABEL, strongestSource } from "@/lib/labels";
import { skillName } from "@/lib/skills";

const SOURCE_STYLE: Record<SkillSource, "primary" | "success" | "neutral"> = { test: "success", fsp: "primary", resume: "neutral", declared: "neutral" };

/** Verified skills first; the icon tells the source (test ✓, FSP trophy). */
export function SkillList({ skills, highlight, limit, className }: { skills: ProfileSkill[]; highlight?: string[]; limit?: number; className?: string }) {
  const order: Record<SkillSource, number> = { test: 0, fsp: 1, resume: 2, declared: 3 };
  const sorted = [...skills].sort((a, b) => order[strongestSource(a.sources)] - order[strongestSource(b.sources)]);
  const shown = limit ? sorted.slice(0, limit) : sorted;
  const rest = sorted.length - shown.length;
  const hl = new Set(highlight ?? []);
  return (
    <ul className={cn("flex flex-wrap gap-1.5", className)} aria-label="Навыки">
      {shown.map((s) => {
        const src = strongestSource(s.sources);
        const tone = hl.has(s.slug) ? "primary" : SOURCE_STYLE[src];
        const icon = src === "test" ? <BadgeCheck className="size-3.5" aria-hidden /> : src === "fsp" ? <Trophy className="size-3.5" aria-hidden /> : null;
        return (
          <li key={s.slug}>
            <Chip tone={tone} icon={icon} title={s.sources.map((x) => SKILL_SOURCE_LABEL[x]).join(", ")}>
              {s.name || skillName(s.slug)}
              <span className="sr-only"> — {SKILL_SOURCE_LABEL[src]}</span>
            </Chip>
          </li>
        );
      })}
      {rest > 0 && (
        <li>
          <Chip tone="neutral">+{rest}</Chip>
        </li>
      )}
    </ul>
  );
}

export function SkillLegend() {
  return (
    <p className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-ink-subtle">
      <span className="inline-flex items-center gap-1">
        <BadgeCheck className="size-3.5 text-success" aria-hidden /> подтверждён тестом
      </span>
      <span className="inline-flex items-center gap-1">
        <Trophy className="size-3.5 text-primary" aria-hidden /> подтверждён ФСП
      </span>
      <span>без значка — из резюме или заявлен</span>
    </p>
  );
}

function matchedAlias(o: SkillView, query: string): string | null {
  const q = query.trim().toLowerCase();
  if (!q || o.name.toLowerCase().includes(q)) return null;
  return o.aliases.find((a) => a.toLowerCase().startsWith(q)) ?? null;
}

interface SkillInputProps {
  value: string[];
  onChange: (next: string[]) => void;
  placeholder?: string;
  max?: number;
  tone?: "primary" | "neutral";
  ariaLabel?: string;
}

/** Combobox with chips: type to search the skill dictionary (synonyms like «постгрес» work), Enter to add. */
export function SkillInput({ value, onChange, placeholder = "Начните вводить: Go, PostgreSQL, React…", max = 30, tone = "neutral", ariaLabel }: SkillInputProps) {
  const field = useFieldControl({});
  const listId = useId();
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const search = useSkillSearch(query, open);
  const options = useMemo(() => (search.data ?? []).filter((s) => !value.includes(s.slug)).slice(0, 8), [search.data, value]);

  const add = (slug: string) => {
    if (!value.includes(slug) && value.length < max) onChange([...value, slug]);
    setQuery("");
    setActive(0);
    inputRef.current?.focus();
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setOpen(true);
      setActive((i) => Math.min(i + 1, Math.max(0, options.length - 1)));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      const opt = options[active];
      if (open && opt) {
        e.preventDefault();
        add(opt.slug);
      }
    } else if (e.key === "Escape") {
      setOpen(false);
    } else if (e.key === "Backspace" && !query && value.length) {
      onChange(value.slice(0, -1));
    }
  };

  return (
    <div className="relative">
      <div
        className={cn(controlBase, "flex min-h-10 flex-wrap items-center gap-1.5 px-2 py-1.5 focus-within:border-primary")}
        aria-invalid={field["aria-invalid"]}
        onClick={() => inputRef.current?.focus()}
      >
        {value.map((slug) => (
          <Chip key={slug} tone={tone === "primary" ? "primary" : "neutral"} onRemove={() => onChange(value.filter((v) => v !== slug))} removeLabel={`Убрать ${skillName(slug)}`}>
            {skillName(slug)}
          </Chip>
        ))}
        <input
          ref={inputRef}
          id={field.id}
          aria-describedby={field["aria-describedby"]}
          aria-label={ariaLabel}
          role="combobox"
          aria-expanded={open && options.length > 0}
          aria-controls={listId}
          aria-autocomplete="list"
          aria-activedescendant={open && options[active] ? `${listId}-${options[active].slug}` : undefined}
          value={query}
          placeholder={value.length ? "" : placeholder}
          onChange={(e) => {
            setQuery(e.target.value);
            setOpen(true);
            setActive(0);
          }}
          onFocus={() => setOpen(true)}
          onBlur={() => setTimeout(() => setOpen(false), 120)}
          onKeyDown={onKeyDown}
          className="h-7 min-w-32 flex-1 bg-transparent px-1 text-sm outline-none placeholder:text-ink-faint"
        />
      </div>
      {open && options.length > 0 && (
        <ul id={listId} role="listbox" className="absolute z-30 mt-1 max-h-64 w-full overflow-auto rounded-control border border-line bg-white py-1 shadow-pop">
          {options.map((o, i) => (
            <li
              key={o.slug}
              id={`${listId}-${o.slug}`}
              role="option"
              aria-selected={i === active}
              onMouseDown={(e) => {
                e.preventDefault();
                add(o.slug);
              }}
              onMouseEnter={() => setActive(i)}
              className={cn("flex cursor-pointer items-center justify-between gap-3 px-3 py-2 text-sm", i === active ? "bg-primary-50 text-primary-strong" : "text-ink")}
            >
              <span className="flex items-center gap-2">
                <Plus className="size-3.5 opacity-60" aria-hidden />
                {o.name}
              </span>
              {matchedAlias(o, query) && <span className="text-xs text-ink-subtle">по запросу «{matchedAlias(o, query)}»</span>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
