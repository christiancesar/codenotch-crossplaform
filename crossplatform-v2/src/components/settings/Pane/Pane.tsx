import { cn } from "@/lib/utils";

/**
 * The building blocks every settings pane is made of, so the five panes read alike: a title with
 * a one-line lede, blocks with a heading, and rows of name + why + control.
 */

export function Pane({ title, lede, children }: { title: string; lede: string; children: React.ReactNode }) {
  return (
    <section className="flex flex-col gap-7">
      <header className="flex flex-col gap-1.5">
        <h1 className="font-heading text-lg font-semibold">{title}</h1>
        <p className="max-w-prose text-xs/relaxed text-muted-foreground">{lede}</p>
      </header>
      {children}
    </section>
  );
}

export function Block({ title, sub, children, className }: { title?: string; sub?: string; children: React.ReactNode; className?: string }) {
  return (
    <div className={cn("flex flex-col gap-3", className)}>
      {(title || sub) && (
        <div className="flex flex-col gap-1">
          {title && <h2 className="text-[13px] font-medium">{title}</h2>}
          {sub && <p className="max-w-prose text-[11px]/relaxed text-muted-foreground">{sub}</p>}
        </div>
      )}
      {children}
    </div>
  );
}

export interface RowProps {
  name: string;
  /** What the setting does, in plain words */
  why: string;
  /** The switch, select or button */
  control: React.ReactNode;
  /** Extra lines under the explanation: a path, a lock note */
  children?: React.ReactNode;
  /** The control's id, so clicking the name toggles it */
  htmlFor?: string;
}

/** One setting: its name and why on the left, the control on the right. */
export function Row({ name, why, control, children, htmlFor }: RowProps) {
  return (
    <div className="flex items-start gap-6 rounded-lg border bg-card px-4 py-3">
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <label htmlFor={htmlFor} className="text-[13px] font-medium">
          {name}
        </label>
        <p className="max-w-prose text-[11px]/relaxed text-muted-foreground">{why}</p>
        {children}
      </div>
      <div className="flex shrink-0 items-center pt-0.5">{control}</div>
    </div>
  );
}

/** A quiet note under a block: what is held on, where a file lives. */
export function Note({ children, className }: { children: React.ReactNode; className?: string }) {
  return <p className={cn("max-w-prose text-[11px]/relaxed text-muted-foreground", className)}>{children}</p>;
}

/** A path or id the user may want to copy. */
export function Path({ children }: { children: React.ReactNode }) {
  return <code className="w-fit rounded bg-muted px-1.5 py-0.5 font-mono text-xs select-text">{children}</code>;
}
