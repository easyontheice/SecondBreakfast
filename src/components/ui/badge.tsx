import { cn } from "@/lib/utils";

export function Badge({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full border border-border/80 bg-secondary px-2.5 py-0.5 text-xs font-semibold text-secondary-foreground",
        className
      )}
      {...props}
    />
  );
}
