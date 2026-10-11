"use client";

import { ArrowLeft, ArrowRight, Check, FileText, LockOpen, Mail, MapPin, X } from "lucide-react";
import { useRef, useState, type KeyboardEvent, type ReactNode } from "react";
import { useMotion } from "@/components/motion/lazy";
import { cn } from "@/lib/cn";

interface Day {
  day: string;
  title: string;
  text: string;
  scene: ReactNode;
}

const RING = 2 * Math.PI * 14;

const loadDraggable = () => import("@/components/motion/extras/draggable");

function Shell({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn("w-full max-w-[18.125rem] rounded-[18px] bg-white p-4 shadow-pop", className)}>{children}</div>;
}

function ProfileScene() {
  return (
    <Shell>
      <div data-pop className="flex items-center justify-between rounded-xl border border-line px-3 py-2">
        <span className="text-[11px] text-ink-subtle">Направление</span>
        <span className="text-[13px] font-semibold text-ink">Бэкенд</span>
      </div>
      <div data-pop className="mt-2.5 rounded-xl border border-line px-3 py-2">
        <span className="text-[11px] text-ink-subtle">Стек</span>
        <span className="mt-1.5 flex flex-wrap gap-1.5">
          {["Go", "PostgreSQL", "Kafka", "Docker"].map((s) => (
            <span key={s} className="rounded-full bg-primary-50 px-2 py-0.5 text-[11.5px] font-medium text-primary-strong">
              {s}
            </span>
          ))}
        </span>
      </div>
      <div data-pop className="mt-2.5 flex items-center gap-2 rounded-xl bg-success-50 px-3 py-2 text-[12px] text-success">
        <FileText className="size-4 shrink-0" aria-hidden />
        <span className="font-medium">resume.pdf</span>
        <span className="ml-auto">поля заполнены</span>
      </div>
    </Shell>
  );
}

function TestScene() {
  return (
    <Shell>
      <div data-pop className="flex items-center justify-between">
        <span className="font-mono text-[11px] tracking-[0.1em] text-ink-subtle uppercase">Задание 6</span>
        <span className="relative flex size-9 items-center justify-center">
          <svg viewBox="0 0 36 36" className="absolute inset-0 -rotate-90" aria-hidden>
            <circle cx="18" cy="18" r="14" fill="none" strokeWidth="3" className="stroke-primary-100" />
            <circle data-ring cx="18" cy="18" r="14" fill="none" strokeWidth="3" strokeLinecap="round" strokeDasharray={`${RING * 0.62} ${RING}`} className="stroke-primary" />
          </svg>
          <span className="tabular font-mono text-[9.5px] font-semibold text-ink">1:04</span>
        </span>
      </div>
      <p data-pop className="mt-2 text-[13.5px] leading-snug font-semibold text-ink">
        Пользователь дважды нажал «Оплатить». Как не списать деньги дважды?
      </p>
      <ul className="mt-3 space-y-1.5">
        {[false, true, false].map((on, i) => (
          <li key={i} data-pop className={cn("flex items-center gap-2 rounded-lg border px-2.5 py-1.5", on ? "border-primary bg-primary-50" : "border-line")}>
            <span className={cn("size-3 shrink-0 rounded-full border", on ? "border-[4px] border-primary" : "border-line-strong")} />
            <span className={cn("h-1.5 rounded-full", on ? "w-[70%] bg-primary-300" : i ? "w-[52%] bg-line" : "w-[64%] bg-line")} />
          </li>
        ))}
      </ul>
    </Shell>
  );
}

function CategoryScene() {
  return (
    <div className="flex flex-col items-center">
      <div data-stamp className="-rotate-3 rounded-[20px] border-2 border-primary bg-white px-6 py-4 text-center shadow-pop">
        <p className="eyebrow text-primary">Категория</p>
        <p className="mt-1.5 text-[26px] leading-none font-semibold tracking-tight text-ink">Бэкенд · Middle</p>
      </div>
      <p data-pop className="mt-6 flex items-center gap-2 rounded-full bg-white px-3.5 py-1.5 text-[12.5px] font-medium text-ink shadow-card">
        <span className="relative flex size-2">
          <span className="absolute inset-0 rounded-full bg-success" />
        </span>
        Профиль виден компаниям
      </p>
    </div>
  );
}

function InviteScene() {
  return (
    <Shell>
      <div data-pop className="flex items-center gap-2.5">
        <span className="flex size-9 items-center justify-center rounded-xl bg-primary text-sm font-semibold text-white">К</span>
        <span className="leading-tight">
          <span className="block text-[13px] font-semibold text-ink">Кедр Платежи</span>
          <span className="block text-[11px] text-ink-subtle">Финтех</span>
        </span>
        <span className="ml-auto rounded-full bg-primary px-2 py-0.5 text-[10.5px] font-medium text-white">Новое</span>
      </div>
      <p data-pop className="mt-3 text-[13.5px] font-semibold text-ink">Бэкенд-разработчик в платёжную команду</p>
      <p data-pop className="mt-1 flex items-center gap-1 text-[11.5px] text-ink-subtle">
        <MapPin className="size-3.5" aria-hidden /> Гибрид · Москва
      </p>
      <p data-pop className="tabular mt-3 rounded-xl bg-canvas px-3 py-2 font-mono text-[15px] font-semibold text-ink">280 000 – 350 000 ₽</p>
    </Shell>
  );
}

function DecideScene() {
  return (
    <Shell className="space-y-2.5">
      <div data-pop className="rounded-xl border border-line px-3 py-2.5">
        <div className="flex items-center justify-between gap-2">
          <span className="text-[13px] font-semibold text-ink">Сота Логистика</span>
          <span className="inline-flex items-center gap-1 rounded-full bg-accent-50 px-2 py-0.5 text-[10.5px] font-medium text-accent-strong">
            <X className="size-3" aria-hidden /> Отклонено
          </span>
        </div>
        <p className="mt-1 text-[11.5px] text-ink-subtle">
          Причина: <span className="font-medium text-ink">формат работы</span>
        </p>
      </div>
      <div data-pop className="rounded-xl border border-primary/40 bg-primary-50 px-3 py-2.5">
        <div className="flex items-center justify-between gap-2">
          <span className="text-[13px] font-semibold text-ink">Кедр Платежи</span>
          <span className="inline-flex items-center gap-1 rounded-full bg-success px-2 py-0.5 text-[10.5px] font-medium text-white">
            <Check className="size-3" aria-hidden /> Принято
          </span>
        </div>
        <p className="mt-1 text-[11.5px] text-ink-muted">Компания получила ваши контакты</p>
      </div>
    </Shell>
  );
}

function TalkScene() {
  return (
    <Shell>
      <div data-pop className="flex items-center gap-2 text-[11px] font-medium text-success">
        <LockOpen className="size-4" aria-hidden /> Контакты открыты для Кедр Платежи
      </div>
      <dl className="mt-2.5 space-y-1.5 text-[12px]">
        {[
          ["Имя", "Анна Соколова"],
          ["Почта", "anna@example.ru"],
        ].map(([k, v]) => (
          <div key={k} data-pop className="flex items-center justify-between rounded-lg bg-canvas px-2.5 py-1.5">
            <dt className="text-ink-subtle">{k}</dt>
            <dd className="relative font-medium text-ink">
              {v}
              <span data-wipe aria-hidden className="absolute -inset-x-1 inset-y-0 rounded bg-ink" style={{ transform: "scaleX(0)" }} />
            </dd>
          </div>
        ))}
      </dl>
      <div data-pop className="mt-3 flex items-start gap-2.5 rounded-xl border border-line px-3 py-2.5">
        <Mail className="mt-0.5 size-4 shrink-0 text-primary" aria-hidden />
        <span className="text-[12px] leading-snug text-ink">
          <span className="block font-semibold">Кедр Платежи</span>
          Анна, добрый день. Удобно созвониться в среду?
        </span>
      </div>
    </Shell>
  );
}

const DAYS: Day[] = [
  {
    day: "Понедельник",
    title: "Профиль",
    text: "Почта, пароль и короткая анкета: направление, стек, ожидания по зарплате. Если есть резюме в PDF, загрузите его, и поля заполнятся сами.",
    scene: <ProfileScene />,
  },
  {
    day: "Вторник",
    title: "Тест",
    text: "Выбираете грейд, на который претендуете, и проходите тест по своему направлению. Обычно уходит от 10 до 25 минут.",
    scene: <TestScene />,
  },
  {
    day: "Вторник, сразу после теста",
    title: "Категория",
    text: "Результат виден сразу. С категорией профиль попадает в поиск компаний, но без имени и контактов.",
    scene: <CategoryScene />,
  },
  {
    day: "Четверг",
    title: "Первое приглашение",
    text: "Компания пишет сама: что за задача, формат, город и зарплата от и до. На ответ у вас 14 дней.",
    scene: <InviteScene />,
  },
  {
    day: "Пятница",
    title: "Ответы",
    text: "Одно приглашение не подошло по формату: отклоняете и отмечаете причину. Другое нравится: нажимаете «Принять».",
    scene: <DecideScene />,
  },
  {
    day: "Следующий понедельник",
    title: "Разговор",
    text: "Компания получила ваше имя и контакты и пишет вам сама. О созвоне и условиях вы договариваетесь напрямую.",
    scene: <TalkScene />,
  },
];

const INSET = "pl-[max(1rem,calc((100vw-1240px)/2+1rem))] pr-[max(1rem,calc((100vw-1240px)/2+1rem))] sm:pl-[max(1.5rem,calc((100vw-1240px)/2+1.5rem))] sm:pr-[max(1.5rem,calc((100vw-1240px)/2+1.5rem))] lg:pl-[max(2rem,calc((100vw-1240px)/2+2rem))] lg:pr-[max(2rem,calc((100vw-1240px)/2+2rem))]";
const SCROLL_INSET = "scroll-pl-[max(1rem,calc((100vw-1240px)/2+1rem))] sm:scroll-pl-[max(1.5rem,calc((100vw-1240px)/2+1.5rem))] lg:scroll-pl-[max(2rem,calc((100vw-1240px)/2+2rem))]";

interface StripApi {
  goTo: (i: number) => void;
}

/**
 * A week as a filmstrip. Without motion it is a native snap-scrolling row;
 * with motion the row becomes a Draggable with inertia that snaps to days, plus trackpad swipes and the arrow buttons.
 * Touch drags are horizontal only: vertical swipes stay native page scroll, and address-bar resizes (height only)
 * do not re-measure the strip.
 */
export function WeekStrip() {
  const root = useRef<HTMLDivElement>(null);
  const viewport = useRef<HTMLDivElement>(null);
  const track = useRef<HTMLOListElement>(null);
  const fill = useRef<HTMLSpanElement>(null);
  const api = useRef<StripApi | null>(null);
  const [active, setActive] = useState(0);
  const activeRef = useRef(0);

  const mark = (i: number) => {
    activeRef.current = i;
    setActive(i);
  };

  useMotion(
    ({ MOTION_OK, ScrollTrigger, gsap }, { Draggable }) => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const vp = viewport.current;
        const tr = track.current;
        if (!vp || !tr) return;
        const panels = gsap.utils.toArray<HTMLElement>("[data-panel]", tr);
        vp.dataset.drag = "on";
        vp.scrollLeft = 0;

        const scenes = panels.map((panel) => {
          const tl = gsap.timeline({ paused: true });
          const pops = panel.querySelectorAll("[data-pop]");
          const stamp = panel.querySelector("[data-stamp]");
          const ring = panel.querySelector("[data-ring]");
          const wipes = panel.querySelectorAll("[data-wipe]");
          if (stamp) tl.from(stamp, { scale: 1.9, rotation: -16, autoAlpha: 0, duration: 0.55, ease: "back.out(2.4)" }, 0);
          if (pops.length) tl.from(pops, { autoAlpha: 0, y: 14, scale: 0.96, duration: 0.6, ease: "back.out(1.7)", stagger: 0.08 }, stamp ? 0.3 : 0);
          if (ring) tl.fromTo(ring, { drawSVG: "0% 100%" }, { drawSVG: "0% 62%", duration: 1.6, ease: "power2.inOut" }, 0.2);
          if (wipes.length) tl.fromTo(wipes, { scaleX: 1 }, { scaleX: 0, transformOrigin: "100% 50%", duration: 0.6, ease: "power3.inOut", stagger: 0.12 }, 0.5);
          return { tl, played: false };
        });

        let snaps: number[] = [];
        let minX = 0;
        const measure = () => {
          const pad = parseFloat(getComputedStyle(tr).paddingLeft) || 0;
          minX = Math.min(0, vp.clientWidth - tr.scrollWidth);
          snaps = panels.map((p) => gsap.utils.clamp(minX, 0, -(p.offsetLeft - pad)));
        };
        measure();

        const nearest = (x: number) => {
          let best = 0;
          snaps.forEach((s, i) => {
            if (Math.abs(s - x) < Math.abs((snaps[best] ?? 0) - x)) best = i;
          });
          return best;
        };

        const sync = (fromDrag: boolean) => {
          const x = Number(gsap.getProperty(tr, "x"));
          gsap.set(fill.current, { scaleX: minX ? x / minX : 0 });
          if (fromDrag) {
            const i = nearest(x);
            if (i !== activeRef.current) mark(i);
          }
          const width = vp.clientWidth;
          panels.forEach((p, i) => {
            const scene = scenes[i];
            if (!scene || scene.played) return;
            const left = p.offsetLeft + x;
            if (left < width * 0.8 && left + p.offsetWidth * 0.5 > 0) {
              scene.played = true;
              scene.tl.play();
            }
          });
        };

        const drag = Draggable.create(tr, {
          type: "x",
          inertia: true,
          allowNativeTouchScrolling: true,
          bounds: { minX, maxX: 0 },
          edgeResistance: 0.85,
          minimumMovement: 4,
          zIndexBoost: false,
          cursor: "grab",
          activeCursor: "grabbing",
          snap: { x: (v: number) => snaps[nearest(v)] ?? 0 },
          onDrag: () => sync(true),
          onThrowUpdate: () => sync(true),
          onThrowComplete: () => sync(true),
        })[0];

        const goTo = (i: number) => {
          const idx = gsap.utils.clamp(0, panels.length - 1, i);
          mark(idx);
          gsap.to(tr, {
            x: snaps[idx] ?? 0,
            duration: 0.9,
            ease: "expo.out",
            overwrite: true,
            onUpdate: () => sync(false),
            onComplete: () => drag?.update(),
          });
        };
        api.current = { goTo };

        let settle: gsap.core.Tween | null = null;
        const onWheel = (e: WheelEvent) => {
          if (Math.abs(e.deltaX) <= Math.abs(e.deltaY)) return;
          e.preventDefault();
          const x = gsap.utils.clamp(minX, 0, Number(gsap.getProperty(tr, "x")) - e.deltaX);
          gsap.set(tr, { x });
          drag?.update();
          sync(true);
          settle?.kill();
          settle = gsap.delayedCall(0.18, () => goTo(nearest(Number(gsap.getProperty(tr, "x")))));
        };
        vp.addEventListener("wheel", onWheel, { passive: false });

        let resizeCall: gsap.core.Tween | null = null;
        let lastWidth = vp.clientWidth;
        const onResize = () => {
          if (vp.clientWidth === lastWidth) return;
          lastWidth = vp.clientWidth;
          resizeCall?.kill();
          resizeCall = gsap.delayedCall(0.15, () => {
            measure();
            drag?.applyBounds({ minX, maxX: 0 });
            gsap.set(tr, { x: snaps[activeRef.current] ?? 0 });
            sync(false);
          });
        };
        window.addEventListener("resize", onResize);

        ScrollTrigger.create({ trigger: vp, start: "top 82%", once: true, onEnter: () => sync(false) });

        return () => {
          vp.removeEventListener("wheel", onWheel);
          window.removeEventListener("resize", onResize);
          drag?.kill();
          api.current = null;
          delete vp.dataset.drag;
        };
      });
    },
    { scope: root, extra: loadDraggable },
  );

  const nativeGoTo = (i: number) => {
    const vp = viewport.current;
    const tr = track.current;
    const idx = Math.max(0, Math.min(DAYS.length - 1, i));
    const panel = tr?.querySelectorAll<HTMLElement>("[data-panel]")[idx];
    if (!vp || !tr || !panel) return;
    const pad = parseFloat(getComputedStyle(tr).paddingLeft) || 0;
    vp.scrollTo({ left: panel.offsetLeft - pad });
    mark(idx);
  };

  const goTo = (i: number) => (api.current ? api.current.goTo(i) : nativeGoTo(i));

  const onNativeScroll = () => {
    const vp = viewport.current;
    const tr = track.current;
    if (!vp || !tr || vp.dataset.drag) return;
    const max = vp.scrollWidth - vp.clientWidth;
    if (fill.current) fill.current.style.transform = `scaleX(${max > 0 ? vp.scrollLeft / max : 0})`;
    const panels = tr.querySelectorAll<HTMLElement>("[data-panel]");
    const pad = parseFloat(getComputedStyle(tr).paddingLeft) || 0;
    let best = 0;
    panels.forEach((p, i) => {
      const best0 = panels[best];
      if (best0 && Math.abs(p.offsetLeft - pad - vp.scrollLeft) < Math.abs(best0.offsetLeft - pad - vp.scrollLeft)) best = i;
    });
    const atEnd = vp.scrollLeft >= max - 2;
    if (atEnd && activeRef.current >= best) return;
    if (best !== activeRef.current) mark(best);
  };

  const onKey = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "ArrowRight") {
      e.preventDefault();
      goTo(activeRef.current + 1);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      goTo(activeRef.current - 1);
    } else if (e.key === "Home") {
      e.preventDefault();
      goTo(0);
    } else if (e.key === "End") {
      e.preventDefault();
      goTo(DAYS.length - 1);
    }
  };

  return (
    <div ref={root}>
      <div className="mx-auto flex w-full max-w-[77.5rem] items-center justify-between gap-6 px-4 sm:px-6 lg:px-8">
        <div className="flex min-w-0 flex-1 items-center gap-4">
          <span className="tabular shrink-0 font-mono text-sm text-ink">
            <span className="text-primary">{String(active + 1).padStart(2, "0")}</span>
            <span className="text-ink-faint"> / {String(DAYS.length).padStart(2, "0")}</span>
          </span>
          <span className="relative h-px max-w-md flex-1 bg-line-strong">
            <span ref={fill} className="absolute inset-0 origin-left bg-primary" style={{ transform: "scaleX(0)" }} />
          </span>
          <span className="hidden truncate text-sm text-ink-subtle md:block">{DAYS[active]?.title}</span>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          <button
            type="button"
            onClick={() => goTo(active - 1)}
            disabled={active === 0}
            aria-label="Предыдущий день"
            className="inline-flex size-12 items-center justify-center rounded-full border border-line-strong bg-white text-ink transition-colors hover:border-ink disabled:opacity-35 disabled:hover:border-line-strong"
          >
            <ArrowLeft className="size-5" aria-hidden />
          </button>
          <button
            type="button"
            onClick={() => goTo(active + 1)}
            disabled={active === DAYS.length - 1}
            aria-label="Следующий день"
            className="inline-flex size-12 items-center justify-center rounded-full bg-ink text-white transition-colors hover:bg-primary disabled:opacity-35 disabled:hover:bg-ink"
          >
            <ArrowRight className="size-5" aria-hidden />
          </button>
        </div>
      </div>

      <div
        ref={viewport}
        tabIndex={0}
        role="region"
        aria-roledescription="лента"
        aria-label="Неделя по дням. Листайте стрелками влево и вправо"
        onKeyDown={onKey}
        onScroll={onNativeScroll}
        className={cn(
          "scrollbar-none mt-10 snap-x snap-mandatory overflow-x-auto pb-4 outline-offset-[-2px] select-none data-[drag=on]:snap-none data-[drag=on]:overflow-hidden",
          SCROLL_INSET,
        )}
      >
        <ol ref={track} className={cn("relative flex w-max gap-5 sm:gap-7", INSET)}>
          {DAYS.map((d, i) => (
            <li key={d.title} data-panel aria-current={i === active ? "step" : undefined} className="flex w-[min(82vw,360px)] shrink-0 snap-start flex-col sm:w-[380px]">
              <p className="font-mono text-xs tracking-[0.12em] text-ink-subtle uppercase">{d.day}</p>
              <div className="relative mt-3 h-3" aria-hidden>
                <span className={cn("absolute top-1/2 left-0 h-px", i === DAYS.length - 1 ? "right-0" : "-right-5 sm:-right-7", "bg-line-strong")} />
                <span className={cn("relative block size-3 rounded-full ring-4 ring-canvas transition-colors", i <= active ? "bg-primary" : "bg-line-strong")} />
              </div>
              <div aria-hidden className="mt-5 flex h-[250px] items-center justify-center rounded-[24px] bg-[radial-gradient(90%_80%_at_50%_100%,rgb(64_47_255/0.1),transparent_70%)] px-4">
                {d.scene}
              </div>
              <h3 className="mt-6 flex items-baseline gap-3 text-2xl font-semibold tracking-tight text-ink">
                <span className="font-mono text-sm font-medium text-primary">{String(i + 1).padStart(2, "0")}</span>
                {d.title}
              </h3>
              <p className="mt-2.5 text-[15.5px] leading-relaxed text-ink-muted">{d.text}</p>
            </li>
          ))}
        </ol>
      </div>
    </div>
  );
}
