import { TriangleAlert, Video, VideoOff } from "lucide-react";
import type { ProctoringBadge as Badgeable } from "@/api/types";
import { Badge, type BadgeTone } from "@/components/ui";
import { pluralize } from "@/lib/format";

const STATUS: Record<Badgeable["status"], { tone: BadgeTone; text: string; video: boolean }> = {
  recorded: { tone: "success", text: "Тест с видеозаписью", video: true },
  recording: { tone: "success", text: "Тест с видеозаписью", video: true },
  declined: { tone: "muted", text: "Тест без видеозаписи", video: false },
  interrupted: { tone: "warning", text: "Запись прервалась", video: false },
};

/** Exactly what an employer reads; the candidate's result shows the same words. */
export function proctoringRemarks(b: Pick<Badgeable, "left_test" | "pastes" | "fullscreen_exits">): string | null {
  const parts = [
    b.left_test ? pluralize(b.left_test, ["уход со вкладки", "ухода со вкладки", "уходов со вкладки"]) : null,
    b.pastes ? pluralize(b.pastes, ["вставка текста", "вставки текста", "вставок текста"]) : null,
    b.fullscreen_exits ? pluralize(b.fullscreen_exits, ["выход из полноэкранного режима", "выхода из полноэкранного режима", "выходов из полноэкранного режима"]) : null,
  ].filter(Boolean);
  return parts.length ? `Есть замечания: ${parts.join(", ")}` : null;
}

export function proctoringStatusText(status: Badgeable["status"]): string {
  return STATUS[status].text;
}

/** The video itself never reaches the employer: only this mark of how the test was taken. */
export function ProctoringBadge({ badge }: { badge: Badgeable | null | undefined }) {
  if (!badge) return null;
  const s = STATUS[badge.status];
  const Icon = s.video ? Video : VideoOff;
  const remarks = proctoringRemarks(badge);
  return (
    <>
      <Badge tone={s.tone} icon={<Icon className="size-3" aria-hidden />} title="Как проходил тест, подтвердивший категорию">
        {s.text}
      </Badge>
      {remarks && (
        <span title="Отметки браузера во время теста" className="inline-flex max-w-full items-start gap-1.5 rounded-md bg-warning-50 px-2 py-1 text-xs leading-4 font-medium text-warning">
          <TriangleAlert className="mt-px size-3 shrink-0" aria-hidden />
          {remarks}
        </span>
      )}
    </>
  );
}
