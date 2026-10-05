"use client";

import { useRef } from "react";
import { MOTION_OK, formatCounter, gsap, useGSAP } from "@/components/motion/gsap";

/** Tweens from the previous value; the span is keyed by value so React never writes into a node GSAP has rewritten. */
export function AnimatedNumber({ value, decimals = 0, duration = 1.1 }: { value: number; decimals?: number; duration?: number }) {
  const ref = useRef<HTMLSpanElement>(null);
  const prev = useRef(0);
  useGSAP(
    () => {
      const el = ref.current;
      const from = prev.current;
      if (!el || from === value || !window.matchMedia(MOTION_OK).matches) {
        prev.current = value;
        return;
      }
      const state = { v: from };
      el.textContent = formatCounter(from, decimals);
      gsap.to(state, {
        v: value,
        duration,
        ease: "power3.out",
        onUpdate: () => {
          el.textContent = formatCounter(state.v, decimals);
        },
      });
      return () => {
        el.textContent = formatCounter(value, decimals);
        prev.current = value;
      };
    },
    { dependencies: [value, decimals, duration] },
  );
  return (
    <span key={value} ref={ref} className="tabular">
      {formatCounter(value, decimals)}
    </span>
  );
}
