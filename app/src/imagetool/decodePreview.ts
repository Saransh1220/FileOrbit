import type { Estimate } from "./types";

/** What `imagetool_preview` sends back, unpacked. */
export interface DecodedPreview {
  estimate: Estimate;
  /** The picture itself, ready for an <img>. */
  blob: Blob;
}

const PNG_SIGNATURE = 0x89504e47;

/** Splits the reply: a 4-byte big-endian length, that many bytes of JSON, then the picture. */
export function decodePreview(buffer: ArrayBuffer): DecodedPreview {
  const view = new DataView(buffer);
  const headerLength = view.getUint32(0);
  const header = JSON.parse(new TextDecoder().decode(new Uint8Array(buffer, 4, headerLength))) as Estimate;
  const picture = buffer.slice(4 + headerLength);
  const isPng = picture.byteLength >= 4 && new DataView(picture).getUint32(0) === PNG_SIGNATURE;
  return { estimate: header, blob: new Blob([picture], { type: isPng ? "image/png" : "image/jpeg" }) };
}
