"use client";

import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { DrawSVGPlugin } from "gsap/DrawSVGPlugin";
import { ScrambleTextPlugin } from "gsap/ScrambleTextPlugin";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { SplitText } from "gsap/SplitText";

gsap.registerPlugin(useGSAP, ScrollTrigger, SplitText, DrawSVGPlugin, ScrambleTextPlugin);

export { DrawSVGPlugin, ScrambleTextPlugin, ScrollTrigger, SplitText, gsap, useGSAP };

export const MOTION_OK = "(prefers-reduced-motion: no-preference)";
export const MOTION_DESKTOP = "(prefers-reduced-motion: no-preference) and (min-width: 1024px)";
export const FINE_POINTER = "(prefers-reduced-motion: no-preference) and (pointer: fine)";

const numberFormats = new Map<number, Intl.NumberFormat>();

export function formatCounter(value: number, decimals: number): string {
  let f = numberFormats.get(decimals);
  if (!f) {
    f = new Intl.NumberFormat("ru-RU", { minimumFractionDigits: decimals, maximumFractionDigits: decimals });
    numberFormats.set(decimals, f);
  }
  return f.format(value);
}
