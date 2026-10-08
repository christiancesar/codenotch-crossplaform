//! Drag along the edge: up and down on the left and right edges, sideways on the top and bottom.
//! The page calls drag_begin once a press on the pill moves more than 4 px; from then on this
//! thread follows the system cursor (WebView mousemove is unreliable once the window itself moves)
//! until the left button is released.

use super::NotchState;
use crate::config::NotchEdge;
use crate::platform::{Input, Platform};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// `on_end(Some(ratio))` after a real move, `on_end(None)` for a press that never moved.
/// `mon` is the screen the notch is on: the drag stays on it.
pub fn begin(w: tauri::WebviewWindow, mon: tauri::Monitor, state: Arc<NotchState>, edge: NotchEdge, on_end: impl FnOnce(Option<f64>) + Send + 'static) {
    if state.dragging.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        let start = (w.cursor_position(), w.outer_position(), w.outer_size());
        let (Ok(start_cur), Ok(start_pos), Ok(size)) = start else {
            state.dragging.store(false, Ordering::SeqCst);
            return;
        };
        let row = edge.is_horizontal();
        // The axis along the edge: where the window may start on it, and where it starts now
        let (lo, hi, from) = if row {
            (mon.position().x, mon.position().x + (mon.size().width as i32 - size.width as i32).max(0), start_pos.x)
        } else {
            (mon.position().y, mon.position().y + (mon.size().height as i32 - size.height as i32).max(0), start_pos.y)
        };
        let mut last = from;
        let mut moved = false;
        while Platform.left_button_down() {
            if let Ok(cur) = w.cursor_position() {
                let delta = if row { cur.x - start_cur.x } else { cur.y - start_cur.y };
                let n = ((from as f64 + delta).round() as i32).clamp(lo, hi);
                if n != last {
                    last = n;
                    moved = true;
                    let p = if row { (n, start_pos.y) } else { (start_pos.x, n) };
                    let _ = w.set_position(tauri::PhysicalPosition::new(p.0, p.1));
                }
            }
            std::thread::sleep(Duration::from_millis(8));
        }
        state.dragging.store(false, Ordering::SeqCst);
        let pos = if row { (last, start_pos.y) } else { (start_pos.x, last) };
        let mon_pos = (mon.position().x, mon.position().y);
        let mon_size = (mon.size().width, mon.size().height);
        on_end(moved.then(|| super::placement::ratio_after_drag(edge, pos, (size.width, size.height), mon_pos, mon_size)));
    });
}
