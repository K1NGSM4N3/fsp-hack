import { Draggable } from "gsap/Draggable";
import { InertiaPlugin } from "gsap/InertiaPlugin";
import { gsap } from "../gsap";

gsap.registerPlugin(Draggable, InertiaPlugin);

export { Draggable, InertiaPlugin };
