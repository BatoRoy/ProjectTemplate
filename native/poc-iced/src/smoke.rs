//! `BATO_SMOKE=1`: print `first-frame-ms=<n>` measured from process start
//! (kernel start time from /proc/self/stat, CLOCK_BOOTTIME based) once the
//! first frame has been presented, then exit 0.
//!
//! "Presented" = the second redraw event: on Wayland, winit only delivers the
//! next redraw after the compositor's frame callback for the first frame, so
//! this is the moment the first frame is known to be on the output.

pub fn process_age_ms() -> f64 {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    // Field 22 (starttime) — counted after the `)` that closes comm.
    let after = stat.rsplit_once(')').map(|x| x.1).unwrap_or("");
    let start_ticks: f64 = after.split_whitespace().nth(19).and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut ts) };
    let now_ms = ts.tv_sec as f64 * 1000.0 + ts.tv_nsec as f64 / 1e6;
    now_ms - start_ticks / hz * 1000.0
}

pub fn report() {
    println!("first-frame-ms={:.0}", process_age_ms());
}

/// `BATO_TIMING=1`: startup milestones on stderr.
pub fn mark(what: &str) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *ON.get_or_init(|| std::env::var_os("BATO_TIMING").is_some()) {
        eprintln!("[timing] {:>7.1} ms  {what}", process_age_ms());
    }
}
