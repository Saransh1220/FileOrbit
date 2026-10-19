import type { ComponentType } from "react";
import { RingView } from "../RingView";
import type { RingViewProps } from "../ringViewProps";
import { RingBold } from "./RingBold";
import { RingBrutal } from "./RingBrutal";
import { RingCompass } from "./RingCompass";
import { RingFan } from "./RingFan";
import { RingHex } from "./RingHex";
import { RingPetals } from "./RingPetals";

export interface RingVariant {
  id: string;
  label: string;
  View: ComponentType<RingViewProps>;
}

/** Every dial design the playground can show. The real overlay uses `Ring.tsx`. */
export const RING_VARIANTS: RingVariant[] = [
  { id: "current", label: "Current", View: RingView },
  { id: "bold", label: "Bold", View: RingBold },
  { id: "brutal", label: "Brutal", View: RingBrutal },
  { id: "petals", label: "Petals", View: RingPetals },
  { id: "compass", label: "Compass", View: RingCompass },
  { id: "fan", label: "Fan", View: RingFan },
  { id: "hex", label: "Hex", View: RingHex },
];

/** The design for an id, falling back to the first (current) one for an unknown id. */
export const ringVariantFor = (id: string): RingVariant => RING_VARIANTS.find((v) => v.id === id) ?? RING_VARIANTS[0];
