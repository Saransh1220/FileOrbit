import { describe, expect, it } from "vitest";
import { decodePreview } from "./decodePreview";

function reply(header: object, picture: number[]): ArrayBuffer {
  const json = new TextEncoder().encode(JSON.stringify(header));
  const out = new Uint8Array(4 + json.length + picture.length);
  new DataView(out.buffer).setUint32(0, json.length);
  out.set(json, 4);
  out.set(picture, 4 + json.length);
  return out.buffer;
}

describe("decodePreview", () => {
  it("separates the numbers from the picture", async () => {
    const { estimate, blob } = decodePreview(reply({ width: 10, height: 5, bytes: 3, notes: ["a note"] }, [0xff, 0xd8, 0xff]));
    expect(estimate).toEqual({ width: 10, height: 5, bytes: 3, notes: ["a note"] });
    expect(blob.size).toBe(3);
    expect(blob.type).toBe("image/jpeg");
  });

  it("recognises a PNG by its signature", () => {
    expect(decodePreview(reply({ width: 1, height: 1, bytes: 4, notes: [] }, [0x89, 0x50, 0x4e, 0x47])).blob.type).toBe("image/png");
  });

  it("copes with a picture shorter than a signature", () => {
    expect(decodePreview(reply({ width: 1, height: 1, bytes: 1, notes: [] }, [1])).blob.size).toBe(1);
  });
});
