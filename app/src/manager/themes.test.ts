import { describe, expect, it } from "vitest";
import { MANAGER_THEMES, visibleThemes } from "./themes";

describe("manager themes", () => {
  it("offers exactly Current, Bold and Brutal", () => {
    expect(MANAGER_THEMES.map((t) => t.id)).toEqual(["current", "bold", "brutal"]);
  });

  it("shows all three while the app's list has not loaded yet", () => {
    expect(visibleThemes([]).map((t) => t.id)).toEqual(["current", "bold", "brutal"]);
  });

  it("ignores themes the app offers that this window has no design for", () => {
    const offered: [string, string][] = [["current", "Current"], ["petals", "Petals"], ["bold", "Bold"], ["brutal", "Brutal"], ["hex", "Hex"]];
    expect(visibleThemes(offered).map((t) => t.id)).toEqual(["current", "bold", "brutal"]);
  });

  it("drops a design the app no longer offers", () => {
    expect(visibleThemes([["current", "Current"], ["bold", "Bold"]]).map((t) => t.id)).toEqual(["current", "bold"]);
  });

  it("describes every theme in a sentence", () => {
    for (const theme of MANAGER_THEMES) expect(theme.blurb.length).toBeGreaterThan(10);
  });
});
