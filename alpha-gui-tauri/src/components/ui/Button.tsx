import clsx from "clsx";
import type { ButtonHTMLAttributes, ReactNode } from "react";

type Variant = "primary" | "secondary" | "danger" | "ghost";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: ReactNode;
}

export function Button({
  className,
  variant = "primary",
  icon,
  children,
  ...props
}: ButtonProps) {
  return (
    <button
      className={clsx(
        "inline-flex items-center justify-center gap-2 rounded-lg px-4 py-2.5 text-sm font-semibold transition",
        "focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-45",
        variant === "primary" && "bg-primary text-on-primary shadow-sm hover:bg-primary-container",
        variant === "secondary" &&
          "border border-outline-variant bg-surface-container-lowest text-on-surface hover:bg-surface-container-low",
        variant === "danger" && "bg-error text-white hover:opacity-90",
        variant === "ghost" && "text-primary hover:bg-surface-container",
        className,
      )}
      {...props}
    >
      {icon}
      {children}
    </button>
  );
}
