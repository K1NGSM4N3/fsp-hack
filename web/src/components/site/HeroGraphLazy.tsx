"use client";

import dynamic from "next/dynamic";

/** Decorative canvas with nothing to server-render: its code loads after hydration, off the critical path. */
export const HeroGraphLazy = dynamic(() => import("./HeroGraph").then((m) => m.HeroGraph), { ssr: false });
