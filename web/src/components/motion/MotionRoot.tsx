"use client";

import { useRef, type ReactNode } from "react";
import { useMotion } from "./lazy";
import type { MotionVariant } from "./MotionRoot.motion";

export type { MotionVariant };

const loadScene = () => import("./MotionRoot.motion");

/**
 * Declarative motion for server-rendered pages: children mark elements with data attributes
 * (data-split, data-reveal, data-stagger, data-counter, data-scramble, data-draw, data-grow, data-parallax, data-skew, data-magnetic, data-tilt)
 * and everything is wired here once; `variant` gives each page its own entrance language on desktop.
 * Phones and tablets get one lighter language for every page; with reduced motion nothing runs and the SSR markup stays.
 */
export function MotionRoot({ children, className, variant = "rise" }: { children: ReactNode; className?: string; variant?: MotionVariant }) {
  const ref = useRef<HTMLDivElement>(null);
  useMotion(
    (_, scene) => {
      if (ref.current) scene.setupMotionRoot(ref.current, variant);
    },
    { scope: ref, dependencies: [variant], extra: loadScene, urgent: true },
  );
  return (
    <div ref={ref} className={className}>
      {children}
    </div>
  );
}
