"use client";

import { useLayoutEffect, useRef } from "react";
import { MOTION_OK, formatCounter, matches } from "@/components/motion/config";

const easeOut = (t: number) => 1 - (1 - t) ** 3;

/**
 * Tweens from the previous value with a plain rAF loop (no GSAP in every page that shows a stat). The span is keyed by
 * value so React never writes into a node the loop has rewritten.
 */
export function AnimatedNumber({ value, decimals = 0, duration = 1.1 }: { value: number; decimals?: number; duration?: number }) {
  const ref = useRef<HTMLSpanElement>(null);
  const prev = useRef(0);
  useLayoutEffect(() => {
    const el = ref.current;
    const from = prev.current;
    if (!el || from === value || !matches(MOTION_OK)) {
      prev.current = value;
      return;
    }
    el.textContent = formatCounter(from, decimals);
    const start = performance.now();
    let frame = 0;
    const step = (now: number) => {
      const t = Math.min(1, (now - start) / (duration * 1000));
      el.textContent = formatCounter(from + (value - from) * easeOut(t), decimals);
      if (t < 1) frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
    return () => {
      cancelAnimationFrame(frame);
      el.textContent = formatCounter(value, decimals);
      prev.current = value;
    };
  }, [value, decimals, duration]);
  return (
    <span key={value} ref={ref} className="tabular">
      {formatCounter(value, decimals)}
    </span>
  );
}
