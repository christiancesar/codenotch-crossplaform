//! The screens the notch can sit on, and which one it is on: the stored choice while that screen
//! is connected, the primary otherwise. The choice is never dropped for a screen that is away (a
//! laptop off its dock): the notch goes back to it when it returns.

use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Monitor};

/// A connected screen, in physical pixels on the virtual desktop, the way the OS arranges them.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct MonitorInfo {
    /// The OS's name for it (\\.\DISPLAY2 on Windows, the output name on Linux): what is stored
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    /// Display scaling, 1 = 100 %
    pub scale: f64,
    pub primary: bool,
}

/// The OS's name, or the position when it gives none (still stable while nothing is rearranged)
fn id_of(m: &Monitor) -> String {
    m.name().cloned().unwrap_or_else(|| format!("{},{}", m.position().x, m.position().y))
}

/// Which of `ids` the notch goes on. Pure, so the fallbacks are testable without screens.
fn pick(ids: &[String], primary: Option<&str>, stored: Option<&str>) -> Option<usize> {
    let find = |id: &str| ids.iter().position(|i| i == id);
    stored.and_then(find).or_else(|| primary.and_then(find)).or(if ids.is_empty() { None } else { Some(0) })
}

fn all(app: &AppHandle) -> (Vec<Monitor>, Option<String>) {
    let monitors = app.available_monitors().unwrap_or_default();
    let primary = app.primary_monitor().ok().flatten().map(|m| id_of(&m));
    (monitors, primary)
}

pub fn list(app: &AppHandle) -> Vec<MonitorInfo> {
    let (monitors, primary) = all(app);
    monitors
        .iter()
        .map(|m| {
            let id = id_of(m);
            MonitorInfo {
                primary: primary.as_deref() == Some(id.as_str()),
                id,
                x: m.position().x,
                y: m.position().y,
                width: m.size().width,
                height: m.size().height,
                scale: m.scale_factor(),
            }
        })
        .collect()
}

/// The screen the notch goes on for the `stored` choice.
pub fn chosen(app: &AppHandle, stored: Option<&str>) -> Option<Monitor> {
    let (monitors, primary) = all(app);
    let ids: Vec<String> = monitors.iter().map(id_of).collect();
    pick(&ids, primary.as_deref(), stored).map(|i| monitors[i].clone())
}

#[cfg(test)]
mod tests {
    use super::pick;

    #[test]
    fn the_stored_screen_while_connected_then_the_primary() {
        let ids: Vec<String> = [r"\\.\DISPLAY1", r"\\.\DISPLAY2"].map(String::from).to_vec();
        assert_eq!(pick(&ids, Some(r"\\.\DISPLAY1"), Some(r"\\.\DISPLAY2")), Some(1));
        assert_eq!(pick(&ids, Some(r"\\.\DISPLAY1"), None), Some(0));
        // Off its dock: the primary stands in
        assert_eq!(pick(&ids, Some(r"\\.\DISPLAY2"), Some(r"\\.\DISPLAY3")), Some(1));
        // No primary reported: the first screen
        assert_eq!(pick(&ids, None, Some(r"\\.\DISPLAY9")), Some(0));
        assert_eq!(pick(&[], None, None), None);
    }
}
