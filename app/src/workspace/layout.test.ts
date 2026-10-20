import { describe, expect, it } from "vitest";
import { cardLayout } from "./layout";

const a4 = { width: 595, height: 842 };

describe("a page card", () => {
  it("keeps the page's proportions", () => {
    const card = cardLayout(a4, 0, 160);
    expect(card.boxWidth / card.boxHeight).toBeCloseTo(595 / 842, 3);
    expect(card.pictureWidth).toBeCloseTo(card.boxWidth, 5);
  });

  it("is never wider than its cell", () => {
    expect(cardLayout({ width: 1000, height: 100 }, 0, 160).boxWidth).toBeLessThanOrEqual(160);
  });

  it("a very tall page is cut down to the row height limit", () => {
    expect(cardLayout({ width: 100, height: 2000 }, 0, 160).boxHeight).toBeLessThanOrEqual(160 * 1.45 + 0.001);
  });

  it("a quarter turn swaps the box but not the picture", () => {
    const upright = cardLayout(a4, 0, 160);
    const turned = cardLayout(a4, 90, 160);
    expect(turned.boxWidth / turned.boxHeight).toBeCloseTo(842 / 595, 3);
    expect(turned.pictureWidth / turned.pictureHeight).toBeCloseTo(upright.pictureWidth / upright.pictureHeight, 5);
  });

  it("a half turn changes nothing about the box", () => {
    expect(cardLayout(a4, 180, 160)).toEqual(cardLayout(a4, 0, 160));
  });
});
