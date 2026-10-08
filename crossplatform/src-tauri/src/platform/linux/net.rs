//! Listening ports of a process from /proc, so nothing depends on lsof being installed.

use std::collections::HashSet;

/// Socket inodes in LISTEN state (st 0A) from /proc/net/tcp-style text, with their local port.
pub fn listening_inodes(table: &str) -> Vec<(u64, u16)> {
    table
        .lines()
        .skip(1)
        .filter_map(|l| {
            let cols: Vec<&str> = l.split_whitespace().collect();
            if cols.len() < 10 || cols[3] != "0A" {
                return None;
            }
            let port = u16::from_str_radix(cols[1].rsplit(':').next()?, 16).ok()?;
            Some((cols[9].parse().ok()?, port))
        })
        .collect()
}

/// Inodes of the sockets `pid` holds open (`socket:[N]` fd links).
fn socket_inodes(pid: u32) -> HashSet<u64> {
    std::fs::read_dir(format!("/proc/{pid}/fd"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| std::fs::read_link(e.path()).ok())
        .filter_map(|t| t.to_string_lossy().strip_prefix("socket:[")?.strip_suffix(']')?.parse().ok())
        .collect()
}

pub fn listening_ports(pid: u32) -> Vec<u16> {
    let mine = socket_inodes(pid);
    let mut ports: Vec<u16> = ["/proc/net/tcp", "/proc/net/tcp6"]
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .flat_map(|t| listening_inodes(&t))
        .filter(|(inode, _)| mine.contains(inode))
        .map(|(_, port)| port)
        .collect();
    ports.sort_unstable();
    ports.dedup();
    ports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_listen_rows_only() {
        let table = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n   0: 0100007F:BDE2 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 424242 1 0000000000000000 100 0 0 10 0\n   1: 0100007F:BDE3 0100007F:1F90 01 00000000:00000000 00:00000000 00000000  1000        0 515151 1 0000000000000000 20 4 30 10 -1\n";
        assert_eq!(listening_inodes(table), vec![(424242, 0xBDE2)]);
    }

    #[test]
    fn finds_a_port_this_process_listens_on() {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        assert!(listening_ports(std::process::id()).contains(&port));
    }
}
