"use client";

const FLAG = `try{if(!window.matchMedia("(prefers-reduced-motion: reduce)").matches)document.documentElement.classList.add("motion")}catch(e){}`;

/**
 * Runs during HTML parsing, before first paint: hero copy is hidden only when GSAP is going to reveal it.
 * On the client the tag is inert text/plain, so React neither re-runs nor warns about it.
 */
export function MotionFlag() {
  return <script type={typeof window === "undefined" ? "text/javascript" : "text/plain"} suppressHydrationWarning dangerouslySetInnerHTML={{ __html: FLAG }} />;
}
