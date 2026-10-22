import { describe, expect, it } from "vitest";
import { DEFAULT_SECTION, SECTIONS, stepSection } from "./sections";

describe("sections", () => {
  it("has unique ids and a label and a blurb for each", () => {
    expect(new Set(SECTIONS.map((s) => s.id)).size).toBe(SECTIONS.length);
    for (const section of SECTIONS) {
      expect(section.label.length).toBeGreaterThan(0);
      expect(section.blurb.length).toBeGreaterThan(10);
    }
  });

  it("opens on a section that exists", () => {
    expect(SECTIONS.map((s) => s.id)).toContain(DEFAULT_SECTION);
  });
});

describe("keyboard navigation between sections", () => {
  it("moves down and up one section at a time", () => {
    expect(stepSection("converters", "ArrowDown")).toBe("appearance");
    expect(stepSection("storage", "ArrowUp")).toBe("appearance");
  });

  it("stays put at the ends", () => {
    expect(stepSection("converters", "ArrowUp")).toBe("converters");
    expect(stepSection("shortcut", "ArrowDown")).toBe("shortcut");
  });

  it("ignores other keys", () => {
    expect(stepSection("storage", "Enter")).toBe("storage");
    expect(stepSection("storage", "a")).toBe("storage");
  });
});
