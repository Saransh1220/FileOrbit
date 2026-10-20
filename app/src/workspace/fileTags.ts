/** How many tag colors there are (--tag-0 ... in WorkspaceView.css); more files reuse them. */
export const TAG_COUNT = 6;

/** The color index for a file's pages, so a page can be traced to its file at a glance. */
export const tagOf = (source: number) => source % TAG_COUNT;
