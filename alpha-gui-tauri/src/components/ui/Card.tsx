import clsx from "clsx";
import type { HTMLAttributes } from "react";

export function Card({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <section
      className={clsx(
        "rounded-xl border border-outline-variant bg-surface-container-lowest shadow-panel",
        className,
      )}
      {...props}
    />
  );
}
