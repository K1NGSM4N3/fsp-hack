"use client";

import { BadgeCheck, Camera, Flashlight, Lock } from "lucide-react";
import Image from "next/image";
import { useRef } from "react";
import { MOTION_OK, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";

interface Push {
  company: string;
  role: string;
  salary: string;
}

const PUSHES: Push[] = [
  { company: "Кедр Платежи", role: "Бэкенд в платёжную команду", salary: "280–350 тыс. ₽" },
  { company: "Сота Логистика", role: "Go-разработчик, сервис доставки", salary: "250–310 тыс. ₽" },
  { company: "Маяк Медиа", role: "Python, рекомендации", salary: "230–290 тыс. ₽" },
  { company: "Тундра Облако", role: "Бэкенд, биллинг", salary: "270–330 тыс. ₽" },
  { company: "Ладья Маркет", role: "Бэкенд, каталог и поиск", salary: "260–320 тыс. ₽" },
  { company: "Гранит Аналитика", role: "Бэкенд, отчёты для банков", salary: "290–360 тыс. ₽" },
];

const SLOT = 102;
const VISIBLE = 3;

function whenLabel(slot: number): string {
  return slot === 0 ? "сейчас" : `${slot * 4} мин назад`;
}

function Notification({ push, index }: { push: Push; index: number }) {
  const shown = index < VISIBLE;
  return (
    <li
      data-push
      className="absolute inset-x-0 top-0 h-[94px] origin-top will-change-transform"
      style={shown ? { transform: `translateY(${index * SLOT}px)` } : { opacity: 0, visibility: "hidden" }}
    >
      <div className="flex h-full items-start gap-3 rounded-[22px] bg-white/82 p-3 shadow-[0_10px_30px_-12px_rgb(11_11_18/0.55)] backdrop-blur-xl backdrop-saturate-150">
        <span className="flex size-9 shrink-0 items-center justify-center rounded-[11px] bg-white shadow-[0_1px_2px_rgb(11_11_18/0.12)]">
          <Image src="/brand/fsp-mark.svg" alt="" width={22} height={22} unoptimized className="size-[22px]" />
        </span>
        <span className="min-w-0 flex-1">
          <span className="flex items-center justify-between gap-2 text-[10.5px] leading-4 text-ink-subtle">
            <span className="font-medium tracking-wide uppercase">Приглашение</span>
            <span data-when>{whenLabel(index)}</span>
          </span>
          <span className="mt-0.5 block truncate text-[14px] leading-[19px] font-semibold text-ink">{push.company}</span>
          <span className="block truncate text-[12.5px] leading-[17px] text-ink-muted">{push.role}</span>
          <span className="tabular mt-0.5 block text-[13px] leading-[18px] font-semibold text-primary-strong">{push.salary}</span>
        </span>
      </div>
    </li>
  );
}

/** Lock-screen mock: invitations drop in with a spring and push the older ones down; the radar rings pulse on every arrival. */
export function PushPhone() {
  const root = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const items = gsap.utils.toArray<HTMLElement>("[data-push]", root.current);
        const rings = gsap.utils.toArray<HTMLElement>("[data-ring]", root.current);
        const floats = gsap.utils.toArray<HTMLElement>("[data-float]", root.current);
        gsap.set(items, { autoAlpha: 0, y: -SLOT * 0.5, scale: 0.9 });
        gsap.set(rings, { autoAlpha: 0, scale: 0.55 });

        const shown: HTMLElement[] = [];
        let next = 0;
        let onScreen = true;

        const drop = () => {
          const el = items[next % items.length];
          if (!el) return;
          next += 1;
          const was = shown.indexOf(el);
          if (was >= 0) shown.splice(was, 1);
          shown.unshift(el);
          gsap.set(el, { zIndex: next, rotationX: -40, transformPerspective: 600 });
          gsap.fromTo(
            el,
            { y: -SLOT * 0.4, scale: 0.88, autoAlpha: 0 },
            { y: 0, scale: 1, rotationX: 0, autoAlpha: 1, duration: 1.15, ease: "elastic.out(1, 0.6)", overwrite: true },
          );
          shown.forEach((item, slot) => {
            const when = item.querySelector("[data-when]");
            if (when) when.textContent = whenLabel(slot);
          });
          shown.slice(1).forEach((older, i) => {
            const slot = i + 1;
            if (slot < VISIBLE) {
              gsap.to(older, { y: slot * SLOT, scale: 1, autoAlpha: 1, duration: 0.75, ease: "power3.out", delay: 0.04 * slot, overwrite: true });
            } else {
              gsap.to(older, { y: slot * SLOT - 18, scale: 0.92, autoAlpha: 0, duration: 0.5, ease: "power2.in", overwrite: true });
            }
          });
          if (shown.length > VISIBLE + 1) shown.length = VISIBLE + 1;
          gsap.fromTo(rings, { autoAlpha: 0.5, scale: 0.55 }, { autoAlpha: 0, scale: 1.35, duration: 2.2, ease: "power2.out", stagger: 0.22, overwrite: true });
        };

        const loop = () => {
          gsap.delayedCall(2.9, () => {
            if (onScreen) drop();
            loop();
          });
        };

        const intro = gsap.timeline({ delay: 0.9 });
        intro.call(drop).call(drop, [], "+=0.55").call(drop, [], "+=0.55").call(loop);

        ScrollTrigger.create({
          trigger: root.current,
          start: "top bottom",
          end: "bottom top",
          onToggle: (self) => {
            onScreen = self.isActive;
          },
        });

        floats.forEach((el, i) => {
          gsap.fromTo(el, { autoAlpha: 0, y: 18, scale: 0.9 }, { autoAlpha: 1, y: 0, scale: 1, duration: 1, ease: "back.out(1.8)", delay: 1.6 + i * 0.25 });
          gsap.to(el, { y: i % 2 ? 8 : -8, duration: 2.6 + i * 0.4, ease: "sine.inOut", yoyo: true, repeat: -1, delay: 2.6 + i * 0.25 });
        });
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="relative mx-auto w-[296px]" role="img" aria-label="Экран телефона: уведомления о приглашениях от компаний, в каждом указана зарплата от и до">
      <div aria-hidden className="pointer-events-none absolute top-1/2 left-1/2 -z-10 size-[620px] -translate-x-1/2 -translate-y-1/2">
        {[0, 1, 2].map((i) => (
          <span key={i} data-ring className="absolute inset-0 rounded-full border border-primary/25 opacity-0" />
        ))}
        <span className="absolute inset-[18%] rounded-full bg-[radial-gradient(circle,rgb(64_47_255/0.28),transparent_68%)]" />
      </div>

      <div className="relative rounded-[54px] bg-night p-[9px] shadow-[0_50px_90px_-40px_rgb(29_20_145/0.75),0_24px_40px_-24px_rgb(11_11_18/0.5),inset_0_0_0_1.5px_rgb(255_255_255/0.08)]">
        <span aria-hidden className="absolute top-28 -left-[3px] h-14 w-[3px] rounded-l-full bg-night-soft" />
        <span aria-hidden className="absolute top-44 -left-[3px] h-14 w-[3px] rounded-l-full bg-night-soft" />
        <span aria-hidden className="absolute top-36 -right-[3px] h-20 w-[3px] rounded-r-full bg-night-soft" />
        <div
          aria-hidden
          className="relative isolate h-[590px] overflow-hidden rounded-[46px] bg-primary-900 bg-[radial-gradient(120%_70%_at_10%_0%,var(--color-primary-400)_0%,transparent_58%),radial-gradient(90%_55%_at_100%_100%,color-mix(in_srgb,var(--color-accent)_55%,transparent)_0%,transparent_62%),linear-gradient(170deg,var(--color-primary)_0%,var(--color-primary-800)_55%,var(--color-primary-900)_100%)]"
        >
          <svg className="absolute inset-0 h-full w-full text-white/10" viewBox="0 0 278 590" fill="none">
            <path d="M-10 410 H90 a22 22 0 0 0 22 -22 V300 a22 22 0 0 1 22 -22 H300" stroke="currentColor" />
            <path d="M40 600 V520 a22 22 0 0 1 22 -22 H210 a22 22 0 0 0 22 -22 V430" stroke="currentColor" />
            <circle cx="232" cy="430" r="3" fill="currentColor" />
          </svg>

          <div className="relative flex h-11 items-center justify-between px-7 pt-2 text-[13px] font-semibold text-white">
            <span>10:42</span>
            <span className="flex items-center gap-1.5">
              <svg width="17" height="11" viewBox="0 0 17 11" fill="currentColor">
                <rect x="0" y="7" width="3" height="4" rx="1" />
                <rect x="4.5" y="5" width="3" height="6" rx="1" />
                <rect x="9" y="2.5" width="3" height="8.5" rx="1" />
                <rect x="13.5" y="0" width="3" height="11" rx="1" />
              </svg>
              <svg width="25" height="12" viewBox="0 0 25 12" fill="none">
                <rect x="0.5" y="0.5" width="21" height="11" rx="3.5" stroke="currentColor" opacity="0.5" />
                <rect x="2" y="2" width="15" height="8" rx="2" fill="currentColor" />
                <rect x="22.5" y="4" width="2" height="4" rx="1" fill="currentColor" opacity="0.5" />
              </svg>
            </span>
          </div>
          <span className="absolute top-3 left-1/2 h-[30px] w-[98px] -translate-x-1/2 rounded-full bg-black" />

          <div className="relative mt-3 text-center text-white">
            <p className="text-[14px] font-medium text-white/80">Четверг, 8 октября</p>
            <p className="font-display text-[68px] leading-[1.05] font-semibold tracking-[-0.04em]">10:42</p>
          </div>

          <ol className="absolute inset-x-3 top-[166px]" style={{ height: SLOT * VISIBLE }}>
            {PUSHES.map((p, i) => (
              <Notification key={p.company} push={p} index={i} />
            ))}
          </ol>

          <div className="absolute inset-x-0 bottom-0 flex items-end justify-between px-8 pb-7 text-white">
            <span className="flex size-11 items-center justify-center rounded-full bg-white/15 backdrop-blur-md">
              <Flashlight className="size-5" />
            </span>
            <span className="flex size-11 items-center justify-center rounded-full bg-white/15 backdrop-blur-md">
              <Camera className="size-5" />
            </span>
          </div>
          <span className="absolute bottom-2.5 left-1/2 h-[5px] w-32 -translate-x-1/2 rounded-full bg-white/85" />
        </div>
      </div>

      <div
        data-float
        aria-hidden
        className="absolute top-[15%] right-[calc(100%-34px)] hidden items-center gap-2 rounded-2xl border border-line bg-white px-3.5 py-2.5 whitespace-nowrap shadow-pop sm:flex"
      >
        <BadgeCheck className="size-5 text-primary" />
        <span className="leading-tight">
          <span className="block text-[11px] text-ink-subtle">Ваша категория</span>
          <span className="block text-sm font-semibold text-ink">Бэкенд · Middle</span>
        </span>
      </div>
      <div
        data-float
        aria-hidden
        className="absolute top-[85%] right-2 flex items-center gap-2 rounded-2xl bg-ink px-3.5 py-2.5 whitespace-nowrap text-white shadow-pop sm:right-auto sm:left-[calc(100%-34px)]"
      >
        <Lock className="size-4 text-primary-300" />
        <span className="text-[13px] leading-tight font-medium">
          Имя и контакты
          <br />
          <span className="text-night-muted">скрыты до «Принять»</span>
        </span>
      </div>
    </div>
  );
}
