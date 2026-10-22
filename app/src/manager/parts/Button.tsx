import type { ButtonHTMLAttributes } from "react";
import "./Button.css";

export type ButtonVariant = "primary" | "quiet" | "danger";

/** The one button style the window uses. `primary` is the main action of a row or page. */
export function Button({ variant = "quiet", className = "", ...rest }: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: ButtonVariant }) {
  return <button type="button" className={`btn ${variant} ${className}`.trim()} {...rest} />;
}
