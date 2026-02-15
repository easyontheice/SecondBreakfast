import * as React from "react";
import { cn } from "@/lib/utils";

interface ProgressProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: number;
}

export function Progress({ value, className, ...props }: ProgressProps) {
  const clamped = typeof value === "number" ? Math.max(0, Math.min(100, value)) : null;

  return (
    <div className={cn("relative h-2 w-full overflow-hidden rounded-full bg-secondary", className)} {...props}>
      {clamped === null ? (
        <div className="absolute inset-y-0 w-1/3 animate-[indeterminate_1.2s_ease-in-out_infinite] rounded-full bg-primary" />
      ) : (
        <div className="h-full bg-primary transition-all" style={{ width: `${clamped}%` }} />
      )}
    </div>
  );
}
