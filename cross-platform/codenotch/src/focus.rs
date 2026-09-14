//! Jump back to the right terminal: from the session's Claude CLI process PID, walk the parent chain
//! to the hosting terminal window, then SetForegroundWindow + FlashWindowEx. Returns false on failure (the page reports it).

#[cfg(windows)]
pub fn focus_terminal(claude_pid: u32) -> bool {
    use std::collections::HashMap;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, FlashWindowEx, GetWindowTextLengthW, GetWindowThreadProcessId, IsIconic,
        IsWindowVisible, SetForegroundWindow, ShowWindow, FLASHWINFO, FLASHW_ALL, SW_RESTORE,
    };

    if claude_pid == 0 {
        return false;
    }

    // 1) Full pid -> ppid snapshot
    let mut ppid_map: HashMap<u32, u32> = HashMap::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return false;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                ppid_map.insert(entry.th32ProcessID, entry.th32ParentProcessID);
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = windows::Win32::Foundation::CloseHandle(snap);
    }

    // 2) claude's ancestor chain (itself included), at most 8 levels: node → shell → WindowsTerminal/conhost host…
    let mut chain: Vec<u32> = vec![claude_pid];
    let mut cur = claude_pid;
    for _ in 0..8 {
        match ppid_map.get(&cur) {
            Some(&p) if p != 0 && !chain.contains(&p) => {
                chain.push(p);
                cur = p;
            }
            _ => break,
        }
    }

    // 3) Enumerate visible top-level windows
    struct Cand {
        hwnd: isize,
        pid: u32,
    }
    let mut wins: Vec<Cand> = Vec::new();
    unsafe extern "system" fn cb(hwnd: HWND, l: LPARAM) -> BOOL {
        let v = &mut *(l.0 as *mut Vec<(isize, u32)>);
        if IsWindowVisible(hwnd).as_bool() && GetWindowTextLengthW(hwnd) > 0 {
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            v.push((hwnd.0 as isize, pid));
        }
        BOOL(1)
    }
    let mut raw: Vec<(isize, u32)> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(&mut raw as *mut _ as isize));
    }
    for (h, p) in raw {
        wins.push(Cand { hwnd: h, pid: p });
    }

    // 4) Score: the window's PID is on the ancestor chain (higher up = the real terminal host = higher
    //    score), or the window PID's parent is on the chain (the classic conhost case).
    let score_of = |pid: u32| -> Option<usize> {
        if let Some(i) = chain.iter().position(|&c| c == pid) {
            return Some(i);
        }
        if let Some(&pp) = ppid_map.get(&pid) {
            if let Some(i) = chain.iter().position(|&c| c == pp) {
                return Some(i);
            }
        }
        None
    };
    let best = wins
        .iter()
        .filter_map(|w| score_of(w.pid).map(|s| (s, w.hwnd)))
        .max_by_key(|(s, _)| *s);

    let Some((_, hwnd_raw)) = best else {
        return false;
    };
    unsafe {
        let hwnd = HWND(hwnd_raw as *mut core::ffi::c_void);
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = SetForegroundWindow(hwnd);
        let fi = FLASHWINFO {
            cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
            hwnd,
            dwFlags: FLASHW_ALL,
            uCount: 2,
            dwTimeout: 0,
        };
        let _ = FlashWindowEx(&fi);
    }
    true
}

#[cfg(not(windows))]
pub fn focus_terminal(claude_pid: u32) -> bool {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{
        ClientMessageEvent, ConnectionExt as XprotoConnectionExt, EventMask, Window as XWindow,
    };
    use x11rb::rust_connection::RustConnection;

    if claude_pid == 0 {
        return false;
    }

    let maps = proc_maps();
    let chain = chain_of(claude_pid, &maps.ppid);

    let Ok((conn, screen_num)) = RustConnection::connect(None) else {
        return false;
    };
    let root = conn.setup().roots[screen_num].root;
    // Ask the WM to raise+focus; a direct XSetInputFocus can be ignored by modern WMs.

    // Higher index = further up the ancestor chain = the real terminal host.
    let score_of = |pid: u32| -> Option<usize> {
        if let Some(i) = chain.iter().position(|&c| c == pid) {
            return Some(i);
        }
        maps.ppid
            .get(&pid)
            .and_then(|&pp| chain.iter().position(|&c| c == pp))
    };

    let target = (|| -> Option<XWindow> {
        let client_list_atom = conn
            .intern_atom(false, b"_NET_CLIENT_LIST")
            .ok()?
            .reply()
            .ok()?
            .atom;
        let window_atom = conn.intern_atom(false, b"WINDOW").ok()?.reply().ok()?.atom;
        let reply = conn
            .get_property(false, root, client_list_atom, window_atom, 0, 0xFFFF)
            .ok()?
            .reply()
            .ok()?;
        let windows: Vec<XWindow> = reply.value32()?.collect();

        let pid_atom = conn
            .intern_atom(false, b"_NET_WM_PID")
            .ok()?
            .reply()
            .ok()?
            .atom;
        let cardinal_atom = conn
            .intern_atom(false, b"CARDINAL")
            .ok()?
            .reply()
            .ok()?
            .atom;

        windows
            .iter()
            .filter_map(|&win| {
                let reply = conn
                    .get_property(false, win, pid_atom, cardinal_atom, 0, 1)
                    .ok()?
                    .reply()
                    .ok()?;
                let pid = reply.value32().and_then(|mut v| v.next())?;
                score_of(pid).map(|s| (s, win))
            })
            .max_by_key(|(s, _)| *s)
            .map(|(_, win)| win)
    })();

    let Some(target) = target else {
        return false;
    };

    let Some(active_atom) = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| r.atom)
    else {
        return false;
    };
    let event = ClientMessageEvent::new(32, target, active_atom, [1u32, 0, 0, 0, 0]);
    if conn
        .send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )
        .is_err()
    {
        return false;
    }
    let _ = conn.flush();
    true
}

// ---------------- Process and foreground helpers shared by seen-clears-it and the desktop jump-back ----------------

pub struct ProcMaps {
    pub ppid: std::collections::HashMap<u32, u32>,
    pub name: std::collections::HashMap<u32, String>, // lower-case exe name
}

#[cfg(windows)]
pub fn proc_maps() -> ProcMaps {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    let mut m = ProcMaps {
        ppid: Default::default(),
        name: Default::default(),
    };
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return m;
        };
        let mut e = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut e).is_ok() {
            loop {
                m.ppid.insert(e.th32ProcessID, e.th32ParentProcessID);
                let len = e.szExeFile.iter().position(|&c| c == 0).unwrap_or(260);
                m.name.insert(
                    e.th32ProcessID,
                    String::from_utf16_lossy(&e.szExeFile[..len]).to_lowercase(),
                );
                if Process32NextW(snap, &mut e).is_err() {
                    break;
                }
            }
        }
        let _ = windows::Win32::Foundation::CloseHandle(snap);
    }
    m
}

#[cfg(windows)]
pub fn fg_pid() -> u32 {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return 0;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid
    }
}

#[cfg(not(windows))]
pub fn proc_maps() -> ProcMaps {
    let mut m = ProcMaps {
        ppid: Default::default(),
        name: Default::default(),
    };
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return m;
    };
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let pid_str = file_name.to_string_lossy();
        let Ok(pid): Result<u32, _> = pid_str.parse() else {
            continue;
        };
        let status_path = entry.path().join("status");
        let Ok(status) = std::fs::read_to_string(&status_path) else {
            continue;
        };
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("PPid:") {
                if let Ok(ppid) = rest.trim().parse::<u32>() {
                    m.ppid.insert(pid, ppid);
                }
                break;
            }
        }
        // Prefer the exe basename; fall back to comm if the process is unreadable.
        let exe_path = entry.path().join("exe");
        if let Ok(target) = std::fs::read_link(&exe_path) {
            if let Some(fname) = target.file_name() {
                m.name.insert(pid, fname.to_string_lossy().to_lowercase());
                continue;
            }
        }
        if let Ok(comm) = std::fs::read_to_string(entry.path().join("comm")) {
            m.name.insert(pid, comm.trim().to_lowercase());
        }
    }
    m
}

#[cfg(not(windows))]
pub fn fg_pid() -> u32 {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::ConnectionExt as XprotoConnectionExt;
    use x11rb::rust_connection::RustConnection;

    let Ok((conn, screen_num)) = RustConnection::connect(None) else {
        return 0;
    };
    let root = conn.setup().roots[screen_num].root;

    (|| -> Option<u32> {
        let active_atom = conn
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")
            .ok()?
            .reply()
            .ok()?
            .atom;
        let window_atom = conn.intern_atom(false, b"WINDOW").ok()?.reply().ok()?.atom;
        let reply = conn
            .get_property(false, root, active_atom, window_atom, 0, 1)
            .ok()?
            .reply()
            .ok()?;
        let active_window = reply.value32().and_then(|mut v| v.next())?;
        let pid_atom = conn
            .intern_atom(false, b"_NET_WM_PID")
            .ok()?
            .reply()
            .ok()?
            .atom;
        let cardinal_atom = conn
            .intern_atom(false, b"CARDINAL")
            .ok()?
            .reply()
            .ok()?
            .atom;
        let reply = conn
            .get_property(false, active_window, pid_atom, cardinal_atom, 0, 1)
            .ok()?
            .reply()
            .ok()?;
        reply.value32().and_then(|mut v| v.next())
    })()
    .unwrap_or(0)
}

pub fn chain_of(pid: u32, ppid: &std::collections::HashMap<u32, u32>) -> Vec<u32> {
    let mut chain = vec![pid];
    let mut cur = pid;
    for _ in 0..8 {
        match ppid.get(&cur) {
            Some(&p) if p != 0 && !chain.contains(&p) => {
                chain.push(p);
                cur = p;
            }
            _ => break,
        }
    }
    chain
}

/// Whether the foreground process belongs to a session's terminal window (itself on the chain, or its parent — the conhost case)
pub fn pid_hits_chain(pid: u32, chain: &[u32], maps: &ProcMaps) -> bool {
    chain.contains(&pid)
        || maps
            .ppid
            .get(&pid)
            .map(|p| chain.contains(p))
            .unwrap_or(false)
}

/// Focus the Claude desktop app's main window (the jump-back target for desktop sessions: the largest visible window whose process name contains claude)
#[cfg(windows)]
pub fn focus_claude_desktop() -> bool {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, FlashWindowEx, GetWindowRect, GetWindowTextLengthW, GetWindowThreadProcessId,
        IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow, FLASHWINFO, FLASHW_ALL,
        SW_RESTORE,
    };
    let maps = proc_maps();
    unsafe extern "system" fn cb(hwnd: HWND, l: LPARAM) -> BOOL {
        let v = &mut *(l.0 as *mut Vec<(isize, u32)>);
        if IsWindowVisible(hwnd).as_bool() && GetWindowTextLengthW(hwnd) > 0 {
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            v.push((hwnd.0 as isize, pid));
        }
        BOOL(1)
    }
    let mut wins: Vec<(isize, u32)> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(&mut wins as *mut _ as isize));
    }
    let mut best: Option<(isize, i64)> = None;
    for (h, pid) in wins {
        let Some(name) = maps.name.get(&pid) else {
            continue;
        };
        if !name.contains("claude") || name.contains("codenotch") {
            continue;
        }
        let mut r = RECT::default();
        let area = unsafe {
            if GetWindowRect(HWND(h as *mut core::ffi::c_void), &mut r).is_ok() {
                ((r.right - r.left) as i64) * ((r.bottom - r.top) as i64)
            } else {
                0
            }
        };
        if best.map(|(_, a)| area > a).unwrap_or(true) {
            best = Some((h, area));
        }
    }
    let Some((h, _)) = best else {
        return false;
    };
    unsafe {
        let hwnd = HWND(h as *mut core::ffi::c_void);
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = SetForegroundWindow(hwnd);
        let fi = FLASHWINFO {
            cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
            hwnd,
            dwFlags: FLASHW_ALL,
            uCount: 2,
            dwTimeout: 0,
        };
        let _ = FlashWindowEx(&fi);
    }
    true
}

#[cfg(not(windows))]
pub fn focus_claude_desktop() -> bool {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{
        ClientMessageEvent, ConnectionExt as XprotoConnectionExt, EventMask, Window as XWindow,
    };
    use x11rb::rust_connection::RustConnection;

    let maps = proc_maps();

    let Ok((conn, screen_num)) = RustConnection::connect(None) else {
        return false;
    };
    let root = conn.setup().roots[screen_num].root;
    // Ask the WM to raise+focus; a direct XSetInputFocus can be ignored by modern WMs.

    let target = (|| -> Option<XWindow> {
        let client_list_atom = conn
            .intern_atom(false, b"_NET_CLIENT_LIST")
            .ok()?
            .reply()
            .ok()?
            .atom;
        let window_atom = conn.intern_atom(false, b"WINDOW").ok()?.reply().ok()?.atom;
        let reply = conn
            .get_property(false, root, client_list_atom, window_atom, 0, 0xFFFF)
            .ok()?
            .reply()
            .ok()?;
        let windows: Vec<XWindow> = reply.value32()?.collect();

        let pid_atom = conn
            .intern_atom(false, b"_NET_WM_PID")
            .ok()?
            .reply()
            .ok()?
            .atom;
        let cardinal_atom = conn
            .intern_atom(false, b"CARDINAL")
            .ok()?
            .reply()
            .ok()?
            .atom;

        windows
            .iter()
            .filter_map(|&win| {
                let reply = conn
                    .get_property(false, win, pid_atom, cardinal_atom, 0, 1)
                    .ok()?
                    .reply()
                    .ok()?;
                let pid = reply.value32().and_then(|mut v| v.next())?;
                let name = maps.name.get(&pid)?;
                if !name.contains("claude") || name.contains("codenotch") {
                    return None;
                }
                let area = match conn.get_geometry(win) {
                    Ok(cookie) => match cookie.reply() {
                        Ok(geo) => (geo.width as i64) * (geo.height as i64),
                        Err(_) => 0,
                    },
                    Err(_) => 0,
                };
                Some((area, win))
            })
            .max_by_key(|(a, _)| *a)
            .map(|(_, win)| win)
    })();

    let Some(target) = target else {
        return false;
    };

    let Some(active_atom) = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| r.atom)
    else {
        return false;
    };
    let event = ClientMessageEvent::new(32, target, active_atom, [1u32, 0, 0, 0, 0]);
    if conn
        .send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )
        .is_err()
    {
        return false;
    }
    let _ = conn.flush();
    true
}
