use crate::platform::ProcMaps;

pub fn proc_maps() -> ProcMaps {
    let mut m = ProcMaps { ppid: Default::default(), name: Default::default() };
    let Ok(entries) = std::fs::read_dir("/proc") else { return m };
    for entry in entries.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
        let Ok(status) = std::fs::read_to_string(entry.path().join("status")) else { continue };
        if let Some(ppid) = status
            .lines()
            .find_map(|l| l.strip_prefix("PPid:"))
            .and_then(|r| r.trim().parse::<u32>().ok())
        {
            m.ppid.insert(pid, ppid);
        }
        // The exe basename is exact; comm is truncated to 15 bytes, so only a fallback for
        // processes whose exe link we may not read.
        let name = std::fs::read_link(entry.path().join("exe"))
            .ok()
            .and_then(|t| t.file_name().map(|f| f.to_string_lossy().to_lowercase()))
            .or_else(|| {
                std::fs::read_to_string(entry.path().join("comm")).ok().map(|c| c.trim().to_lowercase())
            });
        if let Some(name) = name {
            m.name.insert(pid, name);
        }
    }
    m
}

#[cfg(test)]
mod tests {
    #[test]
    fn sees_this_process_and_its_parent() {
        let m = super::proc_maps();
        let me = std::process::id();
        assert_eq!(m.ppid.get(&me), Some(&std::os::unix::process::parent_id()));
        assert!(m.name.contains_key(&me));
    }
}
