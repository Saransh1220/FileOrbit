import { RingView } from "./RingView";
import { useRingDrag } from "./useRingDrag";

export function Ring() {
  const { hovered, confirmed, fileName, wedges, finish } = useRingDrag();
  return (
    <RingView
      wedges={wedges}
      fileName={fileName}
      hovered={hovered}
      confirmed={confirmed}
      onConfirmed={finish}
    />
  );
}
