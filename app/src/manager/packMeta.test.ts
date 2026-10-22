import { describe, expect, it } from "vitest";
import { GROUP_ORDER, groupPacks, metaFor, PACK_META, readyCount } from "./packMeta";
import type { PackInfo } from "./types";

const pack = (id: string, installed = false): PackInfo => ({
  id,
  title: id,
  description: "",
  engines: [],
  installed,
  managed: installed,
  download_bytes: 1,
  disk_bytes: 0,
  shared_with: [],
  busy: false,
});

describe("grouping packs", () => {
  it("lists the document group before the media group, keeping each group's order", () => {
    const groups = groupPacks([pack("audio"), pack("pdf"), pack("images"), pack("documents"), pack("video")]);
    expect(groups.map((g) => g.group)).toEqual(["documents", "media"]);
    expect(groups[0].packs.map((p) => p.id)).toEqual(["pdf", "documents"]);
    expect(groups[1].packs.map((p) => p.id)).toEqual(["audio", "images", "video"]);
  });

  it("leaves out a group with nothing in it", () => {
    expect(groupPacks([pack("audio")]).map((g) => g.group)).toEqual(["media"]);
    expect(groupPacks([])).toEqual([]);
  });

  it("shows a pack it has never heard of, in the first group, with a generic look", () => {
    expect(metaFor("zzz").group).toBe("documents");
    expect(groupPacks([pack("zzz")])[0].packs[0].id).toBe("zzz");
  });
});

describe("pack metadata", () => {
  it("covers every pack the installer ships, so a new pack is not forgotten here", () => {
    for (const id of ["pdf", "documents", "text", "audio", "video", "images"]) {
      expect(PACK_META[id], id).toBeDefined();
    }
  });

  it("only uses groups that the display order knows", () => {
    for (const meta of Object.values(PACK_META)) expect(GROUP_ORDER).toContain(meta.group);
  });
});

describe("ready count", () => {
  it("counts the packs that can be used now", () => {
    expect(readyCount([pack("a", true), pack("b"), pack("c", true)])).toBe(2);
    expect(readyCount([])).toBe(0);
  });
});
