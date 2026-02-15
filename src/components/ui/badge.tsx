import { cn } from "@/lib/utils";

export function Badge({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full border border-[hsl(var(--primary)/0.45)] bg-[hsl(var(--primary)/0.14)] px-2.5 py-0.5 text-xs font-semibold text-foreground",
        className
      )}
      {...props}
    />
  );
}
