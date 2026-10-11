import type { ReactNode } from "react";
import { cn } from "@/lib/cn";
import { CodeBlock } from "./CodeBlock";

function inline(text: string, keyBase: string): ReactNode[] {
  const out: ReactNode[] = [];
  const re = /(`[^`]+`)|(\*\*[^*]+\*\*)/g;
  let last = 0;
  let m: RegExpExecArray | null;
  let i = 0;
  while ((m = re.exec(text))) {
    if (m.index > last) out.push(text.slice(last, m.index));
    const token = m[0];
    if (token.startsWith("`")) {
      out.push(
        <code key={`${keyBase}-${i++}`} className="rounded bg-canvas px-1 py-0.5 font-mono text-[0.9em] text-ink">
          {token.slice(1, -1)}
        </code>,
      );
    } else {
      out.push(<strong key={`${keyBase}-${i++}`}>{token.slice(2, -2)}</strong>);
    }
    last = m.index + token.length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}

function isTable(lines: string[]): boolean {
  return lines.length >= 2 && lines.every((l) => /^\s*\|.*\|\s*$/.test(l)) && /^\s*\|[\s:|-]+\|\s*$/.test(lines[1] ?? "");
}

function cells(line: string): string[] {
  return line.trim().replace(/^\|/, "").replace(/\|$/, "").split("|").map((c) => c.trim());
}

function Table({ lines, keyBase }: { lines: string[]; keyBase: string }) {
  const head = cells(lines[0] ?? "");
  const rows = lines.slice(2).map(cells);
  return (
    <div className="scrollbar-thin max-w-full overflow-x-auto rounded-control border border-line">
      <table className="min-w-full border-collapse font-mono text-[13px]">
        <thead className="bg-canvas">
          <tr>
            {head.map((h, i) => (
              <th key={i} scope="col" className="border-b border-line px-3 py-1.5 text-left font-semibold text-ink">
                {inline(h, `${keyBase}-h${i}`)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((r, ri) => (
            <tr key={ri} className="odd:bg-white even:bg-canvas/50">
              {r.map((c, ci) => (
                <td key={ci} className="border-b border-line px-3 py-1 text-ink last:border-b-0">
                  {inline(c, `${keyBase}-${ri}-${ci}`)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** Subset used by test prompts and tasks: paragraphs, lists, tables, inline code, bold, fenced code. */
export function Markdown({ source, className }: { source: string; className?: string }) {
  const blocks: ReactNode[] = [];
  const parts = source.split(/```(\w*)\n([\s\S]*?)```/g);
  for (let i = 0; i < parts.length; i += 3) {
    const text = parts[i] ?? "";
    for (const [j, para] of text.split(/\n{2,}/).entries()) {
      const trimmed = para.trim();
      if (!trimmed) continue;
      const lines = trimmed.split("\n");
      const tableAt = lines.findIndex((l) => /^\s*\|/.test(l));
      if (tableAt > 0 && isTable(lines.slice(tableAt))) {
        blocks.push(
          <p key={`pt-${i}-${j}`} className="whitespace-pre-line">
            {inline(lines.slice(0, tableAt).join("\n"), `pt-${i}-${j}`)}
          </p>,
        );
        blocks.push(<Table key={`t-${i}-${j}`} lines={lines.slice(tableAt)} keyBase={`t-${i}-${j}`} />);
      } else if (isTable(lines)) {
        blocks.push(<Table key={`t-${i}-${j}`} lines={lines} keyBase={`t-${i}-${j}`} />);
      } else if (lines.every((l) => /^\s*[-*]\s+/.test(l))) {
        blocks.push(
          <ul key={`ul-${i}-${j}`} className="list-disc space-y-1 pl-5">
            {lines.map((l, k) => (
              <li key={k}>{inline(l.replace(/^\s*[-*]\s+/, ""), `li-${i}-${j}-${k}`)}</li>
            ))}
          </ul>,
        );
      } else {
        blocks.push(
          <p key={`p-${i}-${j}`} className="whitespace-pre-line">
            {inline(trimmed, `p-${i}-${j}`)}
          </p>,
        );
      }
    }
    const lang = parts[i + 1];
    const code = parts[i + 2];
    if (code !== undefined) blocks.push(<CodeBlock key={`code-${i}`} code={code} language={lang || null} />);
  }
  return <div className={cn("space-y-3 text-sm leading-relaxed text-ink sm:text-base", className)}>{blocks}</div>;
}
