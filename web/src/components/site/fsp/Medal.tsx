import type { ComponentProps } from "react";
import { cn } from "@/lib/cn";

export type Metal = "gold" | "silver" | "bronze";

const FACE: Record<Metal, string> = {
  gold: "from-warning-50 via-warning-200 to-warning-200 text-warning",
  silver: "from-white via-surface to-line-strong text-ink-muted",
  bronze: "from-accent-50 via-accent-100 to-accent-300 text-accent-strong",
};

const SIZE = {
  sm: { ribbon: "h-4 w-3", disc: "size-9 text-[11px] ring-2", inner: "inset-[3px]" },
  md: { ribbon: "h-6 w-4", disc: "size-12 text-sm ring-[3px]", inner: "inset-1" },
  lg: { ribbon: "h-9 w-6", disc: "size-16 text-base ring-4", inner: "inset-1.5" },
  xl: { ribbon: "h-12 w-8", disc: "size-20 text-lg ring-4 sm:size-24 sm:text-xl", inner: "inset-2" },
} as const;

interface MedalProps extends Omit<ComponentProps<"span">, "children"> {
  mark: string;
  metal: Metal;
  size?: keyof typeof SIZE;
  ribbon?: boolean;
}

export function Medal({ mark, metal, size = "md", ribbon = true, className, ...props }: MedalProps) {
  const s = SIZE[size];
  return (
    <span aria-hidden className={cn("relative inline-flex shrink-0 flex-col items-center", className)} {...props}>
      {ribbon && (
        <span
          className={cn(
            s.ribbon,
            "bg-[linear-gradient(90deg,var(--color-primary)_0_34%,var(--color-white)_34%_66%,var(--color-accent)_66%)] [clip-path:polygon(0_0,100%_0,100%_100%,50%_72%,0_100%)]",
          )}
        />
      )}
      <span
        className={cn(
          "relative flex items-center justify-center rounded-full bg-linear-to-br font-mono font-bold tracking-tight shadow-pop ring-white",
          ribbon && "-mt-1",
          s.disc,
          FACE[metal],
        )}
      >
        <span className={cn("absolute rounded-full border border-white/70", s.inner)} />
        <span className="relative">{mark}</span>
      </span>
    </span>
  );
}
