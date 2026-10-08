"use client";

import { useRef } from "react";
import { useMotion } from "@/components/motion/lazy";

const RULES = [
  { label: "Зарплатная вилка", value: "обязательна", hint: "специалист видит деньги до разговора" },
  { label: "Приглашений в неделю", value: "30", hint: "вы пишете точечно тем, кто подходит" },
  { label: "Активных приглашений на одного специалиста", value: "1", hint: "никаких повторных писем одному человеку" },
  { label: "Срок ответа", value: "14 дней", hint: "потом приглашение закрывается само" },
  { label: "Контакты специалиста", value: "после согласия", hint: "имя и способы связи открываются после «да»" },
  { label: "Жалобы на приглашения", value: "разбираются", hint: "специалист может пожаловаться на некорректное письмо" },
];

const WIDTHS = [3, 1, 2, 1, 3, 2, 1, 1, 3, 1, 2, 3, 1, 2, 1, 3, 1, 1, 2, 3, 1, 2, 1, 1, 3, 2, 1, 3, 1, 2, 2, 1, 3, 1];
const BARS = WIDTHS.map((w, i) => ({ w, x: WIDTHS.slice(0, i).reduce((sum, v) => sum + v + 1, 0), ink: i % 2 === 0 }));
const BARCODE_WIDTH = WIDTHS.reduce((sum, v) => sum + v + 1, 0);

function Barcode() {
  return (
    <svg viewBox={`0 0 ${BARCODE_WIDTH} 36`} className="h-9 w-full fill-ink" preserveAspectRatio="none" aria-hidden>
      {BARS.filter((b) => b.ink).map((b) => (
        <rect key={b.x} x={b.x} y={0} width={b.w} height={36} />
      ))}
    </svg>
  );
}

/**
 * The rules come out of a printer; the paper advances in small steps like a thermal printer.
 * Desktop ties the paper to the scroll; phones print the whole receipt once, in about two seconds, as it comes into view.
 */
export function RulesReceipt() {
  const root = useRef<HTMLDivElement>(null);

  useMotion(
    ({ MOTION_DESKTOP, MOTION_MOBILE, gsap }) => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_MOBILE, () => {
        const el = root.current;
        const clip = el?.querySelector<HTMLElement>("[data-clip]");
        const paper = el?.querySelector<HTMLElement>("[data-paper]");
        if (!clip || !paper) return;
        const led = el?.querySelector<HTMLElement>("[data-led]");
        const blink = led ? gsap.to(led, { opacity: 0.25, duration: 0.18, repeat: -1, yoyo: true, ease: "steps(1)", paused: true }) : null;
        gsap
          .timeline({ scrollTrigger: { trigger: clip, start: "top 75%", once: true }, onStart: () => void blink?.play(), onComplete: () => void blink?.pause(0) })
          .fromTo(paper, { yPercent: -100 }, { yPercent: 0, ease: "steps(24)", duration: 2 })
          .fromTo(el?.querySelector("[data-stamp]") ?? [], { autoAlpha: 0, scale: 1.9, rotation: -26 }, { autoAlpha: 1, scale: 1, rotation: -12, ease: "back.out(2)", duration: 0.45 });
      });
      mm.add(MOTION_DESKTOP, () => {
        const el = root.current;
        const clip = el?.querySelector<HTMLElement>("[data-clip]");
        const paper = el?.querySelector<HTMLElement>("[data-paper]");
        const stamp = el?.querySelector<HTMLElement>("[data-stamp]");
        const led = el?.querySelector<HTMLElement>("[data-led]");
        if (!clip || !paper) return;
        const blink = led ? gsap.to(led, { opacity: 0.25, duration: 0.18, repeat: -1, yoyo: true, ease: "steps(1)", paused: true }) : null;
        const tl = gsap.timeline({
          scrollTrigger: {
            trigger: clip,
            start: "top 72%",
            end: () => `+=${paper.offsetHeight}`,
            scrub: 0.3,
            invalidateOnRefresh: true,
            onToggle: (self) => {
              if (!blink) return;
              if (self.isActive) blink.play();
              else blink.pause(0);
            },
          },
        });
        tl.fromTo(paper, { yPercent: -100 }, { yPercent: 0, ease: "steps(36)", duration: 1 });
        if (stamp) tl.fromTo(stamp, { autoAlpha: 0, scale: 1.9, rotation: -26 }, { autoAlpha: 1, scale: 1, rotation: -12, ease: "back.out(2)", duration: 0.12 }, 0.9);
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="relative mx-auto w-full max-w-[26rem]">
      <div className="relative z-10 h-[4.5rem] rounded-[20px] bg-night shadow-[0_24px_50px_-24px_rgb(11_11_18/0.6)]">
        <span className="absolute top-4 left-6 font-mono text-xs tracking-[0.16em] text-night-faint uppercase">ФСП Таланты · правила</span>
        <span data-led aria-hidden className="absolute top-[18px] right-6 size-2 rounded-full bg-success-300" />
        <span aria-hidden className="absolute inset-x-5 bottom-3.5 h-2 rounded-full bg-black shadow-[inset_0_2px_4px_rgb(0_0_0/0.8)]" />
      </div>
      <div data-clip className="relative mx-4 -mt-[18px] overflow-hidden sm:mx-5">
        <div data-paper className="relative">
          <div className="bg-white px-5 pt-9 pb-6 font-mono text-ink shadow-[inset_0_14px_14px_-12px_rgb(11_11_18/0.25)] sm:px-6">
            <p className="text-center text-sm font-semibold tracking-[0.18em] uppercase">Правила контакта</p>
            <p className="mt-1 text-center text-xs text-ink-subtle">для всех компаний на платформе</p>
            <div aria-hidden className="mt-5 border-t border-dashed border-line-strong" />
            <ol className="mt-4 space-y-3.5">
              {RULES.map((r, i) => (
                <li key={r.label}>
                  <div className="flex items-end gap-2 text-[13px] leading-snug">
                    <span className="text-ink-faint">0{i + 1}</span>
                    <span className="min-w-0">{r.label}</span>
                    <span aria-hidden className="mb-1 min-w-4 flex-1 border-b border-dotted border-ink-faint" />
                    <span className="max-w-[45%] text-right font-semibold sm:max-w-none sm:whitespace-nowrap">{r.value}</span>
                  </div>
                  <p className="mt-0.5 pl-6 text-xs leading-snug text-ink-subtle">{r.hint}</p>
                </li>
              ))}
            </ol>
            <div aria-hidden className="mt-5 border-t border-dashed border-line-strong" />
            <div className="mt-5 flex items-center gap-4">
              <div className="min-w-0 flex-1">
                <p className="text-[13px] font-semibold tracking-[0.14em] uppercase">Исключений нет</p>
                <div className="mt-3">
                  <Barcode />
                </div>
                <p aria-hidden className="mt-1.5 text-xs tracking-[0.3em] text-ink-faint">FSP-TALENTS-RULES</p>
              </div>
              <div
                data-stamp
                aria-hidden
                className="relative flex size-24 shrink-0 items-center justify-center rounded-full border-[3px] border-accent-strong/80 text-center text-xs leading-tight font-bold tracking-[0.12em] text-accent-strong/80 uppercase"
                style={{ transform: "rotate(-12deg)" }}
              >
                <span className="absolute inset-1.5 rounded-full border border-accent-strong/50" />
                Одинаково
                <br />
                для всех
              </div>
            </div>
          </div>
          <div aria-hidden className="h-2.5 bg-[linear-gradient(135deg,var(--color-white)_50%,transparent_50%),linear-gradient(225deg,var(--color-white)_50%,transparent_50%)] bg-[length:14px_14px] bg-repeat-x" />
        </div>
      </div>
    </div>
  );
}
