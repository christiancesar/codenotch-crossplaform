use crate::platform::ProcMaps;
use ::windows::Win32::Foundation::CloseHandle;
use ::windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

pub fn proc_maps() -> ProcMaps {
    let mut m = ProcMaps { ppid: Default::default(), name: Default::default() };
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return m };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        if Process32FirstW(snap, &mut e).is_ok() {
            loop {
                m.ppid.insert(e.th32ProcessID, e.th32ParentProcessID);
                let len = e.szExeFile.iter().position(|&c| c == 0).unwrap_or(e.szExeFile.len());
                m.name.insert(e.th32ProcessID, String::from_utf16_lossy(&e.szExeFile[..len]).to_lowercase());
                if Process32NextW(snap, &mut e).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    m
}

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Runs a console tool without flashing a window and returns its stdout.
fn run_hidden(program: &str, args: &[&str]) -> String {
    use std::os::windows::process::CommandExt;
    std::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// One CIM query: "pid<TAB>commandline" per matching process.
pub fn command_lines(needle: &str) -> Vec<(u32, String)> {
    let needle: String = needle.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').collect();
    let query = format!(
        "Get-CimInstance Win32_Process -Filter \"Name LIKE '%{needle}%'\" | ForEach-Object {{ \"$($_.ProcessId)`t$($_.CommandLine)\" }}"
    );
    run_hidden("powershell", &["-NoProfile", "-NonInteractive", "-Command", &query])
        .lines()
        .filter_map(|l| {
            let (pid, cmd) = l.split_once('\t')?;
            Some((pid.trim().parse().ok()?, cmd.trim().to_string()))
        })
        .collect()
}

/// `netstat -ano -p TCP` rows: `TCP 127.0.0.1:PORT 0.0.0.0:0 LISTENING PID`.
pub fn listening_ports(pid: u32) -> Vec<u16> {
    parse_netstat(&run_hidden("netstat", &["-ano", "-p", "TCP"]), pid)
}

fn parse_netstat(out: &str, pid: u32) -> Vec<u16> {
    let pid_s = pid.to_string();
    let mut ports: Vec<u16> = out
        .lines()
        .filter(|l| l.contains("LISTENING"))
        .filter_map(|l| {
            let cols: Vec<&str> = l.split_whitespace().collect();
            if cols.len() < 5 || cols[4] != pid_s {
                return None;
            }
            cols[1].rsplit(':').next()?.parse::<u16>().ok()
        })
        .collect();
    ports.sort_unstable();
    ports.dedup();
    ports
}

#[cfg(test)]
mod tests {
    #[test]
    fn netstat_rows_for_one_pid() {
        let out = "  TCP    127.0.0.1:52100    0.0.0.0:0    LISTENING    4242\n  TCP    127.0.0.1:52099    0.0.0.0:0    LISTENING    4242\n  TCP    127.0.0.1:52100    0.0.0.0:0    LISTENING    4242\n  TCP    0.0.0.0:445    0.0.0.0:0    LISTENING    4\n  TCP    127.0.0.1:52101    127.0.0.1:443    ESTABLISHED    4242\n";
        assert_eq!(super::parse_netstat(out, 4242), vec![52099, 52100]);
    }
}
