//! Vertical drag along the edge. The page calls drag_begin once a press on the pill moves more
//! than 4 px; from then on this thread follows the system cursor (WebView mousemove is unreliable
//! once the window itself moves) until the left button is released.

use super::NotchState;
use crate::platform::{Input, Platform};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// `on_end(Some(ratio))` after a real move, `on_end(None)` for a press that never moved.
pub fn begin(w: tauri::WebviewWindow, state: Arc<NotchState>, on_end: impl FnOnce(Option<f64>) + Send + 'static) {
    if state.dragging.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        let start = (w.cursor_position(), w.outer_position(), w.outer_size(), w.primary_monitor());
        let (Ok(start_cur), Ok(start_pos), Ok(size), Ok(Some(mon))) = start else {
            state.dragging.store(false, Ordering::SeqCst);
            return;
        };
        let (my, mh) = (mon.position().y, mon.size().height);
        let (lo, hi) = (my, my + (mh as i32 - size.height as i32).max(0));
        let mut last_y = start_pos.y;
        let mut moved = false;
        while Platform.left_button_down() {
            if let Ok(cur) = w.cursor_position() {
                let ny = ((start_pos.y as f64 + (cur.y - start_cur.y)).round() as i32).clamp(lo, hi);
                if ny != last_y {
                    last_y = ny;
                    moved = true;
                    let _ = w.set_position(tauri::PhysicalPosition::new(start_pos.x, ny));
                }
            }
            std::thread::sleep(Duration::from_millis(8));
        }
        state.dragging.store(false, Ordering::SeqCst);
        on_end(moved.then(|| super::placement::ratio_after_drag(last_y, size.height, my, mh)));
    });
}
