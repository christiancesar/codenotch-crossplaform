/** Small renderers for the design system pages: they read the live CSS variables, so a page
 * always shows what the app actually uses. */

export function Swatch({ name, cls, note }: { name: string; cls: string; note?: string }) {
  return (
    <div className="flex items-center gap-3 py-1.5">
      <span className={`size-10 shrink-0 rounded-md border border-border ${cls}`} />
      <div className="min-w-0">
        <div className="font-mono text-xs text-foreground">{name}</div>
        {note && <div className="text-xs text-muted-foreground">{note}</div>}
      </div>
    </div>
  );
}

export function SwatchGrid({ items }: { items: { name: string; cls: string; note?: string }[] }) {
  return (
    <div className="not-prose grid grid-cols-1 gap-x-8 rounded-lg bg-background p-4 sm:grid-cols-2">
      {items.map((i) => (
        <Swatch key={i.name} {...i} />
      ))}
    </div>
  );
}

export function TypeSample({ label, cls, sample }: { label: string; cls: string; sample: string }) {
  return (
    <div className="flex items-baseline gap-6 border-b border-border py-3">
      <div className="w-48 shrink-0 font-mono text-xs text-muted-foreground">{label}</div>
      <div className={cls}>{sample}</div>
    </div>
  );
}

/** A geometry token drawn as a bar of its own length. */
export function Measure({ token, note }: { token: string; note?: string }) {
  return (
    <div className="flex items-center gap-4 py-1">
      <div className="w-56 shrink-0 font-mono text-xs text-foreground">{token}</div>
      <div className="h-2 rounded-sm bg-band-ample" style={{ width: `var(${token})` }} />
      {note && <div className="text-xs text-muted-foreground">{note}</div>}
    </div>
  );
}

export function Surface({ children }: { children: React.ReactNode }) {
  return <div className="not-prose dark rounded-lg bg-notch p-6 text-white">{children}</div>;
}
