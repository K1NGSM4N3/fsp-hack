"use client";

import { Lock, LockOpen, ScrollText } from "lucide-react";
import { useRef } from "react";
import { MOTION_OK, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";

const FIELDS = [
  { label: "ФИО", cipher: "v1·Xk9fQ2LmA0rT7wPz3Hc", plain: "Имя Фамилия" },
  { label: "Email", cipher: "v1·b3JkZXIgaXMgYmxpbmQ", plain: "name@example.ru" },
  { label: "Телефон", cipher: "v1·Q8nW1sYe0vKp4uR6dJ", plain: "+7 999 000-00-00" },
];

const CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/** Contacts stay ciphertext until the candidate accepts; the access then lands in the log. */
export function CipherReveal() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const el = root.current;
      if (!el) return;
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const values = gsap.utils.toArray<HTMLElement>("[data-value]", el);
        const locked = el.querySelector("[data-locked]");
        const open = el.querySelector("[data-open]");
        const log = el.querySelector("[data-log]");
        const badge = el.querySelector<HTMLElement>("[data-badge]");
        const tl = gsap.timeline({ repeat: -1, repeatDelay: 1.2, delay: 1.2 });
        tl.set(open, { autoAlpha: 0 })
          .set(log, { autoAlpha: 0, y: 10 })
          .to({}, { duration: 2 })
          .call(() => badge && (badge.textContent = "Приглашение принято"))
          .to(locked, { autoAlpha: 0, duration: 0.3 })
          .to(open, { autoAlpha: 1, duration: 0.3 }, "<");
        values.forEach((v, i) => {
          tl.to(v, { duration: 1.1, scrambleText: { text: FIELDS[i]?.plain ?? "", chars: CHARS, speed: 0.5 } }, i === 0 ? ">" : "<0.15");
        });
        tl.to(log, { autoAlpha: 1, y: 0, duration: 0.6, ease: "expo.out" })
          .to({}, { duration: 3 })
          .call(() => badge && (badge.textContent = "Ждёт решения кандидата"))
          .to(log, { autoAlpha: 0, duration: 0.3 })
          .to(open, { autoAlpha: 0, duration: 0.3 }, "<")
          .to(locked, { autoAlpha: 1, duration: 0.3 }, "<");
        values.forEach((v, i) => {
          tl.to(v, { duration: 0.8, scrambleText: { text: FIELDS[i]?.cipher ?? "", chars: CHARS, speed: 0.5 } }, i === 0 ? "<" : "<0.1");
        });
        ScrollTrigger.create({ trigger: el, start: "top bottom", end: "bottom top", onToggle: (self) => (self.isActive ? tl.resume() : tl.pause()) });
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="rounded-[26px] border border-line bg-white p-5 shadow-lift sm:p-6">
      <div className="flex items-center justify-between gap-3">
        <p className="font-mono text-[11px] tracking-[0.12em] text-ink-subtle uppercase">Контакты кандидата K-5F8A1F</p>
        <span className="relative inline-flex size-9 items-center justify-center rounded-full bg-ink text-white">
          <Lock data-locked className="absolute size-4" aria-hidden />
          <LockOpen data-open className="invisible absolute size-4 text-success-300" aria-hidden />
        </span>
      </div>
      <p data-badge className="mt-2 text-sm font-medium text-ink">
        Ждёт решения кандидата
      </p>
      <dl className="mt-5 space-y-2.5">
        {FIELDS.map((f) => (
          <div key={f.label} className="flex items-center justify-between gap-4 rounded-2xl border border-line bg-canvas/70 px-4 py-3">
            <dt className="text-xs text-ink-subtle">{f.label}</dt>
            <dd data-value className="truncate font-mono text-sm text-ink">
              {f.cipher}
            </dd>
          </div>
        ))}
      </dl>
      <div data-log className="mt-4 flex items-start gap-3 rounded-2xl bg-ink px-4 py-3 text-xs text-white">
        <ScrollText className="mt-0.5 size-4 shrink-0 text-primary-300" aria-hidden />
        <span className="font-mono leading-relaxed">
          журнал · компания #4F2A · основание: приглашение принято · доступ к контактам записан
        </span>
      </div>
      <p className="mt-4 text-xs leading-relaxed text-ink-subtle">XChaCha20-Poly1305 в базе, расшифровка — только через основание доступа. Значения на схеме условные.</p>
    </div>
  );
}
