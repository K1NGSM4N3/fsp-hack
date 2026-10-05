"use client";

import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

const CARDS = [
  { role: "Платформенная команда, Kafka", meta: "Телеком · офис, Казань", salary: "260–320 тыс. ₽", status: "Просмотрено" },
  { role: "Backend для склада и заказов", meta: "E-commerce · удалённо", salary: "300–380 тыс. ₽", status: "Просмотрено" },
  { role: "Go-разработчик в команду платежей", meta: "Финтех · гибрид, Москва", salary: "280–350 тыс. ₽", status: "Новое" },
];

const FAN = [
  { xPercent: -19, y: 34, rotation: -9 },
  { xPercent: 19, y: 34, rotation: 9 },
  { xPercent: 0, y: 0, rotation: 0 },
];

/** A hand of invitation cards dealt into a fan, then floating idly; each card lifts on hover. */
export function InviteDeck() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const cards = gsap.utils.toArray<HTMLElement>("[data-card]", root.current);
        gsap.set(root.current, { perspective: 1200 });
        const deal = gsap.timeline({ delay: 0.5 });
        cards.forEach((card, i) => {
          const fan = FAN[i] ?? FAN[0];
          deal.fromTo(
            card,
            { xPercent: 0, y: 220, rotation: 0, rotationX: 65, rotationY: -25, z: -200, autoAlpha: 0 },
            { ...fan, rotationX: 0, rotationY: 0, z: 0, autoAlpha: 1, duration: 1.1, ease: "expo.out" },
            i * 0.14,
          );
        });
        deal.add(() => {
          cards.forEach((card, i) => {
            gsap.to(card, { y: `+=${6 + i * 2}`, rotation: `+=${i % 2 ? 1.2 : -1.2}`, duration: 2.2 + i * 0.35, ease: "sine.inOut", yoyo: true, repeat: -1 });
          });
        });
        const off = cards.map((card, i) => {
          const fan = FAN[i] ?? FAN[0];
          const enter = () => gsap.to(card, { y: fan.y - 34, scale: 1.03, duration: 0.45, ease: "power3.out", overwrite: "auto" });
          const leave = () => gsap.to(card, { y: fan.y, scale: 1, duration: 0.6, ease: "power3.out", overwrite: "auto" });
          card.addEventListener("pointerenter", enter);
          card.addEventListener("pointerleave", leave);
          return () => {
            card.removeEventListener("pointerenter", enter);
            card.removeEventListener("pointerleave", leave);
          };
        });
        return () => off.forEach((f) => f());
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="relative mx-auto h-[24rem] w-full max-w-[27rem] sm:h-[26rem]" aria-label="Приглашения с зарплатой" role="img">
      {CARDS.map((c, i) => {
        const fan = FAN[i] ?? FAN[0];
        const front = i === CARDS.length - 1;
        return (
          <div
            key={c.role}
            data-card
            className={cn(
              "absolute top-8 left-1/2 -ml-[36%] w-[72%] origin-bottom rounded-[22px] border p-4 shadow-pop will-change-transform sm:p-5",
              front ? "border-line bg-white" : "border-line bg-white/95",
            )}
            style={{ transform: `translate(${fan.xPercent}%, ${fan.y}px) rotate(${fan.rotation}deg)` }}
          >
            <div className="flex items-center justify-between gap-2 text-[11px]">
              <span className={cn("rounded-full px-2 py-0.5 font-medium", c.status === "Новое" ? "bg-primary text-white" : "bg-canvas text-ink-subtle")}>{c.status}</span>
              <span className="font-mono text-ink-faint">14 дней</span>
            </div>
            <p className="mt-3 text-[15px] leading-snug font-semibold tracking-tight text-ink">{c.role}</p>
            <p className="mt-1 text-xs text-ink-muted">{c.meta}</p>
            <p className="tabular mt-4 rounded-xl bg-canvas px-3 py-2 font-mono text-base font-semibold text-ink">{c.salary}</p>
            {front && (
              <div className="mt-4 grid grid-cols-2 gap-2 text-xs font-medium">
                <span className="inline-flex h-9 items-center justify-center rounded-lg bg-primary text-white">Принять</span>
                <span className="inline-flex h-9 items-center justify-center rounded-lg border border-line-strong text-ink">Отклонить</span>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
