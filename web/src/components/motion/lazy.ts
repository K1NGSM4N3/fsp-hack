"use client";

import { useEffect, useLayoutEffect, useRef, type DependencyList, type RefObject } from "react";
import type * as KitModule from "./gsap";

export type Kit = typeof KitModule;

let kit: Kit | null = null;
let kitRequest: Promise<Kit> | null = null;

/**
 * GSAP and its plugins arrive after hydration as a separate chunk. Nothing painted on first load depends on them:
 * hero copy is visible from the server HTML, and the desktop-only hidden states have a CSS failsafe.
 */
export function loadKit(): Promise<Kit> {
  kitRequest ??= import("./gsap").then((m) => (kit = m));
  return kitRequest;
}

/** The kit if it has already arrived; event handlers fall back to the no-motion path while it is null. */
export function loadedKit(): Kit | null {
  return kit;
}

const useIsoLayoutEffect = typeof window === "undefined" ? useEffect : useLayoutEffect;

const pending: Array<() => void> = [];
let draining = false;

/**
 * Activates one component per task instead of all at once when the kit arrives: each setup (selectors, ScrollTriggers,
 * initial sets) gets its own short task, so a phone never sees one long block of work after hydration.
 */
function activate(task: () => void, urgent: boolean) {
  if (urgent) pending.unshift(task);
  else pending.push(task);
  if (draining) return;
  draining = true;
  const step = () => {
    const next = pending.shift();
    if (!next) {
      draining = false;
      return;
    }
    next();
    setTimeout(step, 0);
  };
  setTimeout(step, 0);
}

type Setup<E> = (kit: Kit, extra: E, context: gsap.Context) => void | (() => void);

interface Options<E> {
  scope: RefObject<Element | null>;
  dependencies?: DependencyList;
  /** Like useGSAP: revert everything from the previous run before running again. */
  revertOnUpdate?: boolean;
  /** Page-specific plugins (Draggable, Flip, MotionPath, MorphSVG) load next to the kit, only where used. */
  extra?: () => Promise<E>;
  /** Jump the activation queue: the hero and the header go first. */
  urgent?: boolean;
}

interface State<E> {
  setup: Setup<E>;
  scope: RefObject<Element | null>;
  revertOnUpdate: boolean;
  loaded: { kit: Kit; extra: E } | null;
  context: gsap.Context | null;
}

/** Revert mode gets a fresh context per run; otherwise every run adds to one context that lives until unmount. */
function run<E>(state: State<E>) {
  const loaded = state.loaded;
  if (!loaded) return;
  const scope = state.scope.current ?? undefined;
  if (state.revertOnUpdate) {
    state.context?.revert();
    state.context = loaded.kit.gsap.context((self) => state.setup(loaded.kit, loaded.extra, self), scope);
    return;
  }
  const context = (state.context ??= loaded.kit.gsap.context(() => {}, scope));
  context.add(() => state.setup(loaded.kit, loaded.extra, context));
}

/**
 * useGSAP with a lazily loaded GSAP. The setup runs inside a gsap.context scoped to `scope` once the kit (and `extra`)
 * have loaded, again whenever `dependencies` change, and everything it created is reverted on unmount; a function
 * returned from setup runs on revert. Loading never re-renders the component: activation happens outside React.
 */
export function useMotion<E = undefined>(setup: Setup<E>, { scope, dependencies = [], revertOnUpdate = false, extra, urgent = false }: Options<E>) {
  const state = useRef<State<E>>({ setup, scope, revertOnUpdate, loaded: null, context: null });
  const loadExtra = useRef(extra);
  const mounted = useRef(false);

  useIsoLayoutEffect(() => {
    state.current.setup = setup;
  });

  useEffect(() => {
    const current = state.current;
    let live = true;
    const loading = loadExtra.current?.() ?? Promise.resolve(undefined as E);
    void Promise.all([loadKit(), loading]).then(([k, e]) =>
      activate(() => {
        if (!live) return;
        current.loaded = { kit: k, extra: e };
        run(current);
      }, urgent),
    );
    return () => {
      live = false;
      current.context?.revert();
      current.context = null;
      current.loaded = null;
    };
  }, [urgent]);

  useIsoLayoutEffect(() => {
    if (!mounted.current) {
      mounted.current = true;
      return;
    }
    run(state.current);
  }, dependencies);
}
