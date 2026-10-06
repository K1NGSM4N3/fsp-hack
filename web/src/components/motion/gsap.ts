import gsap from "gsap";
import { DrawSVGPlugin } from "gsap/DrawSVGPlugin";
import { ScrambleTextPlugin } from "gsap/ScrambleTextPlugin";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { SplitText } from "gsap/SplitText";

gsap.registerPlugin(ScrollTrigger, SplitText, DrawSVGPlugin, ScrambleTextPlugin);
if (typeof window !== "undefined") ScrollTrigger.config({ ignoreMobileResize: true });

export { DrawSVGPlugin, ScrambleTextPlugin, ScrollTrigger, SplitText, gsap };
export * from "./config";

let fontsSettled: Promise<void> | null = null;

/** One refresh per page load once web fonts settle, however many motion roots ask for it. */
export function refreshAfterFonts() {
  if (fontsSettled || typeof document === "undefined" || !document.fonts) return;
  fontsSettled = document.fonts.ready.then(() => ScrollTrigger.refresh());
}
