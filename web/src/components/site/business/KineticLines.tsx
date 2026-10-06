"use client";

import { useRef } from "react";
import { useMotion } from "@/components/motion/lazy";
import { Container } from "@/components/site/primitives";

interface Line {
  before: string;
  key: string;
  after: string;
  note: string;
}

const LINES: Line[] = [
  {
    before: "В подборку попадают ",
    key: "только сдавшие тест",
    after: " на нужную специализацию и грейд.",
    note: "Уровень подтверждает тест. Резюме и самооценка на категорию не влияют.",
  },
  {
    before: "Зарплату вы называете ",
    key: "в первом сообщении,",
    after: " и расхождения видны сразу.",
    note: "Вилка «от и до» обязательна в каждом приглашении.",
  },
  {
    before: "Ваше приглашение ",
    key: "открывают и читают,",
    after: " потому что спама здесь нет.",
    note: "До 30 приглашений в неделю — квота растёт с доверием к компании — и одно активное приглашение на человека.",
  },
  {
    before: "Каждый отказ ",
    key: "приходит с причиной.",
    after: "",
    note: "Причины складываются в статистику по всем приглашениям, и видно, что мешает чаще: зарплата или формат работы.",
  },
  {
    before: "До согласия у вас ",
    key: "нет чужих персональных данных.",
    after: "",
    note: "Вы видите анонимный профиль без имени и контактов. Меньше данных в ваших системах, меньше рисков.",
  },
];

const SIZE = "font-display text-[clamp(2rem,0.9rem+4.1vw,4.6rem)] leading-[1.02] font-semibold tracking-[-0.045em] text-balance";

/**
 * Server HTML holds each sentence once, in the filled layer: the outline under it is drawn from a data attribute by
 * CSS, so crawlers and screen readers never get the text twice. Desktop splits the fill into letters for the scroll
 * fill-in; the split copy is then hidden from screen readers and a plain sentence stands in for it.
 */
function readableCopy(el: HTMLElement): HTMLElement {
  const copy = document.createElement("span");
  copy.className = "sr-only";
  copy.textContent = el.textContent;
  el.after(copy);
  return copy;
}

/**
 * Each benefit is one sentence drawn as an outline. Desktop: scrolling fills it in letter by letter.
 * Phones: no letter split; the filled copy fades in over the outline once, then the note rises.
 */
export function KineticLines() {
  const root = useRef<HTMLDivElement>(null);

  useMotion(
    ({ MOTION_DESKTOP, MOTION_MOBILE, SplitText, gsap }) => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_MOBILE, () => {
        for (const line of gsap.utils.toArray<HTMLElement>("[data-line]", root.current)) {
          gsap
            .timeline({ scrollTrigger: { trigger: line, start: "top 78%", once: true } })
            .fromTo(line.querySelector("[data-fill-text]"), { opacity: 0 }, { opacity: 1, duration: 1.1, ease: "power1.inOut" })
            .fromTo(line.querySelector("[data-note]"), { opacity: 0, y: 14 }, { opacity: 1, y: 0, duration: 0.6, ease: "power3.out" }, 0.5);
        }
      });
      mm.add(MOTION_DESKTOP, () => {
        const copies: HTMLElement[] = [];
        gsap.utils.toArray<HTMLElement>("[data-line]", root.current).forEach((line, i) => {
          const fill = line.querySelector<HTMLElement>("[data-fill-text]");
          const sentence = line.querySelector<HTMLElement>("[data-sentence]");
          const note = line.querySelector<HTMLElement>("[data-note]");
          if (!fill || !sentence) return;
          copies.push(readableCopy(fill));
          const split = SplitText.create(fill, { type: "words,chars", aria: "hidden" });
          gsap.fromTo(
            split.chars,
            { opacity: 0 },
            { opacity: 1, ease: "none", stagger: 0.05, scrollTrigger: { trigger: line, start: "top 82%", end: "bottom 42%", scrub: 0.4 } },
          );
          gsap.fromTo(
            sentence,
            { xPercent: i % 2 ? -4 : 4 },
            { xPercent: 0, ease: "none", scrollTrigger: { trigger: line, start: "top bottom", end: "bottom 42%", scrub: 0.6 } },
          );
          if (note) {
            gsap.fromTo(
              note,
              { opacity: 0, y: 18 },
              { opacity: 1, y: 0, duration: 0.8, ease: "expo.out", scrollTrigger: { trigger: line, start: "top 50%", toggleActions: "play none none reverse" } },
            );
          }
        });
        return () => copies.forEach((c) => c.remove());
      });
    },
    { scope: root },
  );

  return (
    <div ref={root}>
      <Container>
        <h2 className="eyebrow inline-flex items-center gap-2.5 text-primary-300">
          <span aria-hidden className="size-2 rounded-full bg-primary-300" />
          Что получает ваша команда
        </h2>
        <ol className="mt-10 sm:mt-14">
          {LINES.map((l, i) => (
            <li key={l.key} data-line className="grid grid-cols-1 gap-4 border-t border-night-line py-10 sm:grid-cols-[4rem_1fr] sm:gap-6 sm:py-14 lg:py-16">
              <span className="pt-2 font-mono text-sm text-night-faint">0{i + 1}</span>
              <div className="min-w-0">
                <div data-sentence className="grid [grid-template-areas:'t'] *:[grid-area:t]">
                  <p
                    aria-hidden
                    data-outline={`${l.before}${l.key}${l.after}`}
                    className={`${SIZE} text-transparent [-webkit-text-stroke:1px_rgb(169_163_255/0.55)] before:content-[attr(data-outline)]`}
                  />
                  <p data-fill-text className={`${SIZE} text-white`}>
                    {l.before}
                    <span className="text-primary-300">{l.key}</span>
                    {l.after}
                  </p>
                </div>
                <p data-note className="mt-6 max-w-xl text-base leading-relaxed text-night-muted sm:text-lg">
                  {l.note}
                </p>
              </div>
            </li>
          ))}
        </ol>
      </Container>
    </div>
  );
}
