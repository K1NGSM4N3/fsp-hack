"use client";

import { useRef, type ReactNode } from "react";
import { MOTION_OK, SplitText, gsap, useGSAP } from "@/components/motion/gsap";

/** Words light up one by one as the paragraph scrolls through the viewport. */
export function ManifestoScrub({ children, eyebrow }: { children: ReactNode; eyebrow: string }) {
  const root = useRef<HTMLDivElement>(null);
  const text = useRef<HTMLParagraphElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        if (!text.current) return;
        SplitText.create(text.current, {
          type: "words",
          wordsClass: "split-word",
          autoSplit: true,
          onSplit: (self) =>
            gsap.fromTo(
              self.words,
              { opacity: 0.12 },
              { opacity: 1, ease: "none", stagger: 0.08, scrollTrigger: { trigger: text.current, start: "top 78%", end: "bottom 45%", scrub: 0.6 } },
            ),
        });
      });
    },
    { scope: root },
  );
  return (
    <div ref={root} className="mx-auto max-w-5xl">
      <p className="eyebrow mb-8 text-primary">{eyebrow}</p>
      <p ref={text} className="text-[clamp(1.7rem,1.1rem+2.4vw,3.3rem)] leading-[1.15] font-semibold tracking-[-0.03em] text-balance text-ink">
        {children}
      </p>
    </div>
  );
}
