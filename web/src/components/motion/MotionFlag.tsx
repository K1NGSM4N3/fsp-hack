const FLAG = `try{if(!window.matchMedia("(prefers-reduced-motion: reduce)").matches)document.documentElement.classList.add("motion")}catch(e){}`;

/**
 * Runs during HTML parsing, before first paint, and marks that motion scripts will run. CSS combines the flag with the
 * desktop media query, so hero copy starts hidden only where the desktop reveal is going to show it.
 * A server component: hydration adopts the parsed tag without running it again, and no client chunk is needed.
 */
export function MotionFlag() {
  return <script dangerouslySetInnerHTML={{ __html: FLAG }} />;
}
