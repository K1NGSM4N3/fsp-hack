import { Award, Medal, Trophy } from "lucide-react";
import type { ReactNode } from "react";
import type { FspAchievement } from "@/api/types";
import { cn } from "@/lib/cn";
import { formatDate } from "@/lib/format";
import { FSP_DISCIPLINE_LABEL, FSP_LEVEL_LABEL, fspAchievementTitle, isPrize } from "@/lib/labels";

function sortAchievements(list: FspAchievement[]): FspAchievement[] {
  return [...list].sort((a, b) => Number(isPrize(b)) - Number(isPrize(a)) || b.date.localeCompare(a.date));
}

const titleOf = (a: FspAchievement) => a.text || fspAchievementTitle(a);

export function FspAchievementList({ achievements, limit, className }: { achievements: FspAchievement[]; limit?: number; className?: string }) {
  const sorted = sortAchievements(achievements);
  const shown = limit ? sorted.slice(0, limit) : sorted;
  return (
    <ul className={cn("space-y-2.5", className)}>
      {shown.map((a, i) => {
        const prize = isPrize(a);
        const Icon = a.kind === "rank" ? Medal : prize ? Trophy : Award;
        return (
          <li key={`${a.date}-${i}`} className="flex gap-3">
            <span className={cn("mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-full", prize ? "bg-primary-50 text-primary" : "bg-canvas text-ink-subtle")}>
              <Icon className="size-4" aria-hidden />
            </span>
            <div className="min-w-0 text-sm">
              <p className="font-medium text-ink">{titleOf(a)}</p>
              <p className="mt-0.5 text-xs text-ink-subtle">
                {[FSP_DISCIPLINE_LABEL[a.discipline], a.event_level ? FSP_LEVEL_LABEL[a.event_level] : null, a.event_name && !titleOf(a).includes(a.event_name) ? a.event_name : null, a.team ? `команда «${a.team}»` : null, formatDate(a.date)]
                  .filter(Boolean)
                  .join(" · ")}
              </p>
              {a.protocol_url && (
                <a href={a.protocol_url} target="_blank" rel="noreferrer" className="link-inline tap mt-0.5 inline-block text-xs">
                  Протокол
                </a>
              )}
            </div>
          </li>
        );
      })}
      {limit && sorted.length > limit && <li className="pl-11 text-xs text-ink-subtle">и ещё {sorted.length - limit}</li>}
    </ul>
  );
}

/** Neutral state: absence of FSP history must never read as a negative signal (F-3). */
export function FspEmpty({ action, text = "Нет данных ФСП" }: { action?: ReactNode; text?: string }) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-3 rounded-control border border-dashed border-line-strong px-4 py-3">
      <div className="flex items-center gap-3 text-sm text-ink-muted">
        <Trophy className="size-4 text-ink-faint" aria-hidden />
        {text}
      </div>
      {action}
    </div>
  );
}
