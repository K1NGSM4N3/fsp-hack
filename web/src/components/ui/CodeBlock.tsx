import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

const KEYWORDS: Record<string, string[]> = {
  javascript: ["const", "let", "var", "function", "return", "if", "else", "for", "while", "new", "async", "await", "class", "import", "from", "export", "of", "in", "true", "false", "null", "undefined", "this", "typeof"],
  python: ["def", "return", "if", "elif", "else", "for", "while", "in", "and", "or", "not", "import", "from", "as", "class", "True", "False", "None", "print", "range", "lambda", "with", "pass"],
  sql: ["SELECT", "FROM", "WHERE", "GROUP", "BY", "HAVING", "ORDER", "JOIN", "LEFT", "INNER", "ON", "AS", "COUNT", "SUM", "AVG", "AND", "OR", "NOT", "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE", "CREATE", "INDEX", "TABLE", "LIMIT", "DESC", "ASC", "DISTINCT"],
  go: ["func", "var", "for", "range", "go", "return", "if", "else", "package", "import", "type", "struct", "chan", "defer", "select", "case", "map", "make", "nil", "true", "false"],
  bash: ["if", "then", "fi", "for", "do", "done", "echo", "export", "sudo"],
};

const ALIASES: Record<string, string> = { js: "javascript", ts: "javascript", typescript: "javascript", jsx: "javascript", tsx: "javascript", py: "python", golang: "go", sh: "bash", shell: "bash", postgresql: "sql" };

function commentPrefix(lang: string): RegExp {
  if (lang === "python" || lang === "bash") return /#.*/;
  if (lang === "sql") return /--.*/;
  return /\/\/.*/;
}

/** Deliberately tiny tokenizer: enough contrast for short test snippets, no dependency. */
function highlightLine(line: string, lang: string): ReactNode[] {
  const keywords = new Set(KEYWORDS[lang] ?? []);
  const caseInsensitive = lang === "sql";
  const pattern = new RegExp(`(${commentPrefix(lang).source})|("(?:[^"\\\\]|\\\\.)*"|'(?:[^'\\\\]|\\\\.)*'|\`(?:[^\`\\\\]|\\\\.)*\`)|(\\b\\d+(?:\\.\\d+)?\\b)|([A-Za-z_][A-Za-z0-9_]*)`, "g");
  const out: ReactNode[] = [];
  let last = 0;
  let m: RegExpExecArray | null;
  let i = 0;
  while ((m = pattern.exec(line))) {
    if (m.index > last) out.push(line.slice(last, m.index));
    const [text, comment, str, num, word] = m;
    if (comment) out.push(<span key={i++} className="text-ink-subtle italic">{text}</span>);
    else if (str) out.push(<span key={i++} className="text-success">{text}</span>);
    else if (num) out.push(<span key={i++} className="text-accent-strong">{text}</span>);
    else if (word && keywords.has(caseInsensitive ? word.toUpperCase() : word)) out.push(<span key={i++} className="font-semibold text-primary-strong">{text}</span>);
    else out.push(text);
    last = m.index + text.length;
    if (comment) break;
  }
  if (last < line.length) out.push(line.slice(last));
  return out;
}

interface CodeBlockProps {
  code: string;
  language?: string | null;
  className?: string;
  showLanguage?: boolean;
}

export function CodeBlock({ code, language, className, showLanguage = true }: CodeBlockProps) {
  const lang = ALIASES[language?.toLowerCase() ?? ""] ?? language?.toLowerCase() ?? "text";
  const lines = code.replace(/\n$/, "").split("\n");
  return (
    <div className={cn("relative overflow-hidden rounded-control border border-line bg-canvas", className)}>
      {showLanguage && language && <span className="absolute top-2 right-3 font-mono text-[11px] tracking-wide text-ink-subtle uppercase">{language}</span>}
      <pre className="scrollbar-thin overflow-x-auto p-4 font-mono text-[13px] leading-6 text-ink sm:text-sm">
        <code>
          {lines.map((line, n) => (
            <span key={n} className="block min-h-6">
              {highlightLine(line, lang)}
            </span>
          ))}
        </code>
      </pre>
    </div>
  );
}
