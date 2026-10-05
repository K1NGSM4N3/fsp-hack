"use client";

import { Pause, Pin, UserRound } from "lucide-react";
import { useRef } from "react";
import { Flip, MOTION_OK, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

type ChipId = "fsp" | "kafka" | "remote" | "salary";

const CHIPS: { id: ChipId; label: string }[] = [
  { id: "fsp", label: "С достижениями ФСП" },
  { id: "kafka", label: "Kafka обязательна" },
  { id: "remote", label: "Удалённо" },
  { id: "salary", label: "Ожидания в вилке" },
];

interface Row {
  code: string;
  base: number;
  note: string;
  fsp: boolean;
  kafka: "test" | "resume" | "none";
  remote: boolean;
  inRange: boolean;
  pinned?: boolean;
}

const ROWS: Row[] = [
  { code: "K-5F8A1F", base: 88, note: "Все обязательные навыки, два подтверждены тестом", fsp: true, kafka: "test", remote: true, inRange: true, pinned: true },
  { code: "K-C42234", base: 76, note: "Грейд Senior, на ступень выше запроса", fsp: true, kafka: "resume", remote: true, inRange: false },
  { code: "K-71B0E2", base: 72, note: "Результат теста в верхней части грейда", fsp: false, kafka: "none", remote: true, inRange: true },
  { code: "K-9073DC", base: 68, note: "Вместо PostgreSQL близкий навык MySQL", fsp: false, kafka: "test", remote: false, inRange: true },
  { code: "K-2D55A9", base: 64, note: "Навык Docker только заявлен", fsp: true, kafka: "test", remote: false, inRange: true },
];

const SEQUENCE: ChipId[][] = [[], ["kafka"], ["kafka", "fsp"], ["fsp"], ["fsp", "remote"], ["remote"], ["remote", "salary"], ["salary"]];

function evaluate(row: Row, active: ChipId[]) {
  let score = row.base;
  let fits = true;
  if (active.includes("kafka")) score += row.kafka === "test" ? 5 : row.kafka === "resume" ? 1 : -9;
  if (active.includes("salary") && !row.inRange) score -= 6;
  if (active.includes("fsp") && !row.fsp) fits = false;
  if (active.includes("remote") && !row.remote) fits = false;
  return { score: Math.max(0, Math.min(99, score)), fits };
}

function ordered(active: ChipId[]) {
  return ROWS.map((r) => ({ row: r, ...evaluate(r, active) })).sort((a, b) => Number(b.row.pinned ?? false) - Number(a.row.pinned ?? false) || Number(b.fits) - Number(a.fits) || b.score - a.score);
}

const INITIAL = ordered(SEQUENCE[0] ?? []);

/** Filter chips toggle on their own and the shortlist re-sorts with Flip; hovering pauses the loop. */
export function LiveShortlist() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const el = root.current;
        const list = el?.querySelector<HTMLElement>("[data-list]");
        if (!el || !list) return;
        let step = 0;
        let paused = false;
        let visible = true;
        const apply = () => {
          step = (step + 1) % SEQUENCE.length;
          const active = SEQUENCE[step] ?? [];
          el.querySelectorAll<HTMLElement>("[data-chip]").forEach((chip) => chip.toggleAttribute("data-on", active.includes(chip.dataset.chip as ChipId)));
          const rows = gsap.utils.toArray<HTMLElement>("[data-row]", list);
          const state = Flip.getState(rows);
          for (const item of ordered(active)) {
            const node = list.querySelector<HTMLElement>(`[data-row="${item.row.code}"]`);
            if (!node) continue;
            list.appendChild(node);
            node.toggleAttribute("data-out", !item.fits);
            const score = node.querySelector<HTMLElement>("[data-score]");
            if (score && score.textContent !== String(item.score)) gsap.to(score, { duration: 0.6, scrambleText: { text: String(item.score), chars: "0123456789", speed: 0.6 } });
          }
          Flip.from(state, { duration: 0.75, ease: "power3.inOut", stagger: 0.03 });
        };
        const loop = gsap.delayedCall(2.6, function tick() {
          if (!paused && visible && !document.hidden) apply();
          loop.restart(true);
        });
        const enter = () => {
          paused = true;
          el.setAttribute("data-paused", "");
        };
        const leave = () => {
          paused = false;
          el.removeAttribute("data-paused");
        };
        el.addEventListener("pointerenter", enter);
        el.addEventListener("pointerleave", leave);
        ScrollTrigger.create({ trigger: el, start: "top bottom", end: "bottom top", onToggle: (self) => (visible = self.isActive) });
        return () => {
          loop.kill();
          el.removeEventListener("pointerenter", enter);
          el.removeEventListener("pointerleave", leave);
        };
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="group/list rounded-[26px] border border-line bg-white p-5 shadow-lift sm:p-6">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-[11px] tracking-[0.12em] text-ink-subtle uppercase">Подборка · Middle Go, платежи</p>
          <p className="mt-1 text-sm text-ink-muted">Фильтры переключаются сами, наведите курсор, чтобы остановить</p>
        </div>
        <span className="hidden shrink-0 items-center gap-1 rounded-full bg-canvas px-2 py-1 text-[11px] text-ink-subtle group-data-[paused]/list:inline-flex">
          <Pause className="size-3" aria-hidden /> пауза
        </span>
      </div>
      <div className="mt-4 flex flex-wrap gap-1.5">
        {CHIPS.map((c) => (
          <span
            key={c.id}
            data-chip={c.id}
            className="rounded-full border border-line-strong px-3 py-1 text-xs font-medium text-ink-muted transition-colors duration-300 data-[on]:border-primary data-[on]:bg-primary data-[on]:text-white"
          >
            {c.label}
          </span>
        ))}
      </div>
      <ul data-list className="mt-4 space-y-2">
        {INITIAL.map(({ row, score, fits }) => (
          <li
            key={row.code}
            data-row={row.code}
            data-out={fits ? undefined : ""}
            className="flex items-center gap-3 rounded-2xl border border-line bg-white px-3 py-2.5 transition-opacity duration-500 data-[out]:opacity-35"
          >
            <span className="flex size-9 shrink-0 items-center justify-center rounded-full bg-canvas text-ink-subtle">
              <UserRound className="size-4" aria-hidden />
            </span>
            <div className="min-w-0 flex-1">
              <p className="flex items-center gap-2 font-mono text-sm font-semibold text-ink">
                {row.code}
                {row.pinned && <Pin className="size-3.5 text-primary" aria-label="закреплён" />}
              </p>
              <p className="truncate text-xs text-ink-muted">{row.note}</p>
            </div>
            <span className={cn("tabular w-9 shrink-0 rounded-lg bg-primary-50 py-1 text-center font-mono text-sm font-semibold text-primary-strong")}>
              <span data-score>{score}</span>
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
