/** The app mark: a ring of separate segments, one lit. Drawn from one circle so it scales cleanly. */
export function Logo({ size = 22 }: { size?: number }) {
  return (
    <svg className="logo" width={size} height={size} viewBox="0 0 24 24" fill="none" aria-hidden>
      <circle cx="12" cy="12" r="8.5" stroke="currentColor" strokeWidth="4" strokeDasharray="6.4 2.33" opacity="0.32" />
      <circle cx="12" cy="12" r="8.5" stroke="var(--mg-accent)" strokeWidth="4" strokeDasharray="6.4 47" />
    </svg>
  );
}
