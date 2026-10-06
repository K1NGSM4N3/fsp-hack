import type { ReviewAnswer } from "@/api/admin";
import { GRADE_LABEL } from "@/lib/labels";
import type { Grade } from "@/api/types";
import { lz } from "@/features/admin/labels";

const W = 560;
const H = 150;
const LEFT = 78;
const RIGHT = 16;
const ROWS = { correct: 52, wrong: 100 };

interface Props {
  answers: ReviewAnswer[];
  theta: number | null | undefined;
  cut: number;
  grade: Grade;
}

/**
 * Two-row dot strip: every answered item at its IRT difficulty, right answers on top, wrong ones
 * below. An honest pattern leans right answers to the left of θ̂ and wrong ones to the right; the
 * person-fit flag fires when it is the other way round.
 */
export function DifficultyPattern({ answers, theta, cut, grade }: Props) {
  const placed = answers.filter((a): a is ReviewAnswer & { difficulty: number } => typeof a.difficulty === "number");
  const values = [...placed.map((a) => a.difficulty), theta ?? 0, cut];
  const pad = (v: number) => Math.round(v * 2) / 2;
  let lo = pad(Math.min(...values) - 0.5);
  let hi = pad(Math.max(...values) + 0.5);
  if (hi - lo < 3) {
    const mid = (hi + lo) / 2;
    lo = pad(mid - 1.5);
    hi = lo + 3;
  }
  const x = (v: number) => LEFT + ((v - lo) / (hi - lo)) * (W - LEFT - RIGHT);
  const ticks: number[] = [];
  for (let t = Math.ceil(lo * 2) / 2; t <= hi + 1e-9; t += 0.5) ticks.push(t);

  const rowY = (a: ReviewAnswer & { difficulty: number }, i: number, row: typeof placed) => {
    const base = a.correct ? ROWS.correct : ROWS.wrong;
    const near = row.slice(0, i).filter((b) => b.correct === a.correct && Math.abs(x(b.difficulty) - x(a.difficulty)) < 12).length;
    return base + (near % 2 === 0 ? -1 : 1) * Math.min(near, 2) * 7;
  };

  return (
    <figure>
      <div className="mb-2 flex flex-wrap items-center gap-x-5 gap-y-1 text-xs text-ink-muted">
        <span className="inline-flex items-center gap-1.5">
          <svg width="12" height="12" aria-hidden>
            <circle cx="6" cy="6" r="5" className="fill-success" />
          </svg>
          верный ответ
        </span>
        <span className="inline-flex items-center gap-1.5">
          <svg width="12" height="12" aria-hidden>
            <circle cx="6" cy="6" r="4.5" className="fill-white stroke-accent-strong" strokeWidth={2} />
          </svg>
          неверный или нет ответа
        </span>
        <span className="inline-flex items-center gap-1.5">
          <span className="h-3 w-0.5 rounded bg-primary" aria-hidden />
          θ̂ — оценка уровня
        </span>
        <span className="inline-flex items-center gap-1.5">
          <span className="h-3 w-0 border-l-2 border-dashed border-ink-faint" aria-hidden />
          порог {GRADE_LABEL[grade]}
        </span>
      </div>
      <div className="scrollbar-thin -mx-1 overflow-x-auto px-1">
      <svg viewBox={`0 0 ${W} ${H}`} className="h-auto w-full min-w-[480px]" role="img" aria-label="Верные и неверные ответы по сложности заданий; таблица ниже повторяет эти данные">
        {ticks.map((t) => (
          <g key={t}>
            <line x1={x(t)} x2={x(t)} y1={24} y2={H - 26} className="stroke-line" strokeWidth={1} />
            <text x={x(t)} y={H - 10} textAnchor="middle" className="fill-ink-subtle text-[12px]">
              {lz(t, 1)}
            </text>
          </g>
        ))}
        <text x={0} y={ROWS.correct + 4} className="fill-ink-muted text-[12px]">
          Верно
        </text>
        <text x={0} y={ROWS.wrong + 4} className="fill-ink-muted text-[12px]">
          Неверно
        </text>
        <line x1={x(cut)} x2={x(cut)} y1={20} y2={H - 26} className="stroke-ink-faint" strokeWidth={2} strokeDasharray="4 4" />
        {typeof theta === "number" && (
          <g>
            <line x1={x(theta)} x2={x(theta)} y1={16} y2={H - 26} className="stroke-primary" strokeWidth={2} />
            <text x={x(theta)} y={11} textAnchor="middle" className="fill-primary text-[12px] font-medium">
              θ̂ {lz(theta)}
            </text>
          </g>
        )}
        {placed.map((a, i) => {
          const cy = rowY(a, i, placed);
          const label = `Задание ${a.seq + 1}: ${a.title} · сложность ${lz(a.difficulty)} · ${a.correct ? "верно" : a.timed_out ? "время вышло" : "неверно"}`;
          return a.correct ? (
            <circle key={a.seq} cx={x(a.difficulty)} cy={cy} r={6} className="fill-success stroke-white" strokeWidth={2}>
              <title>{label}</title>
            </circle>
          ) : (
            <circle key={a.seq} cx={x(a.difficulty)} cy={cy} r={5} className="fill-white stroke-accent-strong" strokeWidth={2}>
              <title>{label}</title>
            </circle>
          );
        })}
      </svg>
      </div>
      <figcaption className="mt-1 text-xs text-ink-subtle">Сложность задания — параметр b на шкале θ специализации. Наведите на точку, чтобы увидеть задание.</figcaption>
    </figure>
  );
}
