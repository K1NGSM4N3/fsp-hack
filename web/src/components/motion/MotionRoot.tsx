"use client";

import { useRef, type ReactNode } from "react";
import { FINE_POINTER, MOTION_OK, ScrollTrigger, SplitText, formatCounter, gsap, useGSAP } from "./gsap";

const REVEAL_FROM: Record<string, gsap.TweenVars> = {
  up: { y: 56 },
  left: { x: -56 },
  right: { x: 56 },
  scale: { y: 32, scale: 0.94 },
  fade: {},
};

function heroIntro(root: HTMLElement) {
  const items = gsap.utils.toArray<HTMLElement>("[data-hero]:not([data-split])", root);
  if (!items.length) return;
  gsap.fromTo(
    items,
    { autoAlpha: 0, y: 28 },
    { autoAlpha: 1, y: 0, duration: 1.2, ease: "expo.out", stagger: 0.09, delay: 0.35, clearProps: "transform" },
  );
}

function splitHeadings(root: HTMLElement) {
  for (const el of gsap.utils.toArray<HTMLElement>("[data-split]", root)) {
    const onLoad = el.dataset.split === "hero";
    const delay = Number(el.dataset.delay ?? (onLoad ? 0.1 : 0));
    SplitText.create(el, {
      type: "lines,words",
      mask: "lines",
      linesClass: "split-line",
      autoSplit: true,
      onSplit(self) {
        gsap.set(el, { autoAlpha: 1 });
        return gsap.from(self.lines, {
          yPercent: 108,
          rotate: 2,
          transformOrigin: "0% 100%",
          duration: 1.25,
          ease: "expo.out",
          stagger: 0.085,
          delay,
          scrollTrigger: onLoad ? undefined : { trigger: el, start: "top 88%", once: true },
        });
      },
    });
  }
}

function reveals(root: HTMLElement) {
  const els = gsap.utils.toArray<HTMLElement>("[data-reveal]", root);
  if (!els.length) return;
  for (const el of els) gsap.set(el, { autoAlpha: 0, ...(REVEAL_FROM[el.dataset.reveal || "up"] ?? REVEAL_FROM.up) });
  ScrollTrigger.batch(els, {
    start: "top 90%",
    once: true,
    onEnter: (batch) => gsap.to(batch, { autoAlpha: 1, x: 0, y: 0, scale: 1, duration: 1.1, ease: "expo.out", stagger: 0.1, overwrite: true }),
  });
}

function staggers(root: HTMLElement) {
  for (const box of gsap.utils.toArray<HTMLElement>("[data-stagger]", root)) {
    gsap.from(box.children, {
      autoAlpha: 0,
      y: 40,
      duration: 1,
      ease: "expo.out",
      stagger: Number(box.dataset.stagger) || 0.08,
      scrollTrigger: { trigger: box, start: "top 88%", once: true },
    });
  }
}

function counters(root: HTMLElement): () => void {
  const restore: Array<() => void> = [];
  for (const el of gsap.utils.toArray<HTMLElement>("[data-counter]", root)) {
    const target = Number(el.dataset.counter);
    if (!Number.isFinite(target)) continue;
    const decimals = Number(el.dataset.decimals ?? 0);
    const original = el.textContent;
    restore.push(() => {
      el.textContent = original;
    });
    const state = { v: 0 };
    el.textContent = formatCounter(0, decimals);
    gsap.to(state, {
      v: target,
      duration: 2.2,
      ease: "power3.out",
      scrollTrigger: { trigger: el, start: "top 92%", once: true },
      onUpdate: () => {
        el.textContent = formatCounter(state.v, decimals);
      },
    });
  }
  return () => restore.forEach((f) => f());
}

function drawings(root: HTMLElement) {
  for (const svg of gsap.utils.toArray<SVGSVGElement>("[data-draw]", root)) {
    const onLoad = svg.dataset.draw === "hero";
    const tl = gsap.timeline({
      delay: onLoad ? 0.4 : 0,
      scrollTrigger: onLoad ? undefined : { trigger: svg, start: "top 85%", once: true },
    });
    const strokes = svg.querySelectorAll("path, line, polyline, rect[data-stroke]");
    const nodes = svg.querySelectorAll("circle");
    if (strokes.length) tl.from(strokes, { drawSVG: 0, duration: 1.9, ease: "power2.inOut", stagger: 0.14 });
    if (nodes.length) tl.from(nodes, { scale: 0, transformOrigin: "50% 50%", duration: 0.7, ease: "back.out(2.2)", stagger: 0.05 }, strokes.length ? "-=1.3" : 0);
  }
}

function bars(root: HTMLElement) {
  const els = gsap.utils.toArray<HTMLElement>("[data-grow]", root);
  if (!els.length) return;
  gsap.set(els, { scaleX: 0, transformOrigin: "0% 50%" });
  ScrollTrigger.batch(els, {
    start: "top 92%",
    once: true,
    onEnter: (batch) => gsap.to(batch, { scaleX: 1, duration: 1.4, ease: "expo.out", stagger: 0.06, overwrite: true }),
  });
}

function parallax(root: HTMLElement) {
  for (const el of gsap.utils.toArray<HTMLElement>("[data-parallax]", root)) {
    const amount = Number(el.dataset.parallax) || 0.15;
    gsap.to(el, {
      yPercent: -amount * 100,
      ease: "none",
      scrollTrigger: { trigger: el.parentElement ?? el, start: "top bottom", end: "bottom top", scrub: true },
    });
  }
}

function pointerEffects(root: HTMLElement): () => void {
  const off: Array<() => void> = [];
  for (const el of gsap.utils.toArray<HTMLElement>("[data-magnetic]", root)) {
    const strength = Number(el.dataset.magnetic) || 0.35;
    const xTo = gsap.quickTo(el, "x", { duration: 0.7, ease: "power3.out" });
    const yTo = gsap.quickTo(el, "y", { duration: 0.7, ease: "power3.out" });
    const move = (e: PointerEvent) => {
      const r = el.getBoundingClientRect();
      xTo((e.clientX - (r.left + r.width / 2)) * strength);
      yTo((e.clientY - (r.top + r.height / 2)) * strength);
    };
    const leave = () => {
      xTo(0);
      yTo(0);
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerleave", leave);
    off.push(() => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerleave", leave);
    });
  }
  for (const el of gsap.utils.toArray<HTMLElement>("[data-tilt]", root)) {
    const max = Number(el.dataset.tilt) || 6;
    gsap.set(el, { transformPerspective: 1100, transformStyle: "preserve-3d" });
    const rx = gsap.quickTo(el, "rotationX", { duration: 0.8, ease: "power3.out" });
    const ry = gsap.quickTo(el, "rotationY", { duration: 0.8, ease: "power3.out" });
    const move = (e: PointerEvent) => {
      const r = el.getBoundingClientRect();
      ry(((e.clientX - r.left) / r.width - 0.5) * 2 * max);
      rx(-((e.clientY - r.top) / r.height - 0.5) * 2 * max);
    };
    const leave = () => {
      rx(0);
      ry(0);
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerleave", leave);
    off.push(() => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerleave", leave);
    });
  }
  return () => off.forEach((f) => f());
}

/**
 * Declarative motion for server-rendered pages: children mark elements with data attributes
 * (data-split, data-reveal, data-stagger, data-counter, data-draw, data-grow, data-parallax, data-magnetic, data-tilt)
 * and everything is wired here once. With reduced motion nothing runs and the SSR markup stays as is.
 */
export function MotionRoot({ children, className }: { children: ReactNode; className?: string }) {
  const ref = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const root = ref.current;
      if (!root) return;
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        heroIntro(root);
        splitHeadings(root);
        reveals(root);
        staggers(root);
        drawings(root);
        bars(root);
        parallax(root);
        const restoreCounters = counters(root);
        return restoreCounters;
      });
      mm.add(FINE_POINTER, () => pointerEffects(root));
      void document.fonts?.ready.then(() => ScrollTrigger.refresh());
    },
    { scope: ref },
  );
  return (
    <div ref={ref} className={className}>
      {children}
    </div>
  );
}
