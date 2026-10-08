//! Two background passes over the store: the stale sweep, and seen-clears-it (a done session
//! whose terminal, or the Claude desktop app, is in front counts as acknowledged).

use super::Store;
use crate::platform::{chain_of, pid_hits_chain, Focus, Platform, Processes};
use crate::support::time::now_ms;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// What was in front at the previous scan.
#[derive(Default)]
pub struct Front {
    pid: u32,
    claude_desktop: bool,
}

pub fn ack_scan(store: &Mutex<Store>, platform: &(impl Focus + Processes), front: &mut Front) -> bool {
    let fg = platform.foreground_pid();
    if fg == 0 {
        return false;
    }
    let has_done = store.lock().unwrap().has_done();
    // The front is followed even with nothing done, or a session finishing while you are already
    // in the Claude app would read as you having just switched to it
    if !has_done && fg == front.pid {
        return false;
    }
    let maps = platform.proc_maps();
    let was_claude_desktop = front.claude_desktop;
    if fg != front.pid {
        let name = maps.name.get(&fg).cloned().unwrap_or_default();
        *front = Front { pid: fg, claude_desktop: name.contains("claude") && !name.contains("codenotch") };
    }
    if !has_done {
        return false;
    }
    // A desktop session has no process to match, and the Claude app's window does not say which
    // conversation is open. Being in the app is not enough: on Windows you work inside it, and
    // every session would vanish the moment it finished. Switching to it after it finished is.
    let entered_claude_desktop = front.claude_desktop && !was_claude_desktop;
    store.lock().unwrap().ack_done(now_ms(), |s| {
        if s.ppid == 0 {
            entered_claude_desktop
        } else {
            pid_hits_chain(fg, &chain_of(s.ppid, &maps.ppid), &maps)
        }
    })
}

pub fn start(store: Arc<Mutex<Store>>, on_change: Arc<dyn Fn() + Send + Sync>) {
    let (s, c) = (store.clone(), on_change.clone());
    std::thread::Builder::new()
        .name("sessions-ack".into())
        .spawn(move || {
            let mut front = Front::default();
            loop {
                std::thread::sleep(Duration::from_millis(1500));
                if ack_scan(&s, &Platform, &mut front) {
                    c();
                }
            }
        })
        .expect("failed to start the seen-clears-it scan");
    std::thread::Builder::new()
        .name("sessions-sweep".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(30));
            if store.lock().unwrap().sweep(now_ms()) {
                on_change();
            }
        })
        .expect("failed to start the session sweep");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::ProcMaps;
    use crate::sessions::store::SessionState;
    use crate::sessions::{HookEvent, Source};

    struct Fake {
        fg: u32,
    }
    impl Focus for Fake {
        fn focus_terminal(&self, _: u32) -> bool {
            false
        }
        fn focus_claude_desktop(&self) -> bool {
            false
        }
        fn foreground_pid(&self) -> u32 {
            self.fg
        }
    }
    impl Processes for Fake {
        fn proc_maps(&self) -> ProcMaps {
            // claude 100 -> shell 50 -> terminal 10; 300 is the Claude desktop app
            ProcMaps {
                ppid: [(100, 50), (50, 10)].into_iter().collect(),
                name: [(10, "kitty"), (300, "claude")].into_iter().map(|(p, n)| (p, n.to_string())).collect(),
            }
        }
        fn command_lines(&self, _: &str) -> Vec<(u32, String)> {
            vec![]
        }
        fn listening_ports(&self, _: u32) -> Vec<u16> {
            vec![]
        }
        fn lower_current_thread_priority(&self) {}
        fn io_counters(&self, _: u32) -> Option<(u64, u64)> {
            None
        }
    }

    fn done(id: &str, ppid: u32) -> HookEvent {
        HookEvent { e: "done".into(), session_id: id.into(), ppid, cwd: "/p".into(), prompt: String::new(), message: String::new(), tool_name: String::new(), tool_cmd: String::new(), model: String::new(), src: Source::Hook }
    }

    fn state_of(store: &Mutex<Store>, id: &str) -> SessionState {
        store.lock().unwrap().snapshot().sessions.iter().find(|s| s.id == id).unwrap().state
    }

    #[test]
    fn the_session_whose_terminal_is_in_front_is_acknowledged() {
        let store = Mutex::new(Store::default());
        let mut front = Front::default();
        store.lock().unwrap().apply(done("cli", 100), 0);
        store.lock().unwrap().apply(done("desk", 0), 0);
        assert!(!ack_scan(&store, &Fake { fg: 999 }, &mut front));
        assert!(ack_scan(&store, &Fake { fg: 10 }, &mut front));
        assert_eq!((state_of(&store, "cli"), state_of(&store, "desk")), (SessionState::Idle, SessionState::Done));
        assert!(ack_scan(&store, &Fake { fg: 300 }, &mut front));
        assert_eq!(state_of(&store, "desk"), SessionState::Idle);
    }

    #[test]
    fn a_desktop_session_finishing_while_you_are_in_the_claude_app_stays_done() {
        let store = Mutex::new(Store::default());
        let mut front = Front::default();
        // Working inside the Claude app, nothing done yet
        assert!(!ack_scan(&store, &Fake { fg: 300 }, &mut front));
        store.lock().unwrap().apply(done("desk", 0), 0);
        assert!(!ack_scan(&store, &Fake { fg: 300 }, &mut front));
        assert!(!ack_scan(&store, &Fake { fg: 300 }, &mut front));
        assert_eq!(state_of(&store, "desk"), SessionState::Done);
        // Away and back: now it was seen
        assert!(!ack_scan(&store, &Fake { fg: 999 }, &mut front));
        assert!(ack_scan(&store, &Fake { fg: 300 }, &mut front));
        assert_eq!(state_of(&store, "desk"), SessionState::Idle);
    }
}
