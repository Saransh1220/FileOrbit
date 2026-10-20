import { useDesign } from "../design/useDesign";
import { useRingDrag } from "./useRingDrag";
import { ringVariantFor } from "./variants/ringVariants";

export function Ring() {
  const { hovered, confirmed, hub, fileName, wedges, finish } = useRingDrag();
  const { View } = ringVariantFor(useDesign());
  return (
    <View
      wedges={wedges}
      fileName={fileName}
      hovered={hovered}
      confirmed={confirmed}
      hub={hub}
      onConfirmed={finish}
    />
  );
}
