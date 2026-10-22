import type { ReactNode } from "react";
import "./Page.css";

/** The frame every page shares: a title, one line of explanation, then the content. */
export function Page({ title, blurb, children }: { title: string; blurb?: string; children: ReactNode }) {
  return (
    <section className="page" aria-labelledby="page-title">
      <header>
        <h1 id="page-title">{title}</h1>
        {blurb && <p>{blurb}</p>}
      </header>
      {children}
    </section>
  );
}
