//! Moving the notch. On an edge it slides along it: up and down on the left and right, sideways on
//! the top and bottom. The centre widget goes anywhere on its screen. The page calls drag_begin
//! once a press on the pill moves more than 4 px (on the widget, as soon as its grip is pressed);
//! from then on this thread follows the system cursor (WebView mousemove is unreliable once the
//! window itself moves) until the left button is released.

use super::NotchState;
use crate::config::NotchEdge;
use crate::platform::{Input, Platform};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// Where a drag left the window: its top-left and its size, both physical
pub struct Moved {
    pub pos: (i32, i32),
    pub size: (u32, u32),
}

/// `mon` is the screen the notch is on: the drag stays on it. `on_end(Some(..))` after a real
/// move, `on_end(None)` for a press that never moved.
pub fn begin(w: tauri::WebviewWindow, mon: tauri::Monitor, state: Arc<NotchState>, edge: NotchEdge, on_end: impl FnOnce(Option<Moved>) + Send + 'static) {
    if state.dragging.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        let start = (w.cursor_position(), w.outer_position(), w.outer_size());
        let (Ok(start_cur), Ok(start_pos), Ok(size)) = start else {
            state.dragging.store(false, Ordering::SeqCst);
            return;
        };
        let mon_pos = (mon.position().x, mon.position().y);
        let mon_size = (mon.size().width, mon.size().height);
        let win = (size.width, size.height);
        let ((x0, x1), (y0, y1)) = super::placement::drag_bounds(edge, mon_pos, mon_size, win);
        // Which axes follow the cursor: both for the widget, the one along the edge otherwise
        let (free_x, free_y) = match edge {
            NotchEdge::Center => (true, true),
            NotchEdge::Top | NotchEdge::Bottom => (true, false),
            NotchEdge::Left | NotchEdge::Right => (false, true),
        };
        let mut last = (start_pos.x, start_pos.y);
        let mut moved = false;
        while Platform.left_button_down() {
            if let Ok(cur) = w.cursor_position() {
                let follow = |from: i32, delta: f64, lo: i32, hi: i32| ((from as f64 + delta).round() as i32).clamp(lo, hi);
                let next = (
                    if free_x { follow(start_pos.x, cur.x - start_cur.x, x0, x1) } else { start_pos.x },
                    if free_y { follow(start_pos.y, cur.y - start_cur.y, y0, y1) } else { start_pos.y },
                );
                if next != last {
                    last = next;
                    moved = true;
                    let _ = w.set_position(tauri::PhysicalPosition::new(next.0, next.1));
                }
            }
            std::thread::sleep(Duration::from_millis(8));
        }
        state.dragging.store(false, Ordering::SeqCst);
        on_end(moved.then_some(Moved { pos: last, size: win }));
    });
}
