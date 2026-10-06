import type { Glyph } from "@/libs/ipc";
import { cn } from "@/lib/utils";

export interface ProviderGlyphProps {
  /** The mark from `get_glyphs`. Absent, the letter fallback is drawn. */
  glyph?: Glyph;
  /** One or two letters shown when there is no mark */
  fallback: string;
  /** Edge length in CSS px; the notch uses `--glyph-size`, the card header 16 */
  size?: number;
  className?: string;
}

/**
 * A provider's official mark. Colour marks (`image`) are drawn through an <img>, which isolates
 * their gradient ids and runs no script; monochrome marks (`mark`) are inline SVG that follow the
 * text colour, already sanitized by the backend.
 */
export function ProviderGlyph({ glyph, fallback, size, className }: ProviderGlyphProps) {
  const box = size ? { width: size, height: size } : undefined;
  const cls = cn("block shrink-0", !size && "size-(--glyph-size)", className);
  if (glyph?.kind === "image") {
    return <img src={glyph.url} alt="" draggable={false} className={cn(cls, "object-contain select-none")} style={box} />;
  }
  if (glyph?.kind === "mark") {
    return <span aria-hidden className={cn(cls, "text-white [&>svg]:size-full [&>svg]:fill-current")} style={box} dangerouslySetInnerHTML={{ __html: glyph.svg }} />;
  }
  return (
    <span aria-hidden className={cn(cls, "flex items-center justify-center font-sans text-[15px] font-bold leading-none tracking-tight text-white")} style={box}>
      {fallback}
    </span>
  );
}
