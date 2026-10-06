"use client";

const FLAG = `try{if(!window.matchMedia("(prefers-reduced-motion: reduce)").matches)document.documentElement.classList.add("motion")}catch(e){}`;

/**
 * Runs during HTML parsing, before first paint, and marks that motion scripts will run. CSS combines the flag with the
 * desktop media query, so hero copy starts hidden only where the desktop reveal is going to show it.
 * On the client the tag is inert text/plain, so React neither re-runs nor warns about it.
 */
export function MotionFlag() {
  return <script type={typeof window === "undefined" ? "text/javascript" : "text/plain"} suppressHydrationWarning dangerouslySetInnerHTML={{ __html: FLAG }} />;
}
