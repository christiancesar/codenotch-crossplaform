//! EWMH through x11rb. Raising goes through a _NET_ACTIVE_WINDOW request to the window manager;
//! a direct XSetInputFocus is ignored by modern WMs.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, Window};
use x11rb::rust_connection::RustConnection;

pub struct Ewmh {
    conn: RustConnection,
    root: Window,
}

impl Ewmh {
    pub fn connect() -> Option<Ewmh> {
        let (conn, screen) = RustConnection::connect(None).ok()?;
        let root = conn.setup().roots.get(screen)?.root;
        Some(Ewmh { conn, root })
    }

    fn atom(&self, name: &[u8]) -> Option<Atom> {
        Some(self.conn.intern_atom(false, name).ok()?.reply().ok()?.atom)
    }

    fn u32_property(&self, win: Window, name: &[u8], ty: AtomEnum, len: u32) -> Option<Vec<u32>> {
        let atom = self.atom(name)?;
        let reply = self.conn.get_property(false, win, atom, ty, 0, len).ok()?.reply().ok()?;
        let values: Vec<u32> = reply.value32()?.collect();
        Some(values)
    }

    fn pid_of(&self, win: Window) -> Option<u32> {
        self.u32_property(win, b"_NET_WM_PID", AtomEnum::CARDINAL, 1)?.first().copied()
    }

    /// Managed top-level windows with the pid that owns each.
    pub fn client_windows_with_pid(&self) -> Vec<(Window, u32)> {
        self.u32_property(self.root, b"_NET_CLIENT_LIST", AtomEnum::WINDOW, 0xFFFF)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|w| self.pid_of(w).map(|p| (w, p)))
            .collect()
    }

    pub fn area(&self, win: Window) -> i64 {
        self.conn
            .get_geometry(win)
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|g| g.width as i64 * g.height as i64)
            .unwrap_or(0)
    }

    pub fn active_window_pid(&self) -> Option<u32> {
        let active = *self.u32_property(self.root, b"_NET_ACTIVE_WINDOW", AtomEnum::WINDOW, 1)?.first()?;
        self.pid_of(active)
    }

    pub fn left_button_down(&self) -> bool {
        use x11rb::protocol::xproto::KeyButMask;
        self.conn
            .query_pointer(self.root)
            .ok()
            .and_then(|c| c.reply().ok())
            .is_some_and(|r| KeyButMask::from(u16::from(r.mask)).contains(KeyButMask::BUTTON1))
    }

    pub fn focus_signature(&self) -> Option<(u32, u32)> {
        let active = *self.u32_property(self.root, b"_NET_ACTIVE_WINDOW", AtomEnum::WINDOW, 1)?.first()?;
        let desktop = *self.u32_property(self.root, b"_NET_CURRENT_DESKTOP", AtomEnum::CARDINAL, 1)?.first()?;
        Some((active, desktop))
    }

    pub fn activate(&self, win: Window) -> bool {
        let Some(active) = self.atom(b"_NET_ACTIVE_WINDOW") else { return false };
        // Source indication 1 = a normal application request
        let event = ClientMessageEvent::new(32, win, active, [1u32, 0, 0, 0, 0]);
        let sent = self
            .conn
            .send_event(false, self.root, EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY, event)
            .is_ok();
        sent && self.conn.flush().is_ok()
    }
}
