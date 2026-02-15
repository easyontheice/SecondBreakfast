import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap rounded-xl border text-sm font-semibold transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50 active:translate-y-[1px]",
  {
    variants: {
      variant: {
        default:
          "border-[hsl(var(--primary)/0.55)] bg-[hsl(var(--primary))] text-primary-foreground shadow-[inset_0_1px_0_hsl(var(--foreground)/0.3),0_10px_24px_-16px_hsl(var(--primary)/0.7)] hover:bg-[hsl(var(--primary-hover))]",
        secondary:
          "border-[hsl(var(--accent)/0.55)] bg-[hsl(var(--accent)/0.16)] text-foreground hover:bg-[hsl(var(--accent)/0.24)]",
        outline:
          "border-[hsl(var(--foreground)/0.26)] bg-[hsl(var(--background)/0.4)] text-foreground hover:bg-[hsl(var(--foreground)/0.08)]",
        ghost: "border-transparent bg-transparent text-foreground hover:bg-[hsl(var(--foreground)/0.08)]",
        destructive: "border-[hsl(var(--destructive)/0.55)] bg-[hsl(var(--destructive))] text-white hover:bg-[hsl(var(--destructive)/0.9)]"
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-8 rounded-lg px-3 text-xs",
        lg: "h-11 rounded-xl px-6",
        icon: "h-10 w-10"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "default"
    }
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => {
    return <button className={cn(buttonVariants({ variant, size, className }))} ref={ref} {...props} />;
  }
);
Button.displayName = "Button";

export { Button, buttonVariants };
