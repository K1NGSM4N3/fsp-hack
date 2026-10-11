import { FINE_POINTER, MOTION_DESKTOP, MOTION_MOBILE, MOTION_OK, ScrollTrigger, SplitText, formatCounter, gsap, refreshAfterFonts } from "./gsap";

export type MotionVariant = "rise" | "wipe" | "center" | "blur";

interface Look {
  split: { type: string; mask?: "lines" | "words" | "chars"; pick: (s: SplitText) => Element[]; from: gsap.TweenVars; stagger: gsap.TweenVars["stagger"]; duration: number; ease: string };
  reveal: { from: gsap.TweenVars; to: gsap.TweenVars };
  stagger: gsap.StaggerVars;
}

/**
 * Reveals fade with opacity, never visibility: content waiting below the fold stays in the accessibility tree and in
 * the tab order (focusing it scrolls it into view, which plays the reveal).
 */
const SHOW: gsap.TweenVars = { opacity: 1, x: 0, y: 0, scale: 1 };

const REVEAL: Record<string, { from: gsap.TweenVars; to: gsap.TweenVars }> = {
  up: { from: { opacity: 0, y: 56 }, to: SHOW },
  left: { from: { opacity: 0, x: -56 }, to: SHOW },
  right: { from: { opacity: 0, x: 56 }, to: SHOW },
  scale: { from: { opacity: 0, y: 32, scale: 0.94 }, to: SHOW },
  fade: { from: { opacity: 0 }, to: SHOW },
  wipe: { from: { clipPath: "inset(0% 100% 0% 0%)", x: -24 }, to: { clipPath: "inset(0% 0% 0% 0%)", x: 0, clearProps: "clipPath" } },
  center: { from: { clipPath: "inset(50% 50% 50% 50% round 28px)", scale: 0.97 }, to: { clipPath: "inset(0% 0% 0% 0% round 28px)", scale: 1, clearProps: "clipPath" } },
  blur: { from: { opacity: 0, filter: "blur(18px)", scale: 0.97 }, to: { opacity: 1, filter: "blur(0px)", scale: 1, clearProps: "filter" } },
};

const LOOKS: Record<MotionVariant, Look> = {
  rise: {
    split: { type: "lines,words", mask: "lines", pick: (s) => s.lines, from: { yPercent: 108, rotate: 2, transformOrigin: "0% 100%" }, stagger: 0.085, duration: 1.25, ease: "expo.out" },
    reveal: REVEAL.up,
    stagger: { each: 0.08, from: "start" },
  },
  wipe: {
    split: { type: "words,lines", mask: "words", pick: (s) => s.words, from: { yPercent: 110 }, stagger: 0.035, duration: 0.95, ease: "power4.out" },
    reveal: REVEAL.wipe,
    stagger: { each: 0.09, from: "start" },
  },
  center: {
    split: { type: "chars,words", pick: (s) => s.chars, from: { autoAlpha: 0, yPercent: 45, rotateX: -85, transformOrigin: "50% 100%" }, stagger: { each: 0.014, from: "center" }, duration: 0.85, ease: "back.out(1.8)" },
    reveal: REVEAL.center,
    stagger: { each: 0.07, from: "center" },
  },
  blur: {
    split: { type: "lines", pick: (s) => s.lines, from: { autoAlpha: 0, filter: "blur(16px)", y: 28 }, stagger: 0.12, duration: 1.3, ease: "power3.out" },
    reveal: REVEAL.blur,
    stagger: { each: 0.09, from: "edges" },
  },
};

function heroIntro(root: HTMLElement, look: Look) {
  const items = gsap.utils.toArray<HTMLElement>('[data-hero]:not([data-split]):not([data-hero="lead"])', root);
  const leads = gsap.utils.toArray<HTMLElement>('[data-hero="lead"]', root);
  if (leads.length) gsap.from(leads, { y: 22, duration: 1.4, ease: "expo.out", delay: 0.45, clearProps: "transform" });
  if (!items.length) return;
  const base = look === LOOKS.rise ? { from: { autoAlpha: 0, y: 28 }, to: { autoAlpha: 1, y: 0 } } : look.reveal;
  gsap.fromTo(items, { ...base.from, autoAlpha: 0 }, { ...base.to, autoAlpha: 1, duration: 1.2, ease: "expo.out", stagger: 0.09, delay: 0.35, clearProps: "transform,clipPath,filter" });
}

function splitHeadings(root: HTMLElement, look: Look) {
  const cfg = look.split;
  for (const el of gsap.utils.toArray<HTMLElement>("[data-split]", root)) {
    const onLoad = el.dataset.split === "hero";
    const delay = Number(el.dataset.delay ?? (onLoad ? 0.1 : 0));
    SplitText.create(el, {
      type: cfg.type,
      mask: cfg.mask,
      linesClass: "split-line",
      wordsClass: "split-word",
      charsClass: "split-char",
      autoSplit: true,
      onSplit(self) {
        gsap.set(el, { autoAlpha: 1, perspective: 700 });
        return gsap.from(cfg.pick(self), {
          ...cfg.from,
          duration: cfg.duration,
          ease: cfg.ease,
          stagger: cfg.stagger,
          delay,
          scrollTrigger: onLoad ? undefined : { trigger: el, start: "top 88%", once: true },
        });
      },
    });
  }
}

function reveals(root: HTMLElement, look: Look) {
  const els = gsap.utils.toArray<HTMLElement>("[data-reveal]", root);
  if (!els.length) return;
  const kindOf = (el: HTMLElement) => (el.dataset.reveal ? (REVEAL[el.dataset.reveal] ?? look.reveal) : look.reveal);
  for (const el of els) gsap.set(el, kindOf(el).from);
  ScrollTrigger.batch(els, {
    start: "top 90%",
    once: true,
    onEnter: (batch) =>
      batch.forEach((el, i) => gsap.to(el, { ...kindOf(el as HTMLElement).to, duration: 1.15, ease: "expo.out", delay: i * 0.1, overwrite: true })),
  });
}

function staggers(root: HTMLElement, look: Look) {
  for (const box of gsap.utils.toArray<HTMLElement>("[data-stagger]", root)) {
    const kind = look.reveal;
    gsap.fromTo(box.children, kind.from, {
      ...kind.to,
      duration: 1,
      ease: "expo.out",
      stagger: { ...look.stagger, each: Number(box.dataset.stagger) || look.stagger.each },
      scrollTrigger: { trigger: box, start: "top 88%", once: true },
    });
  }
}

function scrambles(root: HTMLElement): () => void {
  const restore: Array<() => void> = [];
  for (const el of gsap.utils.toArray<HTMLElement>("[data-scramble]", root)) {
    const text = el.textContent ?? "";
    restore.push(() => {
      el.textContent = text;
    });
    gsap.to(el, {
      duration: 1.4,
      scrambleText: { text, chars: "0123456789", revealDelay: 0.3, speed: 0.4 },
      scrollTrigger: { trigger: el, start: "top 92%", once: true },
    });
  }
  return () => restore.forEach((f) => f());
}

function velocitySkew(root: HTMLElement) {
  const els = gsap.utils.toArray<HTMLElement>("[data-skew]", root);
  if (!els.length) return;
  const skew = gsap.quickTo(els, "skewY", { duration: 0.5, ease: "power3.out" });
  let settle: gsap.core.Tween | null = null;
  ScrollTrigger.create({
    trigger: root,
    start: "top bottom",
    end: "bottom top",
    onUpdate: (self) => {
      skew(gsap.utils.clamp(-6, 6, self.getVelocity() / -300));
      settle?.kill();
      settle = gsap.delayedCall(0.12, () => skew(0));
    },
  });
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

/** Only what starts below the fold gets hidden on phones, so nothing already on screen blinks out at hydration. */
function belowFold(els: HTMLElement[]): HTMLElement[] {
  const fold = window.innerHeight * 0.92;
  const tops = els.map((el) => el.getBoundingClientRect().top);
  return els.filter((_, i) => (tops[i] ?? 0) > fold);
}

const MOBILE_FROM: Record<string, gsap.TweenVars> = {
  fade: { opacity: 0 },
  scale: { opacity: 0, y: 20, scale: 0.97 },
};
const MOBILE_UP: gsap.TweenVars = { opacity: 0, y: 24 };

/**
 * Phone language: headings and blocks rise a little and fade in once, in batches as they enter. No splitting, no clip
 * paths or blur filters, no sideways offsets that could widen the page, nothing tied to scroll position.
 */
function mobileReveals(root: HTMLElement) {
  const els = belowFold(gsap.utils.toArray<HTMLElement>('[data-reveal], [data-split]:not([data-split="hero"])', root));
  const boxes = belowFold(gsap.utils.toArray<HTMLElement>("[data-stagger]", root));
  for (const el of els) gsap.set(el, MOBILE_FROM[el.dataset.reveal ?? ""] ?? MOBILE_UP);
  if (els.length) {
    ScrollTrigger.batch(els, {
      start: "top 92%",
      once: true,
      onEnter: (batch) => gsap.to(batch, { opacity: 1, y: 0, scale: 1, duration: 0.75, ease: "power3.out", stagger: 0.08, overwrite: true }),
    });
  }
  for (const box of boxes) {
    gsap.fromTo(
      box.children,
      { opacity: 0, y: 16 },
      { opacity: 1, y: 0, duration: 0.6, ease: "power3.out", stagger: 0.05, scrollTrigger: { trigger: box, start: "top 90%", once: true } },
    );
  }
}

export function setupMotionRoot(root: HTMLElement, variant: MotionVariant) {
  const look = LOOKS[variant];
  const mm = gsap.matchMedia();
  mm.add(MOTION_DESKTOP, () => {
    heroIntro(root, look);
    splitHeadings(root, look);
    reveals(root, look);
    staggers(root, look);
    parallax(root);
    velocitySkew(root);
  });
  mm.add(MOTION_MOBILE, () => mobileReveals(root));
  mm.add(MOTION_OK, () => {
    drawings(root);
    bars(root);
    const restoreCounters = counters(root);
    const restoreScrambles = scrambles(root);
    return () => {
      restoreCounters();
      restoreScrambles();
    };
  });
  mm.add(FINE_POINTER, () => pointerEffects(root));
  refreshAfterFonts();
}
