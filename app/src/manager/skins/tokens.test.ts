import { describe, expect, it } from "vitest";
import tokensCss from "./tokens.css?raw";

/** Every manager source file (components and their CSS), as text. */
const sources = import.meta.glob(["../**/*.css", "../**/*.tsx"], { query: "?raw", import: "default", eager: true }) as Record<string, string>;

const stripComments = (css: string) => css.replace(/\/\*[\s\S]*?\*\//g, "");

/** The tokens each theme block defines, keyed by the theme id it is for. */
function definedByTheme(): Record<string, Set<string>> {
  const result: Record<string, Set<string>> = {};
  for (const [, selector, body] of stripComments(tokensCss).matchAll(/([^{}]+)\{([^}]*)\}/g)) {
    const id = selector.match(/\[data-design="([a-z]+)"\]/)?.[1];
    if (!id) continue;
    result[id] = new Set([...body.matchAll(/(--mg-[a-z-]+)\s*:/g)].map((m) => m[1]));
  }
  return result;
}

/** Every --mg-* token that a component reads, ignoring the token file itself. */
function usedTokens(): Set<string> {
  const used = new Set<string>();
  for (const [path, text] of Object.entries(sources)) {
    if (path.includes("/skins/")) continue;
    for (const m of text.matchAll(/var\((--mg-[a-z-]+)/g)) used.add(m[1]);
  }
  return used;
}

describe("manager theme tokens", () => {
  const themes = definedByTheme();

  it("defines a block for each theme the manager offers", () => {
    expect(Object.keys(themes).sort()).toEqual(["bold", "brutal", "current"]);
  });

  it("is actually used by components (the check below would pass vacuously otherwise)", () => {
    expect(usedTokens().size).toBeGreaterThan(15);
  });

  for (const theme of ["current", "bold", "brutal"]) {
    it(`gives the ${theme} theme every token the components use`, () => {
      const missing = [...usedTokens()].filter((token) => !themes[theme]?.has(token));
      expect(missing).toEqual([]);
    });
  }

  it("does not define tokens nothing uses", () => {
    const used = usedTokens();
    const unused = [...(themes.current ?? [])].filter((token) => !used.has(token));
    expect(unused).toEqual([]);
  });
});
