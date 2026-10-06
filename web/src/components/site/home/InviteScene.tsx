"use client";

import { Check, Mail } from "lucide-react";
import { useRef, type ReactNode } from "react";
import { type Kit, useMotion } from "@/components/motion/lazy";
import { cn } from "@/lib/cn";
import { Container } from "../primitives";

const loadMotionPath = () => import("@/components/motion/extras/motion-path");

type Side = "seeker" | "company";

const STAGES: { side: Side; tag: string; title: string; text: string }[] = [
  {
    side: "seeker",
    tag: "Вы",
    title: "Вы проходите тест",
    text: "8–20 заданий по вашему направлению, на каждое от одной до трёх минут. По итогам вы получаете категорию, например «Бэкенд · Middle», и появляетесь в подборках компаний. Имя и контакты при этом скрыты.",
  },
  {
    side: "company",
    tag: "Компания",
    title: "Компания находит вас в подборке",
    text: "В списке только люди, которые прошли тест на нужную категорию. Компания видит ваши навыки и ожидания по зарплате, выбирает вас и нажимает «Пригласить».",
  },
  {
    side: "company",
    tag: "Компания → вам",
    title: "Приглашение уходит с вилкой",
    text: "Внутри описание задачи и зарплата от и до. Без вилки приглашение не отправить. В неделю у компании не больше 30 приглашений, у новой — 10, поэтому их получают те, кого действительно выбрали.",
  },
  {
    side: "seeker",
    tag: "Вы",
    title: "Сумма видна прямо в уведомлении",
    text: "Ещё до созвона понятно, стоит ли разговор вашего времени. На ответ есть 14 дней. Отклонить можно одним нажатием и указать причину, компания её увидит.",
  },
  {
    side: "seeker",
    tag: "Вы → компании",
    title: "«Принять» открывает контакты",
    text: "Только теперь компания видит ваше имя и контакты. Другие компании их не получают.",
  },
];

function offsetWithin(el: HTMLElement, root: HTMLElement) {
  let x = 0;
  let y = 0;
  let node: HTMLElement | null = el;
  while (node && node !== root) {
    x += node.offsetLeft;
    y += node.offsetTop;
    node = node.offsetParent instanceof HTMLElement ? node.offsetParent : null;
  }
  return { x, y };
}

function centerWithin(el: HTMLElement, root: HTMLElement, fx = 0.5, fy = 0.5) {
  const o = offsetWithin(el, root);
  return { x: o.x + el.offsetWidth * fx, y: o.y + el.offsetHeight * fy };
}

/**
 * Phones and tablets: the five steps stay a vertical list that rises in step by step, and once the devices scroll into
 * view the last act plays by itself in about three seconds: the push lands on the lock screen, the invitation opens,
 * «Принять» is tapped and the laptop unlocks the contacts. Time-based, transforms and opacity only.
 */
function mobileStory({ gsap, ScrollTrigger }: Pick<Kit, "gsap" | "ScrollTrigger">, el: HTMLElement) {
  const one = (sel: string) => el.querySelector<HTMLElement>(sel);
  const devices = one("[data-s-devices]");
  const fold = window.innerHeight;
  const devicesTop = devices?.getBoundingClientRect().top ?? 0;
  const caps = gsap.utils.toArray<HTMLElement>("[data-s-cap]", el).filter((cap) => cap.getBoundingClientRect().top > fold * 0.92);
  if (caps.length) {
    gsap.set(caps, { autoAlpha: 0, y: 22 });
    ScrollTrigger.batch(caps, {
      start: "top 90%",
      once: true,
      onEnter: (batch) => gsap.to(batch, { autoAlpha: 1, y: 0, duration: 0.7, ease: "power3.out", stagger: 0.1, overwrite: true }),
    });
  }
  if (!devices || devicesTop < fold * 0.75) return;

  const phone = one("[data-s-phone]");
  const push = one("[data-s-push]");
  const openItems = gsap.utils.toArray<HTMLElement>("[data-s-open-item]", el);
  gsap.set(one("[data-s-scr-lock]"), { autoAlpha: 1 });
  gsap.set(push, { autoAlpha: 0, yPercent: -140 });
  gsap.set(one("[data-s-scr-inv]"), { autoAlpha: 0 });
  gsap.set(one("[data-s-actions]"), { autoAlpha: 1 });
  gsap.set(one("[data-s-accepted]"), { autoAlpha: 0 });
  gsap.set(one("[data-s-lock]"), { autoAlpha: 1 });
  gsap.set(openItems, { autoAlpha: 0, y: 6 });
  gsap.set(one("[data-s-btn-sent]"), { autoAlpha: 1 });
  gsap.set(one("[data-s-btn-accepted]"), { autoAlpha: 0 });

  gsap
    .timeline({ delay: 0.3, scrollTrigger: { trigger: devices, start: "top 70%", once: true } })
    .to(push, { autoAlpha: 1, yPercent: 0, duration: 0.55, ease: "back.out(1.5)" })
    .to(phone, { scale: 1.02, duration: 0.18, ease: "power2.out", transformOrigin: "50% 100%" }, "<0.1")
    .to(phone, { scale: 1, duration: 0.3, ease: "power2.inOut" })
    .to(push, { autoAlpha: 0, scale: 1.05, duration: 0.25 }, "+=0.7")
    .to(one("[data-s-scr-lock]"), { autoAlpha: 0, duration: 0.25 }, "<")
    .fromTo(one("[data-s-scr-inv]"), { scale: 0.95 }, { autoAlpha: 1, scale: 1, duration: 0.4, ease: "power3.out" }, "<0.1")
    .set(one("[data-s-tap]"), { autoAlpha: 0.8, scale: 0.2 }, "+=0.6")
    .to(one("[data-s-tap]"), { autoAlpha: 0, scale: 1.8, duration: 0.4, ease: "power2.out" })
    .to(one("[data-s-accept-btn]"), { scale: 0.94, duration: 0.08, yoyo: true, repeat: 1 }, "<")
    .to(one("[data-s-actions]"), { autoAlpha: 0, y: 6, duration: 0.2 }, "<0.2")
    .fromTo(one("[data-s-accepted]"), { y: 8 }, { autoAlpha: 1, y: 0, duration: 0.35, ease: "power3.out" })
    .to(one("[data-s-lock]"), { autoAlpha: 0, duration: 0.25 }, "+=0.15")
    .to(openItems, { autoAlpha: 1, y: 0, duration: 0.35, ease: "power3.out", stagger: 0.08 }, "<0.1")
    .to(one("[data-s-btn-sent]"), { autoAlpha: 0, duration: 0.2 }, "<")
    .fromTo(one("[data-s-btn-accepted]"), { y: 6 }, { autoAlpha: 1, y: 0, duration: 0.3, ease: "power3.out" }, "<0.1");
}

/**
 * Desktop: a pinned split-screen story, the company's laptop and the job seeker's phone, one invitation travelling
 * between them. Server markup is the static version (all captions listed, devices in their final state); the desktop
 * profile gives the root data-live, switches to the pinned layout and the timeline rewinds the devices to the first stage.
 * The devices are server components passed in as props, so their markup ships no JS.
 */
export function InviteScene({ laptop, phone }: { laptop: ReactNode; phone: ReactNode }) {
  const root = useRef<HTMLElement>(null);

  useMotion(
    ({ MOTION_DESKTOP, MOTION_MOBILE, ScrollTrigger, SplitText, gsap }) => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_MOBILE, () => {
        if (root.current) mobileStory({ gsap, ScrollTrigger }, root.current);
      });
      mm.add(MOTION_DESKTOP, (context) => {
        const el = root.current;
        if (!el) return;
        const one = (sel: string) => el.querySelector<HTMLElement>(sel);
        const all = (sel: string) => gsap.utils.toArray<HTMLElement>(sel, el);

        const stage = one("[data-s-stage]");
        const screen = one("[data-s-screen]");
        const origin = one("[data-s-origin]");
        const target = one("[data-s-push]");
        const overlay = el.querySelector<SVGSVGElement>("[data-s-overlay]");
        const route = el.querySelector<SVGPathElement>("[data-s-route]");
        const trail = el.querySelector<SVGPathElement>("[data-s-trail]");
        const row0 = one('[data-s-row="0"]');
        const counter = one("[data-s-count]");
        if (!stage || !screen || !origin || !target || !overlay || !route || !trail || !row0 || !counter) return;

        el.dataset.live = "";

        const caps = all("[data-s-cap]");
        const capLines: HTMLElement[][] = caps.map(() => []);
        let current = 0;

        caps.forEach((cap, i) => {
          SplitText.create(cap.querySelectorAll("[data-s-cap-line]"), {
            type: "lines",
            mask: "lines",
            aria: "none",
            linesClass: "split-line",
            autoSplit: true,
            onSplit(self) {
              capLines[i] = self.lines as HTMLElement[];
              gsap.set(self.lines, { yPercent: i === current ? 0 : 105 });
            },
          });
          gsap.set(cap, { autoAlpha: i === 0 ? 1 : 0 });
        });

        const show = context.add("show", (next: number) => {
          if (next === current) return;
          const prev = current;
          current = next;
          const dir = next > prev ? 1 : -1;
          caps.forEach((cap, i) => {
            if (i === prev || i === next) return;
            gsap.killTweensOf(capLines[i] ?? []);
            gsap.set(cap, { autoAlpha: 0 });
          });
          const out = capLines[prev] ?? [];
          const inc = capLines[next] ?? [];
          gsap.killTweensOf([...out, ...inc]);
          gsap.to(out, {
            yPercent: -105 * dir,
            duration: 0.35,
            ease: "power3.in",
            stagger: 0.02,
            onComplete: () => {
              if (current !== prev) gsap.set(caps[prev] ?? [], { autoAlpha: 0 });
            },
          });
          gsap.set(caps[next] ?? [], { autoAlpha: 1 });
          gsap.fromTo(inc, { yPercent: 105 * dir }, { yPercent: 0, duration: 0.85, ease: "expo.out", stagger: 0.045, delay: 0.18 });
          gsap.to(counter, { duration: 0.5, scrambleText: { text: String(next + 1).padStart(2, "0"), chars: "0123456789", speed: 0.6 } });
        }) as (next: number) => void;

        const layout = () => {
          const w = stage.clientWidth;
          const h = stage.clientHeight;
          overlay.setAttribute("viewBox", `0 0 ${w} ${h}`);
          const s = centerWithin(origin, stage);
          const e = centerWithin(target, stage, 0.5, 0.3);
          const dx = e.x - s.x;
          const dy = e.y - s.y;
          const vy = Math.abs(dy) < 40 ? 1 : Math.sign(dy);
          const c1 = [s.x, s.y + vy * Math.max(90, Math.abs(dy) * 0.75)];
          const c2 = [e.x - Math.sign(dx || 1) * Math.max(120, Math.abs(dx) * 0.55), e.y];
          const d = `M${s.x.toFixed(1)},${s.y.toFixed(1)} C${c1.map((v) => v.toFixed(1)).join(",")} ${c2.map((v) => v.toFixed(1)).join(",")} ${e.x.toFixed(1)},${e.y.toFixed(1)}`;
          route.setAttribute("d", d);
          trail.setAttribute("d", d);
        };
        layout();
        ScrollTrigger.addEventListener("refreshInit", layout);

        const laptop = one("[data-s-laptop]");
        const phone = one("[data-s-phone]");
        const lapDim = one("[data-s-laptop] [data-s-dim]");
        const phoneDim = one("[data-s-phone] [data-s-dim]");
        const rest = all("[data-s-row]").slice(1);
        const cursor = one("[data-s-cursor]");
        const invite = one("[data-s-btn-invite]");
        const flight = one("[data-s-flight]");
        const flightBack = one("[data-s-flight-back]");
        const push = target;
        const openItems = all("[data-s-open-item]");
        const narrow = () => !window.matchMedia("(min-width: 1024px)").matches;
        const phoneAside = () => (narrow() && phone ? phone.offsetHeight * 0.3 : 0);
        const rowTarget = () => centerWithin(row0, screen, 0.42, 0.55);
        const btnTarget = () => centerWithin(origin, screen, 0.5, 0.6);

        gsap.set([laptop, phone], { transformOrigin: "50% 100%" });
        gsap.set(laptop, { scale: 0.97 });
        gsap.set(lapDim, { opacity: 0.6 });
        gsap.set(one("[data-s-row-hl]"), { autoAlpha: 0 });
        gsap.set(one("[data-s-empty]"), { autoAlpha: 1 });
        gsap.set(one("[data-s-lock]"), { autoAlpha: 1 });
        gsap.set(openItems, { autoAlpha: 0, y: 6 });
        gsap.set(invite, { autoAlpha: 1 });
        gsap.set([one("[data-s-btn-sent]"), one("[data-s-btn-accepted]")], { autoAlpha: 0 });
        gsap.set(one("[data-s-scr-test]"), { autoAlpha: 1 });
        gsap.set([one("[data-s-scr-cat]"), one("[data-s-scr-inv]")], { autoAlpha: 0 });
        gsap.set(one("[data-s-scr-lock]"), { autoAlpha: 0 });
        gsap.set(push, { autoAlpha: 0, yPercent: -140 });
        gsap.set(one("[data-s-actions]"), { autoAlpha: 1 });
        gsap.set(one("[data-s-accepted]"), { autoAlpha: 0 });
        gsap.set([flight, flightBack], { autoAlpha: 0, scale: 0.4, xPercent: -50, yPercent: -50, x: 0, y: 0 });
        gsap.set(route, { autoAlpha: 0 });
        gsap.set(trail, { drawSVG: "0% 0%" });
        gsap.set(all("[data-s-seg]"), { scaleX: 0, transformOrigin: "0% 50%" });

        const tl = gsap.timeline({
          defaults: { ease: "power2.inOut", duration: 0.2 },
          scrollTrigger: {
            trigger: stage,
            pin: true,
            start: "top top",
            end: () => `+=${Math.round(window.innerHeight * 4.4)}`,
            scrub: 0.7,
            invalidateOnRefresh: true,
            anticipatePin: 1,
            onUpdate: (self) => show(Math.min(STAGES.length - 1, Math.floor(self.progress * STAGES.length))),
          },
        });

        all("[data-s-seg]").forEach((seg, i) => tl.to(seg, { scaleX: 1, ease: "none", duration: 1 }, i));

        tl.fromTo(one("[data-s-timer]"), { drawSVG: "0% 100%" }, { drawSVG: "0% 55%", ease: "none", duration: 0.5 }, 0.04)
          .fromTo(one("[data-s-opt-hl]"), { autoAlpha: 0 }, { autoAlpha: 1, duration: 0.1 }, 0.26)
          .fromTo(one("[data-s-opt-dot]"), { scale: 0 }, { scale: 1, duration: 0.1, ease: "back.out(2.5)" }, 0.3)
          .to(one("[data-s-answer]"), { scale: 0.94, duration: 0.04 }, 0.42)
          .to(one("[data-s-answer]"), { scale: 1, duration: 0.04 }, 0.46)
          .to(one("[data-s-scr-test]"), { autoAlpha: 0, scale: 0.96, duration: 0.14 }, 0.52)
          .fromTo(one("[data-s-scr-cat]"), { scale: 1.05 }, { autoAlpha: 1, scale: 1, duration: 0.16 }, 0.54)
          .fromTo(one("[data-s-check]"), { scale: 0, rotate: -40 }, { scale: 1, rotate: 0, duration: 0.16, ease: "back.out(2.2)" }, 0.62);

        tl.to(phone, { scale: 0.97, y: phoneAside, duration: 0.15 }, 1)
          .to(phoneDim, { opacity: 0.55, duration: 0.15 }, 1)
          .to(laptop, { scale: 1, duration: 0.15 }, 1)
          .to(lapDim, { opacity: 0, duration: 0.15 }, 1)
          .to(one("[data-s-scr-cat]"), { autoAlpha: 0, duration: 0.1 }, 1.04)
          .to(one("[data-s-scr-lock]"), { autoAlpha: 1, duration: 0.1 }, 1.06)
          .fromTo(rest, { y: () => -row0.offsetHeight }, { y: 0, duration: 0.16, ease: "power3.out" }, 1.1)
          .fromTo(row0, { autoAlpha: 0, x: -18 }, { autoAlpha: 1, x: 0, duration: 0.14, ease: "power3.out" }, 1.16)
          .fromTo(cursor, { autoAlpha: 0, x: () => screen.clientWidth * 0.55, y: () => screen.clientHeight * 1.05 }, { autoAlpha: 1, duration: 0.04 }, 1.24)
          .to(cursor, { x: () => rowTarget().x, y: () => rowTarget().y, duration: 0.18 }, 1.26)
          .to(one("[data-s-row-hl]"), { autoAlpha: 1, duration: 0.05 }, 1.45)
          .to(one("[data-s-empty]"), { autoAlpha: 0, duration: 0.05 }, 1.46)
          .fromTo(one("[data-s-profile]"), { autoAlpha: 0, y: 8 }, { autoAlpha: 1, y: 0, duration: 0.1 }, 1.48)
          .to(cursor, { x: () => btnTarget().x, y: () => btnTarget().y, duration: 0.2 }, 1.56)
          .to(invite, { scale: 0.92, duration: 0.04 }, 1.78)
          .to(invite, { scale: 1, duration: 0.04 }, 1.82)
          .to(invite, { autoAlpha: 0, duration: 0.05 }, 1.86)
          .fromTo(one("[data-s-btn-sent]"), { y: 6 }, { autoAlpha: 1, y: 0, duration: 0.07 }, 1.87)
          .to(cursor, { autoAlpha: 0, duration: 0.06 }, 1.93);

        tl.to(route, { autoAlpha: 1, duration: 0.1 }, 2)
          .to(flight, { autoAlpha: 1, scale: 1, duration: 0.08, ease: "back.out(2)" }, 2.05)
          .to(flight, { motionPath: { path: route }, duration: 0.6, ease: "power1.inOut" }, 2.05)
          .fromTo(trail, { drawSVG: "0% 0%" }, { drawSVG: "0% 100%", duration: 0.6, ease: "power1.inOut" }, 2.05)
          .to(laptop, { scale: 0.98, duration: 0.2 }, 2.35)
          .to(lapDim, { opacity: 0.45, duration: 0.2 }, 2.35)
          .to(phone, { scale: 1, y: 0, duration: 0.2 }, 2.35)
          .to(phoneDim, { opacity: 0, duration: 0.2 }, 2.35)
          .to(flight, { autoAlpha: 0, scale: 0.3, duration: 0.07 }, 2.65)
          .to([route, trail], { autoAlpha: 0, duration: 0.12 }, 2.72);

        tl.to(push, { autoAlpha: 1, yPercent: 0, duration: 0.16, ease: "back.out(1.5)" }, 3)
          .to(phone, { scale: 1.025, duration: 0.12 }, 3.02)
          .to(push, { autoAlpha: 0, scale: 1.06, duration: 0.12 }, 3.46)
          .to(one("[data-s-scr-lock]"), { autoAlpha: 0, duration: 0.12 }, 3.48)
          .fromTo(one("[data-s-scr-inv]"), { scale: 0.94 }, { autoAlpha: 1, scale: 1, duration: 0.15 }, 3.5)
          .to(phone, { scale: 1, duration: 0.12 }, 3.55);

        tl.set(one("[data-s-tap]"), { autoAlpha: 0.8, scale: 0.2 }, 4.04)
          .to(one("[data-s-tap]"), { autoAlpha: 0, scale: 1.8, duration: 0.14, ease: "power2.out" }, 4.04)
          .to(one("[data-s-accept-btn]"), { scale: 0.94, duration: 0.04 }, 4.04)
          .to(one("[data-s-accept-btn]"), { scale: 1, duration: 0.04 }, 4.08)
          .to(one("[data-s-actions]"), { autoAlpha: 0, y: 6, duration: 0.07 }, 4.12)
          .fromTo(one("[data-s-accepted]"), { y: 8 }, { autoAlpha: 1, y: 0, duration: 0.1 }, 4.15)
          .to(laptop, { scale: 1, duration: 0.15 }, 4.2)
          .to(lapDim, { opacity: 0, duration: 0.15 }, 4.2)
          .to(route, { autoAlpha: 1, duration: 0.06 }, 4.22)
          .to(flightBack, { autoAlpha: 1, scale: 1, duration: 0.06 }, 4.24)
          .to(flightBack, { motionPath: { path: route, start: 1, end: 0 }, duration: 0.3, ease: "power1.inOut" }, 4.24)
          .to(phone, { scale: 0.98, y: phoneAside, duration: 0.15 }, 4.4)
          .to(phoneDim, { opacity: 0.45, duration: 0.15 }, 4.4)
          .to(flightBack, { autoAlpha: 0, scale: 0.3, duration: 0.06 }, 4.52)
          .to(route, { autoAlpha: 0, duration: 0.08 }, 4.54)
          .to(one("[data-s-lock]"), { autoAlpha: 0, scale: 0.97, duration: 0.08 }, 4.54)
          .to(openItems, { autoAlpha: 1, y: 0, duration: 0.1, stagger: 0.04 }, 4.58)
          .to(one("[data-s-btn-sent]"), { autoAlpha: 0, duration: 0.05 }, 4.56)
          .fromTo(one("[data-s-btn-accepted]"), { y: 6 }, { autoAlpha: 1, y: 0, duration: 0.07 }, 4.6)
          .set({}, {}, STAGES.length);

        return () => {
          ScrollTrigger.removeEventListener("refreshInit", layout);
          delete el.dataset.live;
        };
      });
    },
    { scope: root, extra: loadMotionPath },
  );

  return (
    <section ref={root} id="scene" aria-labelledby="scene-title" className="group/scene relative overflow-x-clip bg-canvas">
      <div data-s-stage className="relative py-20 sm:py-28 lg:py-32 group-data-live/scene:h-[100svh] group-data-live/scene:overflow-hidden group-data-live/scene:py-0">
        <div aria-hidden className="bg-grid mask-radial pointer-events-none absolute inset-0 opacity-70" />
        <Container
          className={cn(
            "relative grid grid-cols-1 gap-12 lg:grid-cols-[minmax(0,0.8fr)_minmax(0,1.2fr)] lg:items-center lg:gap-16",
            "group-data-live/scene:flex group-data-live/scene:h-full group-data-live/scene:flex-col group-data-live/scene:items-stretch group-data-live/scene:gap-0 group-data-live/scene:pt-20 group-data-live/scene:pb-5 lg:group-data-live/scene:pt-24 lg:group-data-live/scene:pb-8",
          )}
        >
          <div className="min-w-0">
            <div className="flex items-center justify-between gap-6">
              <h2 id="scene-title" className="eyebrow inline-flex items-center gap-2.5 text-primary">
                <span aria-hidden className="size-2 rounded-full bg-primary" />
                Как это выглядит
              </h2>
              <p aria-hidden className="hidden items-center gap-3 font-mono text-xs text-ink-subtle group-data-live/scene:flex">
                <span data-s-count className="tabular text-ink">
                  01
                </span>
                <span>/ 05</span>
              </p>
            </div>
            <div aria-hidden className="mt-4 hidden gap-1.5 group-data-live/scene:flex">
              {STAGES.map((s, i) => (
                <span key={i} className="h-[3px] flex-1 overflow-hidden rounded-full bg-line">
                  <span data-s-seg className={cn("block h-full w-full rounded-full", s.side === "seeker" ? "bg-primary" : "bg-accent")} />
                </span>
              ))}
            </div>

            <ol className="mt-10 space-y-9 group-data-live/scene:mt-5 group-data-live/scene:grid group-data-live/scene:space-y-0 lg:group-data-live/scene:mt-7">
              {STAGES.map((s, i) => (
                <li
                  key={s.title}
                  data-s-cap
                  className="grid grid-cols-[2.25rem_1fr] gap-x-3 group-data-live/scene:block group-data-live/scene:[grid-area:1/1] lg:group-data-live/scene:grid lg:group-data-live/scene:grid-cols-[minmax(0,1fr)_minmax(0,1fr)] lg:group-data-live/scene:items-end lg:group-data-live/scene:gap-x-14"
                >
                  <span aria-hidden className="pt-1 font-mono text-sm text-ink-subtle group-data-live/scene:hidden">
                    {String(i + 1).padStart(2, "0")}
                  </span>
                  <div>
                    <p data-s-cap-line className={cn("font-mono text-xs tracking-[0.14em] uppercase", s.side === "seeker" ? "text-primary-strong" : "text-accent-strong")}>
                      <span className={cn("mr-2 inline-block size-1.5 rounded-full align-middle", s.side === "seeker" ? "bg-primary" : "bg-accent")} aria-hidden />
                      {s.tag}
                    </p>
                    <h3 data-s-cap-line className="mt-2 text-xl font-semibold tracking-tight text-balance text-ink group-data-live/scene:text-[clamp(1.45rem,1.05rem+1.6vw,2.6rem)] group-data-live/scene:leading-[1.05] group-data-live/scene:tracking-[-0.035em]">
                      {s.title}
                    </h3>
                  </div>
                  <p
                    data-s-cap-line
                    className="col-start-2 mt-2 max-w-xl text-[15px] leading-relaxed text-ink-muted group-data-live/scene:mt-2.5 group-data-live/scene:text-[14px] group-data-live/scene:leading-snug sm:group-data-live/scene:text-base lg:group-data-live/scene:mt-0 lg:group-data-live/scene:text-[17px] lg:group-data-live/scene:leading-relaxed"
                  >
                    {s.text}
                  </p>
                </li>
              ))}
            </ol>
          </div>

          <div
            data-s-devices
            className={cn(
              "relative flex flex-col lg:flex-row lg:items-end lg:justify-between",
              "group-data-live/scene:mt-5 group-data-live/scene:min-h-0 group-data-live/scene:flex-1 group-data-live/scene:[container-type:size] lg:group-data-live/scene:mt-8",
            )}
          >
            {laptop}
            {phone}
          </div>
        </Container>

        <svg data-s-overlay aria-hidden className="pointer-events-none absolute inset-0 hidden size-full overflow-visible group-data-live/scene:block" fill="none">
          <path data-s-route className="stroke-primary-300" strokeWidth="1.5" strokeDasharray="2 7" strokeLinecap="round" vectorEffect="non-scaling-stroke" />
          <path data-s-trail className="stroke-primary" strokeWidth="2" strokeLinecap="round" vectorEffect="non-scaling-stroke" />
        </svg>
        <div aria-hidden data-s-flight className="pointer-events-none absolute top-0 left-0 z-20 hidden items-center gap-2 rounded-full bg-white py-1.5 pr-3 pl-1.5 text-xs font-medium whitespace-nowrap text-ink shadow-pop ring-1 ring-primary-200 group-data-live/scene:flex sm:py-2 sm:pr-4 sm:pl-2 sm:text-sm">
          <span className="flex size-6 items-center justify-center rounded-full bg-primary text-white sm:size-7">
            <Mail className="size-3.5" />
          </span>
          <span className="font-mono">280–350 тыс. ₽</span>
        </div>
        <div aria-hidden data-s-flight-back className="pointer-events-none absolute top-0 left-0 z-20 hidden items-center gap-2 rounded-full bg-success py-1.5 pr-3 pl-1.5 text-xs font-medium whitespace-nowrap text-white shadow-pop group-data-live/scene:flex sm:py-2 sm:pr-4 sm:pl-2 sm:text-sm">
          <span className="flex size-6 items-center justify-center rounded-full bg-white text-success sm:size-7">
            <Check className="size-3.5" strokeWidth={3} />
          </span>
          Принято
        </div>
      </div>
    </section>
  );
}
