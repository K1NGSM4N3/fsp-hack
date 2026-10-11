"use client";

import { useRef, type CSSProperties, type ReactNode } from "react";
import { useMotion } from "@/components/motion/lazy";
import { cn } from "@/lib/cn";

interface MarqueeProps {
  items: ReactNode[];
  reverse?: boolean;
  duration?: number;
  className?: string;
  label?: string;
}

/**
 * Endless row. Desktop: a GSAP loop that speeds up with scroll velocity and sleeps off screen.
 * Phones: a plain CSS keyframe loop that runs on the compositor and costs no main-thread work while scrolling.
 * With reduced motion it is a plain clipped row.
 */
export function Marquee({ items, reverse = false, duration = 45, className, label }: MarqueeProps) {
  const ref = useRef<HTMLDivElement>(null);
  useMotion(
    ({ MOTION_DESKTOP, MOTION_MOBILE, ScrollTrigger, gsap }) => {
      const track = ref.current?.querySelector<HTMLElement>("[data-track]");
      if (!track) return;
      const mm = gsap.matchMedia();
      mm.add(MOTION_DESKTOP, () => {
        const tween = gsap.fromTo(track, { xPercent: reverse ? -50 : 0 }, { xPercent: reverse ? 0 : -50, duration, ease: "none", repeat: -1, paused: true });
        const boost = gsap.quickTo(tween, "timeScale", { duration: 0.6, ease: "power2.out" });
        let settle: gsap.core.Tween | null = null;
        ScrollTrigger.create({
          trigger: ref.current,
          start: "top bottom",
          end: "bottom top",
          onToggle: (self) => void (self.isActive ? tween.play() : tween.pause()),
          onUpdate: (self) => {
            boost(1 + Math.min(Math.abs(self.getVelocity()) / 500, 4));
            settle?.kill();
            settle = gsap.delayedCall(0.2, () => boost(1));
          },
        });
      });
      mm.add(MOTION_MOBILE, () => {
        track.dataset.css = "";
        return () => delete track.dataset.css;
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
      <div
        data-track
        style={{ "--marquee-duration": `${duration}s` } as CSSProperties}
        className={cn(
          "flex w-max data-css:animate-[marquee_var(--marquee-duration)_linear_infinite] data-css:will-change-transform",
          reverse && "data-css:[animation-direction:reverse]",
        )}
      >
        {row(false)}
        {row(true)}
      </div>
    </div>
  );
}
