//! WebView mouseleave is unreliable in a no-activate transparent window, so the system cursor is
//! watched against the hot rectangles and `pointer_left` fires once it has been outside for a
//! moment. Where the OS needs it (Windows), the same loop toggles whole-window click-through,
//! which is why it runs whether or not the card is open: a click-through window gets no
//! mousemove, so only this loop can see the cursor arriving and hand the input back.

use super::hit_test::cursor_in_hot;
use super::NotchState;
use crate::platform::{Input, Platform, Window};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// Also gates whether a click reaches the notch on Windows; at 150 ms a click in the wrong
/// sample went to the window behind. 25 ms, so the centre widget goes back behind the windows
/// as soon as the pointer has left it.
const TICK: Duration = Duration::from_millis(25);
/// How long outside before the card closes (the user's choice: 300 ms kept the widget over the
/// windows noticeably after the pointer had gone)
const LEAVE_MS: u64 = 100;

pub fn start(w: tauri::WebviewWindow, state: Arc<NotchState>, pointer_left: impl Fn() + Send + 'static) {
    std::thread::Builder::new()
        .name("notch-watchdog".into())
        .spawn(move || {
            let need = (LEAVE_MS / TICK.as_millis() as u64).max(1) as u32;
            let mut miss = 0u32;
            let mut click_through: Option<bool> = None;
            // Taken when the card opens; a focus or desktop change while open closes it at once
            let mut focus_base: Option<(u32, u32)> = None;
            loop {
                std::thread::sleep(TICK);
                let (Ok(pos), Ok(cur)) = (w.outer_position(), w.cursor_position()) else { continue };
                let size = w.outer_size().ok().map(|s| (s.width as f64, s.height as f64));
                let rects = state.hot.lock().unwrap().clone();
                let inside = cursor_in_hot(&rects, cur.x - pos.x as f64, cur.y - pos.y as f64, size);

                if !Platform.shapes_input() && click_through != Some(!inside) {
                    let _ = w.set_ignore_cursor_events(!inside);
                    click_through = Some(!inside);
                }

                if !state.expanded.load(Ordering::Relaxed) {
                    miss = 0;
                    focus_base = None;
                    continue;
                }
                if focus_base.is_none() {
                    focus_base = Platform.focus_signature();
                }
                if let (Some(base), Some(now)) = (focus_base, Platform.focus_signature()) {
                    if now != base {
                        miss = 0;
                        focus_base = None;
                        state.expanded.store(false, Ordering::Relaxed);
                        pointer_left();
                        continue;
                    }
                }
                miss = if inside { 0 } else { miss + 1 };
                if miss >= need {
                    miss = 0;
                    state.expanded.store(false, Ordering::Relaxed);
                    pointer_left();
                }
            }
        })
        .expect("failed to start the notch watchdog");
}
