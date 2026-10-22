/**
 * Options for the dev-only demo pages, read from the address: `?clean` hides the row of
 * preview buttons (for screenshots), and a page may accept more, such as `?preset=email`.
 * Example: http://localhost:1420/?clean&preset=email#imagetool-demo
 */
const params = () => new URLSearchParams(window.location.search);

export const demoParam = (name: string): string | null => params().get(name);

/** Whether the demo should hide its own controls. */
export const isClean = (): boolean => params().has("clean");
