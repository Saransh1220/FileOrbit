/** "375 MB", "1.2 GB", "0 B": decimal units, as file managers show them. */
export function formatBytes(bytes: number): string {
  if (bytes < 1000) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = -1;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit += 1;
  }
  // Whole numbers from 100 up; one decimal below that (1.2 GB, 42 MB would read "42.0", so trim it).
  const text = value >= 100 ? value.toFixed(0) : value.toFixed(1).replace(/\.0$/, "");
  return `${text} ${units[unit]}`;
}
