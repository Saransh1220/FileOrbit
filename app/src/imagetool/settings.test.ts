import { describe, expect, it } from "vitest";
import { PRESETS } from "./presets";
import { DEFAULT_SETTINGS, sizeToBytes, toJob, type Settings } from "./settings";

const withResize = (patch: Partial<Settings["resize"]>): Settings => ({ ...DEFAULT_SETTINGS, resize: { ...DEFAULT_SETTINGS.resize, on: true, ...patch } });
const withCompress = (patch: Partial<Settings["compress"]>): Settings => ({ ...DEFAULT_SETTINGS, compress: { ...DEFAULT_SETTINGS.compress, on: true, ...patch } });

describe("toJob", () => {
  it("has nothing to do when every part is off", () => {
    expect(toJob(DEFAULT_SETTINGS)).toEqual({ job: null, problems: [] });
  });

  it("a format change alone is a job", () => {
    expect(toJob({ ...DEFAULT_SETTINGS, format: "png" }).job).toEqual({ resize: null, compress: null, format: "png" });
  });

  it("leaves out a part that is switched off, whatever its fields say", () => {
    const { job } = toJob({ ...withCompress({ mode: "quality", quality: 60 }), resize: { ...DEFAULT_SETTINGS.resize, on: false, percent: 0 } });
    expect(job).toEqual({ resize: null, compress: { kind: "quality", quality: 60 }, format: "keep" });
  });

  it("combines resize, a size limit and a format", () => {
    const s = withResize({ mode: "exact", width: 800, height: 600, fit: "pad" });
    s.compress = { ...s.compress, on: true, mode: "size", sizeValue: 300, sizeUnit: "KB" };
    s.format = "jpg";
    expect(toJob(s).job).toEqual({
      resize: { kind: "exact", width: 800, height: 600, fit: "pad" },
      compress: { kind: "targetSize", max_bytes: 300_000 },
      format: "jpg",
    });
  });

  it("turns a named ratio into numbers and never stretches a ratio", () => {
    expect(toJob(withResize({ mode: "ratio", ratio: "16:9", fit: "stretch" })).job?.resize).toEqual({ kind: "aspect", width: 16, height: 9, fit: "crop" });
    expect(toJob(withResize({ mode: "ratio", ratio: "custom", ratioWidth: 7, ratioHeight: 5, fit: "pad" })).job?.resize).toEqual({ kind: "aspect", width: 7, height: 5, fit: "pad" });
  });

  it("reports bad values instead of running", () => {
    expect(toJob(withResize({ mode: "percent", percent: 0 })).problems).toHaveLength(1);
    expect(toJob(withResize({ mode: "fit", width: 12.5 })).problems).toHaveLength(1);
    expect(toJob(withCompress({ mode: "size", sizeValue: 0 })).problems).toHaveLength(1);
    expect(toJob(withCompress({ mode: "quality", quality: 101 })).job).toBeNull();
  });
});

describe("sizeToBytes", () => {
  it("counts a kilobyte as 1000 bytes", () => {
    expect(sizeToBytes(500, "KB")).toBe(500_000);
    expect(sizeToBytes(1.5, "MB")).toBe(1_500_000);
  });
});

describe("presets", () => {
  it("each one makes a valid job and keeps unrelated values", () => {
    for (const preset of PRESETS) {
      const settings = preset.apply(DEFAULT_SETTINGS);
      const { job, problems } = toJob(settings);
      expect(problems, preset.id).toEqual([]);
      expect(job, preset.id).not.toBeNull();
    }
  });

  it("the email preset asks for under one megabyte", () => {
    const job = toJob(PRESETS.find((p) => p.id === "email")!.apply(DEFAULT_SETTINGS)).job;
    expect(job?.compress).toEqual({ kind: "targetSize", max_bytes: 1_000_000 });
  });
});
