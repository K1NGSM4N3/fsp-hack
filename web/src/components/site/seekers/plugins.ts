import { Draggable } from "gsap/Draggable";
import { InertiaPlugin } from "gsap/InertiaPlugin";
import { MorphSVGPlugin } from "gsap/MorphSVGPlugin";
import { MOTION_OK, gsap } from "@/components/motion/gsap";

gsap.registerPlugin(Draggable, InertiaPlugin, MorphSVGPlugin);

export { Draggable, InertiaPlugin, MorphSVGPlugin };

export function motionAllowed(): boolean {
  return typeof window !== "undefined" && window.matchMedia(MOTION_OK).matches;
}
