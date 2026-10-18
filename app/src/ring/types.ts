export interface Wedge {
  id: string;
  label: string;
}

export interface Point {
  x: number;
  y: number;
}

/** Payload of the "ring-opened" event sent by the Rust overlay module. */
export interface RingOpened {
  name: string;
  targets: Wedge[];
}
