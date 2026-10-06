import Link from "next/link";
import type { ComponentProps, ReactNode } from "react";
import { cn } from "@/lib/cn";
import { Spinner } from "./Spinner";

export type ButtonVariant = "primary" | "secondary" | "soft" | "ghost" | "danger";
export type ButtonSize = "sm" | "md" | "lg";

const VARIANTS: Record<ButtonVariant, string> = {
  primary: "bg-primary text-white shadow-[0_6px_18px_-8px_rgb(64_47_255/0.75)] hover:bg-primary-hover active:bg-primary-active",
  secondary: "border border-line-strong bg-white text-ink hover:border-primary hover:text-primary",
  soft: "bg-primary-50 text-primary-strong hover:bg-primary-100",
  ghost: "text-ink-muted hover:bg-canvas hover:text-ink",
  danger: "bg-accent-strong text-white hover:bg-accent-hover",
};

const SIZES: Record<ButtonSize, string> = {
  sm: "tap h-8 gap-1.5 px-3 text-sm",
  md: "h-10 gap-2 px-4 text-sm",
  lg: "h-12 gap-2 px-6 text-base",
};

export function buttonClasses(variant: ButtonVariant = "primary", size: ButtonSize = "md", className?: string) {
  return cn(
    "inline-flex shrink-0 items-center justify-center whitespace-nowrap rounded-control font-medium transition-[background-color,border-color,color,box-shadow,transform] select-none active:translate-y-px",
    "disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50",
    VARIANTS[variant],
    SIZES[size],
    className,
  );
}

interface ButtonProps extends ComponentProps<"button"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  icon?: ReactNode;
}

export function Button({ variant = "primary", size = "md", loading = false, icon, className, children, disabled, type = "button", ...props }: ButtonProps) {
  return (
    <button type={type} className={buttonClasses(variant, size, className)} disabled={disabled || loading} aria-busy={loading || undefined} {...props}>
      {loading ? <Spinner className="size-4" /> : icon}
      {children}
    </button>
  );
}

interface ButtonLinkProps extends ComponentProps<typeof Link> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  icon?: ReactNode;
}

export function ButtonLink({ variant = "primary", size = "md", icon, className, children, ...props }: ButtonLinkProps) {
  return (
    <Link className={buttonClasses(variant, size, className)} {...props}>
      {icon}
      {children}
    </Link>
  );
}

interface IconButtonProps extends ComponentProps<"button"> {
  label: string;
  size?: "sm" | "md";
}

export function IconButton({ label, size = "md", className, children, type = "button", ...props }: IconButtonProps) {
  return (
    <button
      type={type}
      aria-label={label}
      title={label}
      className={cn(
        "inline-flex shrink-0 items-center justify-center rounded-control text-ink-muted transition-colors hover:bg-canvas hover:text-ink disabled:opacity-50",
        size === "sm" ? "tap size-8" : "size-10",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
