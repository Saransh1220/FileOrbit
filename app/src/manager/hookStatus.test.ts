import { describe, expect, it } from "vitest";
import { needsAttention, STATUS_HELP, STATUS_LABEL, STATUS_SHORT, STATUS_TONE } from "./hookStatus";
import type { HookStatus } from "./types";

const ALL: HookStatus[] = ["starting", "listening", "permission-denied", "unsupported", "error"];

describe("Shift+drag status text", () => {
  it("has a label, a short label and a tone for every status", () => {
    for (const status of ALL) {
      expect(STATUS_LABEL[status], status).toBeTruthy();
      expect(STATUS_SHORT[status], status).toBeTruthy();
      expect(STATUS_TONE[status], status).toBeTruthy();
    }
  });

  it("keeps the short labels short enough for the sidebar footer", () => {
    for (const status of ALL) expect(STATUS_SHORT[status].length, status).toBeLessThanOrEqual(18);
  });

  it("only flags the sidebar when the user has to do something", () => {
    expect(ALL.filter(needsAttention)).toEqual(["permission-denied", "error"]);
  });

  it("explains what to do whenever it flags the sidebar", () => {
    for (const status of ALL.filter(needsAttention)) expect(STATUS_HELP[status], status).toBeTruthy();
  });
});
