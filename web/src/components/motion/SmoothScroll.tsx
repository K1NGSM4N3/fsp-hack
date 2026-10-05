"use client";

import Lenis from "lenis";
import { usePathname } from "next/navigation";
import { createContext, useCallback, useContext, useEffect, useRef, type ReactNode } from "react";
import { MOTION_OK, ScrollTrigger, gsap } from "./gsap";

const LenisContext = createContext<() => Lenis | null>(() => null);

/** Returns a getter: the instance exists only on the client and only when motion is allowed. */
export function useLenis() {
  return useContext(LenisContext);
}

/** Lenis drives the scroll position from GSAP's ticker so pinned ScrollTriggers and smooth scroll stay in one frame. */
export function SmoothScroll({ children }: { children: ReactNode }) {
  const lenis = useRef<Lenis | null>(null);
  const lastPath = useRef<string | null>(null);
  const pathname = usePathname();

  useEffect(() => {
    if (!window.matchMedia(MOTION_OK).matches) return;
    const instance = new Lenis({ lerp: 0.11, anchors: { offset: -96 }, autoRaf: false, stopInertiaOnNavigate: true });
    const tick = (time: number) => instance.raf(time * 1000);
    instance.on("scroll", ScrollTrigger.update);
    gsap.ticker.add(tick);
    gsap.ticker.lagSmoothing(0);
    lenis.current = instance;
    return () => {
      gsap.ticker.remove(tick);
      gsap.ticker.lagSmoothing(500, 33);
      instance.destroy();
      lenis.current = null;
    };
  }, []);

  useEffect(() => {
    const previous = lastPath.current;
    lastPath.current = pathname;
    if (previous === null || previous === pathname || window.location.hash) return;
    lenis.current?.scrollTo(0, { immediate: true, force: true });
    const id = requestAnimationFrame(() => ScrollTrigger.refresh());
    return () => cancelAnimationFrame(id);
  }, [pathname]);

  const get = useCallback(() => lenis.current, []);
  return <LenisContext.Provider value={get}>{children}</LenisContext.Provider>;
}
