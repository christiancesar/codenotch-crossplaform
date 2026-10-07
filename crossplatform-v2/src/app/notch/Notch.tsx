import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { NotchShell } from "@/components/notch/NotchShell";
import { ProviderCell } from "@/components/notch/ProviderCell";
import { HoverCard } from "@/components/notch/HoverCard";
import { CardHeader } from "@/components/notch/CardHeader";
import { LimitWindowBlock } from "@/components/notch/LimitWindowBlock";
import { SessionList } from "@/components/notch/SessionList";
import { ScaleSlider } from "@/components/notch/ScaleSlider";
import { NoticeToast } from "@/components/notch/NoticeToast";
import type { ActivityArcState } from "@/components/notch/ActivityArc";
import { commands, events, type ProviderId, type Slot } from "@/libs/ipc";
import { notchWindowHeight, providerName } from "@/libs/usage";
import { useNotchData, type NotchData } from "./useNotchData";

/** The window's designed width; a WebView that picked another monitor's DPR is zoomed back to it */
const DESIGN_W = 340;
/** The grace for crossing the gap between card and cell (upstream motion rule) */
const GRACE_MS = 250;
/** Press and move further than this and it is a drag, not a click */
const DRAG_PX = 4;

/** Which rings to draw: the stored slots that still exist, or every provider that is installed. */
function cells(data: NotchData): Slot[] {
  const present = data.usage.filter((u) => u.snapshot.status !== "absent").map((u) => u.provider as string);
  const all = present.map((provider) => ({ provider, window: "" }));
  if (!data.slots.length) return all;
  const on = data.slots.filter((s) => present.includes(s.provider));
  return on.length ? on : all;
}

/** Claude's ring follows its sessions; the others follow the activity the backend detects. */
function activityOf(data: NotchData, id: ProviderId): ActivityArcState {
  if (id === "claude") {
    const agg = data.sessions.agg;
    if (agg === "attention") return "waiting";
    if (agg === "running") return "working";
  }
  const acts = data.activity.filter((a) => a.provider === id);
  if (acts.some((a) => a.state === "waiting")) return "waiting";
  return acts.length ? "working" : "idle";
}

/** A DOM rect in window-relative physical pixels, as `set_hot` takes it. */
const physical = (r: DOMRect): [number, number, number, number] => {
  const k = window.devicePixelRatio || 1;
  return [r.left * k, r.top * k, r.width * k, r.height * k];
};

/**
 * The notch window: the pill on the right edge and the hover card. The window is always the full
 * 340 x 460; Rust makes everything outside the rectangles reported through `set_hot`
 * click-through, so they are reported whenever the pill or the card moves or resizes.
 */
export default function Notch() {
  const [notice, setNotice] = useState<string | null>(null);
  const noticeTimer = useRef<number>(undefined);
  const showNotice = useCallback((msg: string) => {
    setNotice(msg);
    window.clearTimeout(noticeTimer.current);
    noticeTimer.current = window.setTimeout(() => setNotice(null), 6000);
  }, []);
  const { data, ready, setData } = useNotchData(showNotice);

  const root = useRef<HTMLDivElement>(null);
  const shell = useRef<HTMLDivElement>(null);
  const cardWrap = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<{ id: ProviderId; y: number } | null>(null);
  const [atShell, setAtShell] = useState(false);
  const [dragging, setDragging] = useState(false);
  const hideTimer = useRef<number>(undefined);
  const scaleTimer = useRef<number>(undefined);
  const sliding = useRef(false);
  const press = useRef<{ x: number; y: number; id: ProviderId | null } | null>(null);

  // The first layout is a placement: no fold animation on launch
  const first = useRef(true);
  useEffect(() => {
    if (ready) first.current = false;
  }, [ready]);

  const open = hover !== null;
  const collapsed = data.startsCollapsed && !atShell && !open;

  // ---- hot rectangles --------------------------------------------------------------------------
  const reportHot = useCallback(() => {
    const rects: [number, number, number, number][] = [];
    if (shell.current) rects.push(physical(shell.current.getBoundingClientRect()));
    const card = cardWrap.current?.firstElementChild as HTMLElement | null;
    if (open && card) rects.push(physical(card.getBoundingClientRect()));
    commands.setHot(rects, open).catch(() => {});
  }, [open]);

  // While anything animates the rectangles move every frame: report for a while after a change
  useEffect(() => {
    let frame = 0;
    const until = performance.now() + 700;
    const tick = () => {
      reportHot();
      if (performance.now() < until) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [reportHot, hover, collapsed, data.scale, data.usage, data.slots, data.sessions]);

  // ---- window height: grows when the pill does not fit the 460 px minimum ---------------------
  const ringCount = ready ? cells(data).length : 0;
  useEffect(() => {
    if (ringCount) commands.setNotchHeight(notchWindowHeight(ringCount, data.scale / 100)).catch(() => {});
  }, [ringCount, data.scale]);

  // ---- DPR fit ---------------------------------------------------------------------------------
  useLayoutEffect(() => {
    const fit = () => {
      const z = window.innerWidth / DESIGN_W;
      document.documentElement.style.zoom = Math.abs(z - 1) > 0.02 ? String(z) : "";
      commands.reportDpr(window.devicePixelRatio || 1, window.innerWidth, window.innerHeight).catch(() => {});
    };
    fit();
    let t: number | undefined;
    const onResize = () => {
      window.clearTimeout(t);
      t = window.setTimeout(fit, 120);
    };
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  // ---- hover -----------------------------------------------------------------------------------
  const keep = () => window.clearTimeout(hideTimer.current);
  const hide = useCallback(() => {
    window.clearTimeout(hideTimer.current);
    setHover(null);
    setAtShell(false);
  }, []);
  const scheduleHide = () => {
    window.clearTimeout(hideTimer.current);
    hideTimer.current = window.setTimeout(hide, GRACE_MS);
  };
  const enterCell = (id: ProviderId, el: HTMLElement) => {
    if (dragging) return;
    keep();
    const r = el.getBoundingClientRect();
    const top = root.current?.getBoundingClientRect().top ?? 0;
    setHover({ id, y: r.top + r.height / 2 - top });
  };

  // Rust's watchdog (cursor left, focus or workspace changed) collapses it from outside
  useEffect(() => {
    const offs = [
      events.pointerLeft.listen(() => !sliding.current && hide()),
      events.dragEnd.listen(() => {
        setDragging(false);
        press.current = null;
      }),
      events.scale.listen((e) => {
        // The notch's own slider wins while it is being dragged
        if (sliding.current || e.payload === null) return;
        setData((d) => ({ ...d, scale: Math.max(40, Math.min(100, Math.round((e.payload as number) * 100))) }));
      }),
    ];
    return () => offs.forEach((off) => off.then((f) => f()));
  }, [hide, setData]);

  // ---- press: click opens the provider's page, a move hands the drag to Rust -------------------
  useEffect(() => {
    const move = (e: PointerEvent) => {
      const p = press.current;
      if (!p || dragging) return;
      if (Math.abs(e.clientX - p.x) > DRAG_PX || Math.abs(e.clientY - p.y) > DRAG_PX) {
        setDragging(true);
        hide();
        commands.dragBegin().catch((err) => {
          showNotice(`drag_begin failed: ${String(err)}`);
          setDragging(false);
        });
      }
    };
    const up = (e: PointerEvent) => {
      sliding.current = false;
      if (e.button !== 0) return;
      const p = press.current;
      press.current = null;
      if (p && !dragging && p.id) commands.openProviderPage(p.id).then((r) => r.status === "error" && showNotice(r.error));
    };
    document.addEventListener("pointermove", move);
    document.addEventListener("pointerup", up);
    return () => {
      document.removeEventListener("pointermove", move);
      document.removeEventListener("pointerup", up);
    };
  }, [dragging, hide, showNotice]);

  // ---- scale -----------------------------------------------------------------------------------
  const setScale = (v: number) => {
    sliding.current = true;
    setData((d) => ({ ...d, scale: v }));
    window.clearTimeout(scaleTimer.current);
    scaleTimer.current = window.setTimeout(() => commands.setScale(v / 100).catch(() => {}), 150);
  };

  if (!ready) return <NoticeArea notice={notice} />;

  const shown = cells(data);
  const snapshotOf = (id: string) => data.usage.find((u) => u.provider === id)?.snapshot;
  const card = hover && snapshotOf(hover.id);
  const now = Date.now();

  return (
    <div ref={root} className="fixed inset-0 overflow-hidden select-none" onPointerLeave={() => open && scheduleHide()}>
      <div
        ref={shell}
        className="absolute top-1/2 right-0 -translate-y-1/2"
        onPointerEnter={() => {
          keep();
          setAtShell(true);
        }}
        onPointerLeave={() => scheduleHide()}
        onPointerDown={(e) => {
          if (e.button !== 0) return;
          const cell = (e.target as HTMLElement).closest<HTMLElement>("[data-provider]");
          press.current = { x: e.clientX, y: e.clientY, id: (cell?.dataset.provider as ProviderId) ?? hover?.id ?? null };
        }}
      >
        <NotchShell collapsed={collapsed} scale={data.scale / 100} instant={first.current}>
          {shown.map((s) => {
            const id = s.provider as ProviderId;
            const snapshot = snapshotOf(id);
            if (!snapshot) return null;
            return (
              <div key={id} onPointerEnter={(e) => enterCell(id, e.currentTarget)}>
                <ProviderCell provider={id} snapshot={snapshot} windowId={s.window} glyph={data.glyphs[id]} activity={activityOf(data, id)} active={hover?.id === id} />
              </div>
            );
          })}
        </NotchShell>
      </div>

      <div ref={cardWrap} onPointerEnter={keep} onPointerLeave={() => scheduleHide()}>
        <HoverCard open={!!card} anchorY={hover?.y ?? 230}>
          {hover && card && (
            <>
              <CardHeader name={providerName[hover.id]} glyph={data.glyphs[hover.id]} fallback={hover.id === "gemini" ? "Ag" : hover.id[0].toUpperCase()} note={card.note} />
              {card.windows.map((w) => (
                <LimitWindowBlock key={w.id} window={w} now={now} />
              ))}
              <SessionList
                sessions={hover.id === "claude" ? data.sessions.sessions.filter((s) => s.state !== "idle") : []}
                activity={hover.id === "claude" ? [] : data.activity.filter((a) => a.provider === hover.id)}
                onFocus={(id) => commands.focusSession(id).catch(() => {})}
              />
              <ScaleSlider value={data.scale} onChange={setScale} />
            </>
          )}
        </HoverCard>
      </div>

      <NoticeArea notice={notice} />
    </div>
  );
}

/** One line at the bottom of the window, above everything else. */
function NoticeArea({ notice }: { notice: string | null }) {
  return (
    <div className="pointer-events-none fixed inset-x-2 bottom-2 flex justify-end">
      <NoticeToast message={notice} />
    </div>
  );
}
