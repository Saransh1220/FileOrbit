import type { ComponentType } from "react";
import { RingView } from "../RingView";
import type { RingViewProps } from "../ringViewProps";
import { RingBlueprint } from "./RingBlueprint";
import { RingBold } from "./RingBold";
import { RingBrutal } from "./RingBrutal";
import { RingChips } from "./RingChips";
import { RingClock } from "./RingClock";
import { RingHalo } from "./RingHalo";
import { RingOrbit } from "./RingOrbit";

export interface RingVariant {
  id: string;
  label: string;
  View: ComponentType<RingViewProps>;
}

/** Every dial design the playground can show. The real overlay uses `Ring.tsx`. */
export const RING_VARIANTS: RingVariant[] = [
  { id: "current", label: "Current", View: RingView },
  { id: "orbit", label: "Orbit", View: RingOrbit },
  { id: "bold", label: "Bold", View: RingBold },
  { id: "blueprint", label: "Blueprint", View: RingBlueprint },
  { id: "halo", label: "Halo", View: RingHalo },
  { id: "chips", label: "Chips", View: RingChips },
  { id: "clock", label: "Clock", View: RingClock },
  { id: "brutal", label: "Brutal", View: RingBrutal },
];

/** The design for an id, falling back to the first (current) one for an unknown id. */
export const ringVariantFor = (id: string): RingVariant => RING_VARIANTS.find((v) => v.id === id) ?? RING_VARIANTS[0];
