"use client";

import { useRef, type ReactNode } from "react";
import { MOTION_OK, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

interface MarqueeProps {
  items: ReactNode[];
  reverse?: boolean;
  duration?: number;
  className?: string;
  label?: string;
}

/** Endless row that speeds up with scroll velocity; with reduced motion it is a plain clipped row. */
export function Marquee({ items, reverse = false, duration = 45, className, label }: MarqueeProps) {
  const ref = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const track = ref.current?.querySelector("[data-track]");
        if (!track) return;
        const tween = gsap.fromTo(track, { xPercent: reverse ? -50 : 0 }, { xPercent: reverse ? 0 : -50, duration, ease: "none", repeat: -1 });
        const boost = gsap.quickTo(tween, "timeScale", { duration: 0.6, ease: "power2.out" });
        let settle: gsap.core.Tween | null = null;
        ScrollTrigger.create({
          trigger: ref.current,
          start: "top bottom",
          end: "bottom top",
          onUpdate: (self) => {
            boost(1 + Math.min(Math.abs(self.getVelocity()) / 500, 4));
            settle?.kill();
            settle = gsap.delayedCall(0.2, () => boost(1));
          },
        });
      });
    },
    { scope: ref },
  );
  const row = (hidden: boolean) => (
    <ul aria-hidden={hidden || undefined} className="flex shrink-0 items-center">
      {items.map((item, i) => (
        <li key={i} className="flex shrink-0 items-center">
          {item}
        </li>
      ))}
    </ul>
  );
  return (
    <div ref={ref} className={cn("mask-fade-x overflow-hidden", className)} aria-label={label} role={label ? "region" : undefined}>
      <div data-track className="flex w-max">
        {row(false)}
        {row(true)}
      </div>
    </div>
  );
}
