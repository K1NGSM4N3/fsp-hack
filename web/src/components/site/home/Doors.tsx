"use client";

import { ArrowRight, ArrowUpRight, Check, Clock } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { useRef } from "react";
import { MOTION_OK, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";
import { Container } from "../primitives";

const DOORS_WIDE = "(prefers-reduced-motion: no-preference) and (min-width: 1024px)";

const OFFERS = [
  { role: "Go-разработчик", pay: "280 000 – 350 000 ₽", meta: "Финтех · гибрид" },
  { role: "Фронтенд-разработчик", pay: "230 000 – 290 000 ₽", meta: "Ритейл · удалённо" },
  { role: "Инженер по тестированию", pay: "170 000 – 220 000 ₽", meta: "Логистика · офис" },
  { role: "DevOps-инженер", pay: "300 000 – 380 000 ₽", meta: "Телеком · удалённо" },
];

const STEP = 84;

const PEOPLE = [
  { code: "К-5F8A", name: "Анна Соколова", cat: "Бэкенд · Middle" },
  { code: "К-91C0", name: "Илья Мартынов", cat: "Бэкенд · Senior" },
  { code: "К-3D27", name: "Дарья Белова", cat: "Бэкенд · Middle" },
];

function OfferStack() {
  return (
    <div className="relative h-[15.5rem] w-full max-w-[22rem]" aria-hidden>
      {OFFERS.map((o, i) => (
        <div
          key={o.role}
          data-offer
          className="absolute inset-x-0 top-0 h-[4.5rem] rounded-2xl border border-white/20 bg-white/12 px-4 py-3 backdrop-blur-sm"
          style={{ transform: `translateY(${i * STEP}px) scale(${1 - i * 0.05})`, opacity: i === 3 ? 0 : 1 - i * 0.3, transformOrigin: "50% 0%" }}
        >
          <div className="flex items-center justify-between gap-3 text-xs text-primary-100">
            <span className="truncate">{o.meta}</span>
            <span className="shrink-0">сейчас</span>
          </div>
          <p className="mt-1 flex items-baseline justify-between gap-3">
            <span className="truncate text-[15px] font-semibold">{o.role}</span>
            <span className="shrink-0 font-mono text-[13px] text-white">{o.pay}</span>
          </p>
        </div>
      ))}
    </div>
  );
}

function PeopleList() {
  return (
    <ul className="w-full max-w-[24rem] divide-y divide-line overflow-hidden rounded-2xl border border-line bg-white shadow-card" aria-hidden>
      {PEOPLE.map((p, i) => (
        <li key={p.code} data-person className="relative flex items-center gap-3 px-4 py-3">
          <span data-person-hl className={cn("absolute inset-0 bg-primary-50", i === 0 ? "opacity-100" : "opacity-0")} />
          <span className="relative flex size-9 shrink-0 items-center justify-center rounded-full bg-surface text-xs font-semibold text-ink-muted">К</span>
          <span className="relative min-w-0 flex-1">
            <span className="relative block h-5 text-sm font-semibold">
              <span data-person-code className={cn("absolute inset-0 truncate", i === 0 && "opacity-0")}>
                Кандидат {p.code}
              </span>
              <span data-person-name className={cn("absolute inset-0 truncate text-primary-strong", i !== 0 && "opacity-0")}>
                {p.name}
              </span>
            </span>
            <span className="block text-xs text-ink-subtle">{p.cat}</span>
          </span>
          <span className="relative h-8 w-[7.5rem] shrink-0 text-xs font-medium">
            <span data-pill="invite" className={cn("absolute inset-0 flex items-center justify-center rounded-full bg-primary text-white", i !== 2 && "opacity-0")}>
              Пригласить
            </span>
            <span data-pill="sent" className={cn("absolute inset-0 flex items-center justify-center gap-1 rounded-full border border-line-strong text-ink-muted", i !== 1 && "opacity-0")}>
              <Clock className="size-3.5" /> Ждём ответа
            </span>
            <span data-pill="ok" className={cn("absolute inset-0 flex items-center justify-center gap-1 rounded-full bg-success-50 text-success", i !== 0 && "opacity-0")}>
              <Check className="size-3.5" strokeWidth={3} /> Принято
            </span>
          </span>
        </li>
      ))}
    </ul>
  );
}

function offersLoop(scope: HTMLElement) {
  const items = gsap.utils.toArray<HTMLElement>("[data-offer]", scope);
  const n = items.length;
  const tl = gsap.timeline({ repeat: -1, paused: true, defaults: { duration: 0.8, ease: "expo.out" } });
  for (let k = 1; k <= n; k++) {
    const at = (k - 1) * 2.1 + 1;
    items.forEach((item, j) => {
      const slot = (j + k) % n;
      if (slot === 0) tl.fromTo(item, { y: -STEP * 0.6, scale: 1.04, opacity: 0 }, { y: 0, scale: 1, opacity: 1, immediateRender: false }, at);
      else tl.to(item, { y: slot * STEP, scale: 1 - slot * 0.05, opacity: slot === n - 1 ? 0 : 1 - slot * 0.3 }, at);
    });
  }
  return tl;
}

function peopleLoop(scope: HTMLElement) {
  const rows = gsap.utils.toArray<HTMLElement>("[data-person]", scope);
  const pick = (row: HTMLElement, sel: string) => row.querySelector(sel);
  const tl = gsap.timeline({ repeat: -1, paused: true, repeatDelay: 0.6, defaults: { duration: 0.35, ease: "power2.out" } });
  tl.set(gsap.utils.toArray("[data-person-hl], [data-person-name], [data-pill='sent'], [data-pill='ok']", scope), { opacity: 0 }, 0)
    .set(gsap.utils.toArray("[data-person-code], [data-pill='invite']", scope), { opacity: 1 }, 0);
  rows.forEach((row, i) => {
    const at = 0.4 + i * 2.2;
    tl.to(pick(row, "[data-person-hl]"), { opacity: 1 }, at)
      .to(pick(row, "[data-pill='invite']"), { scale: 0.9, duration: 0.1, yoyo: true, repeat: 1 }, at + 0.3)
      .to(pick(row, "[data-pill='invite']"), { opacity: 0 }, at + 0.5)
      .to(pick(row, "[data-pill='sent']"), { opacity: 1 }, at + 0.55)
      .to(pick(row, "[data-pill='sent']"), { opacity: 0 }, at + 1.4)
      .fromTo(pick(row, "[data-pill='ok']"), { opacity: 0, scale: 0.8 }, { opacity: 1, scale: 1, ease: "back.out(2)", immediateRender: false }, at + 1.45)
      .to(pick(row, "[data-person-code]"), { opacity: 0, y: -8 }, at + 1.6)
      .fromTo(pick(row, "[data-person-name]"), { opacity: 0, y: 8 }, { opacity: 1, y: 0, immediateRender: false }, at + 1.65)
      .to(pick(row, "[data-person-hl]"), { opacity: 0 }, at + 2.1);
  });
  tl.to(gsap.utils.toArray("[data-person-name], [data-pill='ok']", scope), { opacity: 0, duration: 0.4 }, "+=1.4").to(
    gsap.utils.toArray("[data-person-code], [data-pill='invite']", scope),
    { opacity: 1, y: 0, duration: 0.4 },
    "<0.1",
  );
  return tl;
}

/** Final chooser: two full-bleed halves; on wide screens the hovered or focused half widens. */
export function Doors() {
  const root = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const el = root.current;
      if (!el) return;
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const seeker = el.querySelector<HTMLElement>("[data-door='seeker']");
        const business = el.querySelector<HTMLElement>("[data-door='business']");
        if (!seeker || !business) return;
        for (const [door, loop] of [
          [seeker, offersLoop(seeker)],
          [business, peopleLoop(business)],
        ] as const) {
          ScrollTrigger.create({ trigger: door, start: "top bottom", end: "bottom top", onToggle: (self) => (self.isActive ? loop.play() : loop.pause()) });
        }
      });
      mm.add(DOORS_WIDE, (context) => {
        const doors = gsap.utils.toArray<HTMLElement>("[data-door]", el);
        const grow = context.add("grow", (active: HTMLElement | null) => {
          for (const door of doors) {
            const on = door === active;
            gsap.to(door, { flexGrow: active ? (on ? 1.4 : 1) : 1, duration: 1, ease: "expo.out", overwrite: "auto" });
            gsap.to(door.querySelectorAll("[data-door-fade]"), { opacity: active && !on ? 0.55 : 1, duration: 0.6, overwrite: "auto" });
            gsap.to(door.querySelector("[data-door-arrow]"), { rotate: on ? 45 : 0, scale: on ? 1.12 : 1, duration: 0.6, ease: "back.out(2)", overwrite: "auto" });
            gsap.to(door.querySelector("[data-door-visual]"), { scale: on ? 1.06 : 1, duration: 1, ease: "expo.out", overwrite: "auto" });
          }
        }) as (active: HTMLElement | null) => void;
        const off: Array<() => void> = [];
        const reset = () => grow(null);
        for (const door of doors) {
          const enter = () => grow(door);
          door.addEventListener("pointerenter", enter);
          door.addEventListener("focus", enter);
          door.addEventListener("blur", reset);
          off.push(() => {
            door.removeEventListener("pointerenter", enter);
            door.removeEventListener("focus", enter);
            door.removeEventListener("blur", reset);
          });
        }
        const row = doors[0]?.parentElement;
        row?.addEventListener("pointerleave", reset);
        return () => {
          off.forEach((f) => f());
          row?.removeEventListener("pointerleave", reset);
        };
      });
    },
    { scope: root },
  );

  return (
    <section ref={root} aria-labelledby="doors-title" className="relative isolate">
      <h2 id="doors-title" className="sr-only">
        С чего начать
      </h2>
      <div className="flex flex-col lg:h-[min(92svh,880px)] lg:min-h-[640px] lg:flex-row">
        <Link
          href="/seekers"
          data-door="seeker"
          className="group relative flex min-h-[38rem] flex-1 basis-0 flex-col overflow-hidden bg-primary px-5 pt-10 pb-10 text-white outline-offset-[-6px] focus-visible:outline-white sm:px-10 lg:min-h-0 lg:px-14 lg:pt-28 lg:pb-14"
        >
          <span aria-hidden className="pointer-events-none absolute -top-40 -right-40 size-[34rem] rounded-full bg-primary-400/60 blur-[120px]" />
          <span data-door-fade className="relative flex items-center justify-between gap-4">
            <span className="font-mono text-xs tracking-[0.14em] text-primary-100 uppercase">Соискателям</span>
            <span data-door-arrow className="flex size-12 items-center justify-center rounded-full bg-white text-primary">
              <ArrowUpRight className="size-5" aria-hidden />
            </span>
          </span>
          <span data-door-visual className="relative my-10 flex flex-1 items-center justify-center lg:my-8">
            <OfferStack />
          </span>
          <span data-door-fade className="relative block">
            <span className="block text-[clamp(2.6rem,1.1rem+4vw,4.6rem)] leading-[0.95] font-semibold tracking-[-0.05em]">
              Ищу
              <br />
              работу
            </span>
            <span className="mt-5 block max-w-md text-base leading-relaxed text-primary-100 sm:text-lg">
              Пройдите тест один раз. Дальше компании пишут вам сами и сразу называют сумму.
            </span>
          </span>
        </Link>

        <Link
          href="/business"
          data-door="business"
          className="group relative flex min-h-[38rem] flex-1 basis-0 flex-col overflow-hidden border-t border-line bg-white px-5 pt-10 pb-10 text-ink outline-offset-[-6px] sm:px-10 lg:min-h-0 lg:border-t-0 lg:border-l lg:px-14 lg:pt-28 lg:pb-14"
        >
          <span aria-hidden className="bg-grid mask-radial pointer-events-none absolute inset-0 opacity-80" />
          <span data-door-fade className="relative flex items-center justify-between gap-4">
            <span className="font-mono text-xs tracking-[0.14em] text-ink-subtle uppercase">Компаниям</span>
            <span data-door-arrow className="flex size-12 items-center justify-center rounded-full bg-ink text-white">
              <ArrowUpRight className="size-5" aria-hidden />
            </span>
          </span>
          <span data-door-visual className="relative my-10 flex flex-1 items-center justify-center lg:my-8">
            <PeopleList />
          </span>
          <span data-door-fade className="relative block">
            <span className="block text-[clamp(2.6rem,1.1rem+4vw,4.6rem)] leading-[0.95] font-semibold tracking-[-0.05em]">
              Ищу
              <br />
              специалистов
            </span>
            <span className="mt-5 block max-w-md text-base leading-relaxed text-ink-muted sm:text-lg">
              В подборке только люди с проверенным уровнем. Пишете тем, кто подходит, и сразу называете вилку.
            </span>
          </span>
        </Link>
      </div>

      <Link href="/fsp" className="group block border-t border-line bg-canvas">
        <Container className="flex items-center gap-4 py-5 sm:gap-6">
          <Image src="/brand/fsp-mark.svg" alt="" width={28} height={28} unoptimized className="size-7 shrink-0" />
          <span className="min-w-0 flex-1 text-[15px] leading-snug text-ink-muted sm:text-base">
            <span className="font-semibold text-ink">Участникам соревнований ФСП.</span> Разряды и призовые места попадают в профиль через ФСП ID и поднимают вас в подборке.
          </span>
          <span className="inline-flex shrink-0 items-center gap-2 text-sm font-medium text-ink transition-colors group-hover:text-primary">
            <span className="hidden sm:inline">Подробнее</span>
            <ArrowRight className="size-4 transition-transform duration-300 group-hover:translate-x-1" aria-hidden />
          </span>
        </Container>
      </Link>
    </section>
  );
}
