import type { ReactNode } from "react";

export type DisciplineKey = "algo" | "product" | "security" | "ai" | "robotics" | "uav";

const ACCENT = "text-accent-300";

const SHAPES: Record<DisciplineKey, ReactNode> = {
  algo: (
    <>
      <path d="M54.5 32.3 L37.5 57.7 M65.5 32.3 L82.5 57.7 M28.5 75.4 L20.8 96.5 M35.5 75.4 L43.2 96.5 M88 76 V96" />
      <circle cx="60" cy="24" r="10" />
      <circle cx="32" cy="66" r="10" />
      <circle cx="88" cy="66" r="10" />
      <circle cx="18" cy="104" r="8" />
      <circle cx="46" cy="104" r="8" className={ACCENT} />
      <circle cx="88" cy="104" r="8" />
    </>
  ),
  product: (
    <>
      <rect x="12" y="18" width="96" height="72" rx="10" />
      <path d="M12 36 H108" />
      <path d="M24 48 H54 V78 H24 Z" />
      <path d="M66 50 H96 M66 62 H90 M66 74 H80" />
      <path d="M78 80 L78 110 L86 102 L92 114 L98 111 L92 99 L103 99 Z" className={ACCENT} />
    </>
  ),
  security: (
    <>
      <path d="M60 10 L100 24 V56 C100 84 82 100 60 110 C38 100 20 84 20 56 V24 Z" />
      <path d="M60 22 L88 32 V56 C88 76 76 88 60 96" />
      <circle cx="60" cy="54" r="10" className={ACCENT} />
      <path d="M60 64 V80" className={ACCENT} />
    </>
  ),
  ai: (
    <>
      <rect x="30" y="30" width="60" height="60" rx="12" />
      <path d="M45 30 V14 M60 30 V14 M75 30 V14 M45 90 V106 M60 90 V106 M75 90 V106 M30 45 H14 M30 60 H14 M30 75 H14 M90 45 H106 M90 60 H106 M90 75 H106" />
      <path d="M52 49 H68 M48 55 L56 67 M72 55 L64 67" />
      <circle cx="46" cy="49" r="5" />
      <circle cx="74" cy="49" r="5" />
      <circle cx="60" cy="72" r="5" className={ACCENT} />
    </>
  ),
  robotics: (
    <>
      <path d="M26 108 H78 M34 108 V98 H70 V108" />
      <circle cx="52" cy="88" r="8" />
      <path d="M49 80 L39 54" />
      <circle cx="36" cy="46" r="8" />
      <path d="M44 43 L76 30" />
      <circle cx="83" cy="27" r="7" />
      <path d="M88 32 Q100 38 98 52 M89 21 Q102 16 110 28" className={ACCENT} />
    </>
  ),
  uav: (
    <>
      <rect x="46" y="46" width="28" height="28" rx="7" />
      <path d="M48 48 L34 34 M72 48 L86 34 M48 72 L34 86 M72 72 L86 86" />
      <circle cx="27" cy="27" r="15" />
      <circle cx="93" cy="27" r="15" />
      <circle cx="27" cy="93" r="15" />
      <circle cx="93" cy="93" r="15" />
      <path d="M60 46 V30 M54 36 L60 30 L66 36" className={ACCENT} />
    </>
  ),
};

export function Emblem({ kind, className }: { kind: DisciplineKey; className?: string }) {
  return (
    <svg
      data-emblem
      viewBox="0 0 120 120"
      fill="none"
      stroke="currentColor"
      strokeWidth="5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden
      className={className}
    >
      {SHAPES[kind]}
    </svg>
  );
}
