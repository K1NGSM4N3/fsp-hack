import { Building2, MapPin } from "lucide-react";
import Link from "next/link";
import type { ReactNode } from "react";
import type { PublicVacancy } from "@/api/types";
import { Chip } from "@/components/ui";
import { formatRelative } from "@/lib/format";
import { WORK_FORMAT_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";
import { CategoryBadge, SalaryRange } from "./Category";

export function VacancyCard({ vacancy: v, href, footer }: { vacancy: PublicVacancy; href: string; footer?: ReactNode }) {
  return (
    <article className="rounded-card border border-line bg-white p-5 shadow-card transition-colors hover:border-primary-200">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0">
          <h3 className="font-mono text-base font-semibold tracking-tight">
            <Link href={href} className="rounded text-ink hover:text-primary hover:underline">
              {v.title}
            </Link>
          </h3>
          <p className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-ink-muted">
            <span className="inline-flex items-center gap-1.5">
              <Building2 className="size-4 text-ink-subtle" aria-hidden />
              {v.company_name}
            </span>
            <span className="inline-flex items-center gap-1.5">
              <MapPin className="size-4 text-ink-subtle" aria-hidden />
              {WORK_FORMAT_LABEL[v.work_format]}
              {v.city && v.work_format !== "remote" ? `, ${v.city}` : ""}
            </span>
          </p>
        </div>
        <div className="shrink-0 sm:text-right">
          <SalaryRange salary={v.salary} label="Зарплата" />
          <p className="mt-0.5 text-xs text-ink-subtle">{v.published_at ? `опубликована ${formatRelative(v.published_at)}` : "черновик"}</p>
        </div>
      </div>
      <div className="mt-4 flex flex-wrap items-center gap-1.5">
        <CategoryBadge category={{ specialization: v.specialization, grade: v.grade }} size="sm" />
        {v.skills.slice(0, 6).map((s) => (
          <Chip key={s}>{skillName(s)}</Chip>
        ))}
      </div>
      {footer && <div className="mt-4 flex flex-wrap items-center gap-2 border-t border-line pt-4">{footer}</div>}
    </article>
  );
}
