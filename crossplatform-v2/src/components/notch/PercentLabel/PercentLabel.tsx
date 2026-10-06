import { cn } from "@/lib/utils";

export interface PercentLabelProps {
  /** Fraction used, 0 to 1; null when nothing has been read */
  used: number | null;
  /** A count with no denominator: shown as ~N, never as a percentage */
  count?: number | null;
  /** The number is Codenotch's, not the vendor's: prefixed with ~ */
  derived?: boolean;
  className?: string;
}

/**
 * The number under a ring. A dash, never "0%", when nothing was read: nothing read is not the
 * same as nothing used. Space Grotesk with tabular figures so digits do not shift as it changes.
 */
export function PercentLabel({ used, count = null, derived = false, className }: PercentLabelProps) {
  const text = count !== null ? `~${count}` : used === null ? "—" : `${derived ? "~" : ""}${Math.round(used * 100)}%`;
  return <span className={cn("font-sans text-[14px] leading-none font-semibold tabular-nums text-white", className)}>{text}</span>;
}
