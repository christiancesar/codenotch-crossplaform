//! Native Wayland ignores window positions (mutter centres the window) and GNOME has no
//! layer-shell to anchor an edge window, so the app runs under XWayland. There the X pointer
//! freezes while the cursor is over a native Wayland surface, so a fully click-through notch never
//! learns the cursor arrived: the input region is shaped to the hot rectangles instead, and the
//! compositor routes the pointer to the pill itself.

/// `CODENOTCH_WAYLAND=1` opts out, to try native Wayland.
pub fn pick_gdk_backend(display: Option<&str>, opt_out: Option<&str>) -> Option<&'static str> {
    if opt_out == Some("1") {
        return None;
    }
    display.filter(|d| !d.is_empty()).map(|_| "x11")
}

pub fn prepare_process() {
    let display = std::env::var("DISPLAY").ok();
    let opt_out = std::env::var("CODENOTCH_WAYLAND").ok();
    if let Some(backend) = pick_gdk_backend(display.as_deref(), opt_out.as_deref()) {
        // Single-threaded here: main calls this first
        std::env::set_var("GDK_BACKEND", backend);
    }
}

pub fn no_activate(w: &tauri::WebviewWindow) {
    use gtk::prelude::{GtkWindowExt, WidgetExt};
    let Ok(gw) = w.gtk_window() else { return };
    if !gw.is_realized() {
        gw.realize();
    }
    gw.set_accept_focus(false);
}

pub fn set_input_region(w: &tauri::WebviewWindow, rects: Vec<[f64; 4]>) {
    let w = w.clone();
    let _ = w.clone().run_on_main_thread(move || {
        use gtk::prelude::WidgetExt;
        let Ok(gw) = w.gtk_window() else { return };
        // Hot rects are physical px; GDK works in logical px
        let k = gw.scale_factor().max(1) as f64;
        let cells: Vec<gtk::cairo::RectangleInt> = rects
            .iter()
            .map(|r| {
                gtk::cairo::RectangleInt::new(
                    (r[0] / k).floor().max(0.0) as i32,
                    (r[1] / k).floor().max(0.0) as i32,
                    (r[2] / k).ceil() as i32,
                    (r[3] / k).ceil() as i32,
                )
            })
            .collect();
        gw.input_shape_combine_region(Some(&gtk::cairo::Region::create_rectangles(&cells)));
    });
}

#[cfg(test)]
mod tests {
    use super::pick_gdk_backend;

    #[test]
    fn xwayland_when_an_x_display_exists_unless_opted_out() {
        assert_eq!(pick_gdk_backend(Some(":0"), None), Some("x11"));
        assert_eq!(pick_gdk_backend(Some(":0"), Some("0")), Some("x11"));
        assert_eq!(pick_gdk_backend(None, None), None);
        assert_eq!(pick_gdk_backend(Some(""), None), None);
        assert_eq!(pick_gdk_backend(Some(":0"), Some("1")), None);
    }
}
