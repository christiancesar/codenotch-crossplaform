//! Pure geometry over the hot rectangles the page reports, in window-relative physical pixels.

/// Slack around every hot rectangle: the cursor is sampled on a timer, so arriving at the pill
/// has to count slightly early, or a quick click lands between two polls while the window is
/// still click-through and goes to whatever is behind it.
pub const HOT_PAD: f64 = 10.0;

fn bounding_box(rects: &[[f64; 4]]) -> [f64; 4] {
    let x0 = rects.iter().map(|r| r[0]).fold(f64::MAX, f64::min);
    let y0 = rects.iter().map(|r| r[1]).fold(f64::MAX, f64::min);
    let x1 = rects.iter().map(|r| r[0] + r[2]).fold(f64::MIN, f64::max);
    let y1 = rects.iter().map(|r| r[1] + r[3]).fold(f64::MIN, f64::max);
    [x0, y0, x1 - x0, y1 - y0]
}

/// Is the cursor on something the window is there for? `window` is the outer size, or None when
/// it could not be read. The gap between pill and card counts as inside.
pub fn cursor_in_hot(rects: &[[f64; 4]], lx: f64, ly: f64, window: Option<(f64, f64)>) -> bool {
    if rects.is_empty() {
        return false;
    }
    if let Some((w, h)) = window {
        if !(lx >= 0.0 && ly >= 0.0 && lx < w && ly < h) {
            return false;
        }
    }
    let inside = |r: &[f64; 4], pad: f64| lx >= r[0] - pad && ly >= r[1] - pad && lx < r[0] + r[2] + pad && ly < r[1] + r[3] + pad;
    rects.iter().any(|r| inside(r, HOT_PAD)) || (rects.len() > 1 && inside(&bounding_box(rects), 0.0))
}

/// The input region matching `cursor_in_hot`: each rect padded, plus the bounding box when the
/// card is up so the gap to the pill does not drop the hover.
pub fn input_region(rects: &[[f64; 4]]) -> Vec<[f64; 4]> {
    let mut out: Vec<[f64; 4]> = rects.iter().map(|r| [r[0] - HOT_PAD, r[1] - HOT_PAD, r[2] + 2.0 * HOT_PAD, r[3] + 2.0 * HOT_PAD]).collect();
    if rects.len() > 1 {
        out.push(bounding_box(rects));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real values from a v0.3 run.log: a 2560x1600 display at 150 %.
    const PILL: [f64; 4] = [405.0, 183.5, 105.0, 323.0];
    const CARD: [f64; 4] = [21.0, 142.5, 369.0, 262.0];
    const WINDOW: Option<(f64, f64)> = Some((510.0, 690.0));

    #[test]
    fn pill_card_gap_and_outside() {
        assert!(cursor_in_hot(&[PILL], 450.0, 300.0, WINDOW));
        assert!(cursor_in_hot(&[PILL], 400.0, 300.0, WINDOW), "padding counts");
        assert!(!cursor_in_hot(&[PILL], 200.0, 300.0, WINDOW));
        assert!(cursor_in_hot(&[PILL, CARD], 395.0, 200.0, WINDOW), "the gap between card and pill");
        assert!(!cursor_in_hot(&[PILL, CARD], 600.0, 300.0, WINDOW), "outside the window");
        assert!(!cursor_in_hot(&[], 450.0, 300.0, WINDOW), "nothing reported: click-through");
    }

    #[test]
    fn input_region_pads_each_rect_and_bridges_pill_to_card() {
        assert!(input_region(&[]).is_empty());
        let pill = [322.0, 170.0, 18.0, 120.0];
        assert_eq!(input_region(&[pill]), vec![[312.0, 160.0, 38.0, 140.0]]);
        let r = input_region(&[pill, [21.0, 142.5, 300.0, 262.0]]);
        assert_eq!(r[2], [21.0, 142.5, 319.0, 262.0]);
    }
}
