"use client";

import { ArrowRight, ArrowUpRight, Menu, X } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useCallback, useEffect, useRef, useState } from "react";
import { Logo } from "@/components/brand/Logo";
import { useMotion } from "@/components/motion/lazy";
import { useLenis } from "@/components/motion/SmoothScroll";
import type { Me } from "@/api/types";
import { useSessionRole } from "@/features/auth/session";
import { cn } from "@/lib/cn";
import { SITE_NAV } from "./nav";

function AuthActions({ role, stacked = false, onNavigate }: { role: Me["role"] | null; stacked?: boolean; onNavigate?: () => void }) {
  if (role) {
    const href = role === "employer" ? "/employer" : "/candidate";
    return (
      <Link
        href={href}
        onClick={onNavigate}
        className={cn(
          "inline-flex h-10 items-center justify-center gap-2 rounded-full bg-ink px-5 text-sm font-medium text-white transition-colors hover:bg-primary",
          stacked && "h-12 w-full text-base",
        )}
      >
        Мой кабинет
        <ArrowRight className="size-4" aria-hidden />
      </Link>
    );
  }
  return (
    <div className={cn("flex items-center gap-1.5", stacked && "flex-col-reverse items-stretch gap-3")}>
      <Link
        href="/login"
        onClick={onNavigate}
        className={cn(
          "inline-flex h-10 items-center justify-center rounded-full px-4 text-sm font-medium transition-colors",
          stacked ? "h-12 border border-night-line text-base text-white hover:border-white" : "text-ink-muted hover:text-ink",
        )}
      >
        Войти
      </Link>
      <Link
        href="/register"
        onClick={onNavigate}
        className={cn(
          "group inline-flex h-10 items-center justify-center gap-2 rounded-full bg-primary px-5 text-sm font-medium text-white shadow-[0_8px_24px_-10px_rgb(64_47_255/0.8)] transition-colors hover:bg-primary-hover",
          stacked && "h-12 text-base",
        )}
      >
        Начать
        <ArrowRight className="size-4 transition-transform group-hover:translate-x-0.5" aria-hidden />
      </Link>
    </div>
  );
}

export function SiteHeader() {
  const pathname = usePathname();
  const getLenis = useLenis();
  const role = useSessionRole();
  const [open, setOpen] = useState(false);
  const shell = useRef<HTMLDivElement>(null);
  const bar = useRef<HTMLDivElement>(null);
  const nav = useRef<HTMLElement>(null);
  const indicator = useRef<HTMLSpanElement>(null);

  const activeHref = SITE_NAV.find((l) => pathname === l.href || pathname.startsWith(`${l.href}/`))?.href;

  useMotion(
    ({ MOTION_DESKTOP, MOTION_OK, ScrollTrigger, gsap }) => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_DESKTOP, () => {
        gsap.from(bar.current, { yPercent: -160, autoAlpha: 0, duration: 1.2, ease: "expo.out", delay: 0.1 });
      });
      mm.add(MOTION_OK, () => {
        let hidden = false;
        ScrollTrigger.create({
          start: 0,
          end: "max",
          onUpdate: (self) => {
            const y = self.scroll();
            bar.current?.toggleAttribute("data-scrolled", y > 24);
            const hide = self.direction === 1 && y > 220;
            if (hide !== hidden) {
              hidden = hide;
              gsap.to(bar.current, { yPercent: hide ? -170 : 0, duration: 0.55, ease: "power3.out", overwrite: "auto" });
            }
          },
        });
      });
    },
    { scope: shell, urgent: true },
  );

  const moveIndicator = useCallback((target: HTMLElement | null, instant = false) => {
    const el = indicator.current;
    if (!el) return;
    el.style.transitionDuration = instant ? "0s" : "";
    if (!target) {
      el.style.opacity = "0";
      return;
    }
    el.style.transform = `translateX(${target.offsetLeft}px)`;
    el.style.width = `${target.offsetWidth}px`;
    el.style.opacity = "1";
  }, []);

  useEffect(() => {
    const current = activeHref ? nav.current?.querySelector<HTMLElement>(`[data-href="${activeHref}"]`) ?? null : null;
    moveIndicator(current, true);
    const refit = () => moveIndicator(current, true);
    void document.fonts?.ready.then(refit);
    window.addEventListener("resize", refit);
    return () => window.removeEventListener("resize", refit);
  }, [activeHref, moveIndicator]);

  useEffect(() => {
    if (!open) return;
    const lenis = getLenis();
    lenis?.stop();
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      lenis?.start();
      document.body.style.overflow = prev;
      window.removeEventListener("keydown", onKey);
    };
  }, [open, getLenis]);

  const close = () => setOpen(false);

  return (
    <div ref={shell} className="pointer-events-none fixed inset-x-0 top-0 z-50 px-3 pt-3 sm:px-5 sm:pt-4 print:hidden">
      <div
        ref={bar}
        className="group/bar pointer-events-auto mx-auto flex h-14 max-w-[1240px] items-center justify-between gap-3 rounded-full border border-white/60 bg-white/75 pr-2 pl-4 shadow-[0_10px_40px_-18px_rgb(11_11_18/0.35)] backdrop-blur-xl backdrop-saturate-150 transition-[background-color,box-shadow] duration-300 data-[scrolled]:bg-white/90 data-[scrolled]:shadow-[0_14px_44px_-16px_rgb(11_11_18/0.4)] sm:h-16 sm:pl-5"
      >
        <Logo />
        <nav
          ref={nav}
          aria-label="Основная навигация"
          className="relative hidden items-center lg:flex"
          onMouseLeave={() => moveIndicator(activeHref ? (nav.current?.querySelector<HTMLElement>(`[data-href="${activeHref}"]`) ?? null) : null)}
        >
          <span
            ref={indicator}
            aria-hidden
            className="absolute top-0 left-0 h-full w-0 rounded-full bg-primary-50 opacity-0 transition-[transform,width,opacity] duration-500 ease-out-expo"
          />
          {SITE_NAV.map((l) => {
            const active = l.href === activeHref;
            return (
              <Link
                key={l.href}
                href={l.href}
                data-href={l.href}
                aria-current={active ? "page" : undefined}
                onMouseEnter={(e) => moveIndicator(e.currentTarget)}
                onFocus={(e) => moveIndicator(e.currentTarget)}
                className={cn(
                  "relative z-10 rounded-full px-3 py-2.5 text-sm font-medium transition-colors xl:px-3.5",
                  active ? "text-primary-strong" : "text-ink-muted hover:text-ink",
                )}
              >
                {l.label}
              </Link>
            );
          })}
        </nav>
        <div className="hidden lg:block">
          <AuthActions role={role} />
        </div>
        <button
          type="button"
          onClick={() => setOpen(true)}
          aria-expanded={open}
          aria-controls="site-menu"
          aria-label="Открыть меню"
          className="inline-flex size-10 items-center justify-center rounded-full bg-ink text-white lg:hidden"
        >
          <Menu className="size-5" aria-hidden />
        </button>
      </div>

      {open && (
        <div
          id="site-menu"
          role="dialog"
          aria-modal="true"
          aria-label="Меню сайта"
          className="pointer-events-auto fixed inset-0 z-50 flex animate-fade-in flex-col overflow-y-auto bg-night px-5 pt-5 pb-8 text-white lg:hidden"
        >
          <div className="pointer-events-none absolute -top-40 -right-40 size-[28rem] rounded-full bg-primary/40 blur-[120px]" aria-hidden />
          <div className="relative flex items-center justify-between">
            <Logo tone="dark" />
            <button
              type="button"
              onClick={close}
              aria-label="Закрыть меню"
              className="inline-flex size-11 items-center justify-center rounded-full border border-night-line text-white"
            >
              <X className="size-5" aria-hidden />
            </button>
          </div>
          <nav aria-label="Основная навигация" className="stagger-rise relative mt-10 flex flex-col">
            {SITE_NAV.map((l, i) => (
              <Link
                key={l.href}
                href={l.href}
                onClick={close}
                className="group flex items-baseline justify-between gap-4 border-b border-night-line py-4"
              >
                <span className="flex items-baseline gap-4">
                  <span className="font-mono text-xs text-night-faint">0{i + 1}</span>
                  <span className={cn("text-[1.7rem] leading-none font-semibold tracking-tight", l.href === activeHref ? "text-primary-300" : "text-white")}>{l.label}</span>
                </span>
                <ArrowUpRight className="size-5 shrink-0 text-night-faint transition-colors group-hover:text-white" aria-hidden />
              </Link>
            ))}
          </nav>
          <div className="relative mt-auto animate-rise pt-10 [animation-delay:240ms]">
            <AuthActions role={role} stacked onNavigate={close} />
          </div>
        </div>
      )}
    </div>
  );
}
