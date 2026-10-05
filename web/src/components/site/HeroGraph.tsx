"use client";

import { useEffect, useRef } from "react";
import { MOTION_OK, gsap } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

interface GraphNode {
  bx: number;
  by: number;
  x: number;
  y: number;
  r: number;
  phase: number;
  amp: number;
  speed: number;
  hub: boolean;
  red: boolean;
}

interface Pulse {
  edge: number;
  t: number;
  speed: number;
  red: boolean;
}

function mulberry32(seed: number) {
  let a = seed;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function buildGraph(w: number, h: number) {
  const rand = mulberry32(20260405);
  const mobile = w < 768;
  const count = Math.round(Math.min(96, Math.max(24, (w * h) / (mobile ? 22000 : 16500))));
  const minDist = mobile ? 34 : 52;
  const nodes: GraphNode[] = [];
  for (let tries = 0; nodes.length < count && tries < count * 40; tries++) {
    const x = mobile ? w * (0.04 + 0.92 * rand()) : w * (0.08 + 0.9 * Math.pow(rand(), 0.62));
    const y = h * (0.06 + 0.88 * rand());
    if (nodes.some((n) => Math.hypot(n.bx - x, n.by - y) < minDist)) continue;
    const hub = rand() < 0.12;
    nodes.push({ bx: x, by: y, x, y, r: hub ? 3.2 : 1.4 + rand() * 1.1, phase: rand() * Math.PI * 2, amp: 4 + rand() * 12, speed: 0.15 + rand() * 0.35, hub, red: hub && rand() < 0.25 });
  }
  const edges: [number, number][] = [];
  const seen = new Set<string>();
  nodes.forEach((n, i) => {
    const near = nodes
      .map((m, j) => ({ j, d: Math.hypot(m.bx - n.bx, m.by - n.by) }))
      .filter((e) => e.j !== i && e.d < (mobile ? 150 : 230))
      .sort((a, b) => a.d - b.d)
      .slice(0, n.hub ? 3 : 2);
    for (const { j } of near) {
      const key = i < j ? `${i}-${j}` : `${j}-${i}`;
      if (seen.has(key)) continue;
      seen.add(key);
      edges.push([i, j]);
    }
  });
  return { nodes, edges };
}

function connector(ctx: CanvasRenderingContext2D, ax: number, ay: number, bx: number, by: number) {
  const r = Math.min(16, Math.abs(bx - ax) / 2, Math.abs(by - ay) / 2);
  ctx.moveTo(ax, ay);
  if (r < 1) ctx.lineTo(bx, by);
  else {
    ctx.arcTo(bx, ay, bx, by, r);
    ctx.lineTo(bx, by);
  }
}

function pointOnConnector(ax: number, ay: number, bx: number, by: number, t: number): [number, number] {
  const dx = bx - ax;
  const dy = by - ay;
  const total = Math.abs(dx) + Math.abs(dy) || 1;
  const s = t * total;
  if (s <= Math.abs(dx)) return [ax + Math.sign(dx) * s, ay];
  return [bx, ay + Math.sign(dy) * (s - Math.abs(dx))];
}

export function HeroGraph({ className }: { className?: string }) {
  const wrap = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const box = wrap.current;
    const el = canvas.current;
    const ctx = el?.getContext("2d");
    if (!box || !el || !ctx) return;
    const animate = window.matchMedia(MOTION_OK).matches;
    let w = 0;
    let h = 0;
    let graph = buildGraph(1, 1);
    let pulses: Pulse[] = [];
    const pointer = { x: -9999, y: -9999, tx: -9999, ty: -9999, active: false };
    let visible = true;
    const rand = Math.random;

    const spawn = (): Pulse => ({ edge: Math.floor(rand() * Math.max(1, graph.edges.length)), t: 0, speed: 0.18 + rand() * 0.3, red: rand() < 0.14 });

    const resize = () => {
      const rect = box.getBoundingClientRect();
      w = rect.width;
      h = rect.height;
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      el.width = Math.round(w * dpr);
      el.height = Math.round(h * dpr);
      el.style.width = `${w}px`;
      el.style.height = `${h}px`;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      graph = buildGraph(w, h);
      pulses = Array.from({ length: Math.max(6, Math.round(graph.edges.length / 9)) }, () => ({ ...spawn(), t: rand() }));
      if (!animate) draw(0, 0);
    };

    const draw = (time: number, dt: number) => {
      ctx.clearRect(0, 0, w, h);
      pointer.x += (pointer.tx - pointer.x) * 0.12;
      pointer.y += (pointer.ty - pointer.y) * 0.12;
      const R = 170;
      for (const n of graph.nodes) {
        let x = n.bx + (animate ? Math.sin(time * n.speed + n.phase) * n.amp : 0);
        let y = n.by + (animate ? Math.cos(time * n.speed * 0.8 + n.phase) * n.amp * 0.7 : 0);
        if (pointer.active) {
          const d = Math.hypot(x - pointer.x, y - pointer.y);
          if (d < R && d > 0.1) {
            const f = (1 - d / R) ** 2 * 26;
            x += ((x - pointer.x) / d) * f;
            y += ((y - pointer.y) / d) * f;
          }
        }
        n.x = x;
        n.y = y;
      }

      ctx.lineWidth = 1;
      for (const [i, j] of graph.edges) {
        const a = graph.nodes[i];
        const b = graph.nodes[j];
        if (!a || !b) continue;
        let alpha = w < 768 ? 0.08 : 0.13;
        if (pointer.active) {
          const d = Math.min(Math.hypot(a.x - pointer.x, a.y - pointer.y), Math.hypot(b.x - pointer.x, b.y - pointer.y));
          if (d < R * 1.3) alpha += (1 - d / (R * 1.3)) * 0.45;
        }
        ctx.strokeStyle = `rgba(169,163,255,${alpha.toFixed(3)})`;
        ctx.beginPath();
        connector(ctx, a.x, a.y, b.x, b.y);
        ctx.stroke();
      }

      if (animate) {
        for (const p of pulses) {
          p.t += dt * p.speed;
          if (p.t >= 1) Object.assign(p, spawn());
          const edge = graph.edges[p.edge];
          const a = edge && graph.nodes[edge[0]];
          const b = edge && graph.nodes[edge[1]];
          if (!a || !b) continue;
          const [px, py] = pointOnConnector(a.x, a.y, b.x, b.y, p.t);
          const fade = Math.sin(Math.PI * p.t);
          const color = p.red ? "255,111,128" : "214,209,255";
          ctx.fillStyle = `rgba(${color},${(0.16 * fade).toFixed(3)})`;
          ctx.beginPath();
          ctx.arc(px, py, 7, 0, Math.PI * 2);
          ctx.fill();
          ctx.fillStyle = `rgba(${color},${(0.95 * fade).toFixed(3)})`;
          ctx.beginPath();
          ctx.arc(px, py, 1.8, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      for (const n of graph.nodes) {
        let glow = 0;
        if (pointer.active) {
          const d = Math.hypot(n.x - pointer.x, n.y - pointer.y);
          if (d < R) glow = 1 - d / R;
        }
        const base = n.red ? "255,111,128" : n.hub ? "255,255,255" : "214,209,255";
        if (n.hub) {
          ctx.strokeStyle = `rgba(${base},${(0.22 + glow * 0.4).toFixed(3)})`;
          ctx.beginPath();
          ctx.arc(n.x, n.y, n.r + 5 + glow * 3, 0, Math.PI * 2);
          ctx.stroke();
        }
        ctx.fillStyle = `rgba(${base},${(0.55 + glow * 0.45).toFixed(3)})`;
        ctx.beginPath();
        ctx.arc(n.x, n.y, n.r + glow * 1.4, 0, Math.PI * 2);
        ctx.fill();
      }

      if (pointer.active) {
        for (const n of graph.nodes) {
          const d = Math.hypot(n.x - pointer.x, n.y - pointer.y);
          if (d > 140) continue;
          ctx.strokeStyle = `rgba(123,111,255,${((1 - d / 140) * 0.5).toFixed(3)})`;
          ctx.beginPath();
          connector(ctx, pointer.x, pointer.y, n.x, n.y);
          ctx.stroke();
        }
      }
    };

    resize();
    const ro = new ResizeObserver(resize);
    ro.observe(box);
    if (!animate) return () => ro.disconnect();

    const onMove = (e: PointerEvent) => {
      const rect = box.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      pointer.active = x >= 0 && y >= 0 && x <= rect.width && y <= rect.height;
      pointer.tx = x;
      pointer.ty = y;
      if (pointer.x < -999) {
        pointer.x = x;
        pointer.y = y;
      }
    };
    const onLeave = () => {
      pointer.active = false;
    };
    const io = new IntersectionObserver(([entry]) => {
      visible = Boolean(entry?.isIntersecting);
    });
    io.observe(box);
    const tick = (time: number, deltaMs: number) => {
      if (!visible || document.hidden) return;
      draw(time, Math.min(deltaMs, 50) / 1000);
    };
    gsap.ticker.add(tick);
    window.addEventListener("pointermove", onMove, { passive: true });
    document.documentElement.addEventListener("pointerleave", onLeave);
    const fade = gsap.fromTo(el, { autoAlpha: 0 }, { autoAlpha: 1, duration: 2.2, ease: "power2.out", delay: 0.2 });
    return () => {
      fade.kill();
      gsap.ticker.remove(tick);
      ro.disconnect();
      io.disconnect();
      window.removeEventListener("pointermove", onMove);
      document.documentElement.removeEventListener("pointerleave", onLeave);
    };
  }, []);

  return (
    <div ref={wrap} aria-hidden className={cn("pointer-events-none absolute inset-0", className)}>
      <canvas ref={canvas} className="block" />
    </div>
  );
}
