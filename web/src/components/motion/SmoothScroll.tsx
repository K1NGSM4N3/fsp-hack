"use client";

import type Lenis from "lenis";
import { usePathname } from "next/navigation";
import { createContext, useCallback, useContext, useEffect, useRef, type ReactNode } from "react";
import { MOTION_DESKTOP } from "./config";
import { loadKit } from "./lazy";

const LenisContext = createContext<() => Lenis | null>(() => null);

/** Returns a getter: the instance exists only on the client and only with the desktop motion profile. */
export function useLenis() {
  return useContext(LenisContext);
}

/**
 * Lenis drives the scroll position from GSAP's ticker so pinned ScrollTriggers and smooth scroll stay in one frame.
 * Touch devices keep native scrolling (momentum and the address bar behave as users expect) and never download Lenis.
 * lagSmoothing(0) is needed only while Lenis is in charge; otherwise the default lets tweens skip ahead calmly
 * after a long task instead of jumping.
 */
export function SmoothScroll({ children }: { children: ReactNode }) {
  const lenis = useRef<Lenis | null>(null);
  const lastPath = useRef<string | null>(null);
  const pathname = usePathname();

  useEffect(() => {
    const query = window.matchMedia(MOTION_DESKTOP);
    let stop: (() => void) | null = null;
    let generation = 0;

    const start = async (run: number) => {
      const [{ default: LenisClass }, { gsap, ScrollTrigger }] = await Promise.all([import("lenis"), loadKit()]);
      if (run !== generation || lenis.current) return;
      const instance = new LenisClass({ lerp: 0.11, anchors: { offset: -96 }, autoRaf: false, stopInertiaOnNavigate: true });
      const tick = (time: number) => instance.raf(time * 1000);
      instance.on("scroll", ScrollTrigger.update);
      gsap.ticker.add(tick);
      gsap.ticker.lagSmoothing(0);
      lenis.current = instance;
      stop = () => {
        gsap.ticker.remove(tick);
        gsap.ticker.lagSmoothing(500, 33);
        instance.destroy();
        lenis.current = null;
      };
    };
    const sync = () => {
      generation += 1;
      stop?.();
      stop = null;
      if (query.matches) void start(generation);
    };

    sync();
    query.addEventListener("change", sync);
    return () => {
      query.removeEventListener("change", sync);
      generation += 1;
      stop?.();
    };
  }, []);

  useEffect(() => {
    const previous = lastPath.current;
    lastPath.current = pathname;
    const instance = lenis.current;
    if (!instance || previous === null || previous === pathname || window.location.hash) return;
    instance.scrollTo(0, { immediate: true, force: true });
    const id = requestAnimationFrame(() => void loadKit().then(({ ScrollTrigger }) => ScrollTrigger.refresh()));
    return () => cancelAnimationFrame(id);
  }, [pathname]);

  const get = useCallback(() => lenis.current, []);
  return <LenisContext.Provider value={get}>{children}</LenisContext.Provider>;
}
