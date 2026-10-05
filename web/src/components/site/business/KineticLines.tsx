"use client";

import { useRef } from "react";
import { MOTION_OK, SplitText, gsap, useGSAP } from "@/components/motion/gsap";
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
    note: "У каждой компании 30 приглашений в неделю и одно активное приглашение на человека.",
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

/** Each benefit is one sentence drawn as an outline; scrolling fills it in letter by letter. */
export function KineticLines() {
  const root = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        gsap.utils.toArray<HTMLElement>("[data-line]", root.current).forEach((line, i) => {
          const fill = line.querySelector<HTMLElement>("[data-fill-text]");
          const sentence = line.querySelector<HTMLElement>("[data-sentence]");
          const note = line.querySelector<HTMLElement>("[data-note]");
          if (!fill || !sentence) return;
          const split = SplitText.create(fill, { type: "words,chars" });
          gsap.fromTo(
            split.chars,
            { opacity: 0 },
            { opacity: 1, ease: "none", stagger: 0.05, scrollTrigger: { trigger: line, start: "top 82%", end: "bottom 42%", scrub: 0.4 } },
          );
          const drift = window.innerWidth < 768 ? 0 : 4;
          gsap.fromTo(
            sentence,
            { xPercent: i % 2 ? -drift : drift },
            { xPercent: 0, ease: "none", scrollTrigger: { trigger: line, start: "top bottom", end: "bottom 42%", scrub: 0.6 } },
          );
          if (note) {
            gsap.fromTo(
              note,
              { autoAlpha: 0, y: 18 },
              { autoAlpha: 1, y: 0, duration: 0.8, ease: "expo.out", scrollTrigger: { trigger: line, start: "top 50%", toggleActions: "play none none reverse" } },
            );
          }
        });
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
                  <p aria-hidden className={`${SIZE} text-transparent [-webkit-text-stroke:1px_rgb(169_163_255/0.4)]`}>
                    {l.before}
                    {l.key}
                    {l.after}
                  </p>
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
