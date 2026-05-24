// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

const PING_HOST:  &str = "1.1.1.1";
const PING_COUNT: &str = "4";
const PING_WAIT:  &str = "1";

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct PingState {
    /// Last measured round-trip time in ms (0 = not yet measured / unreachable)
    pub ping_ms:   f32,
    /// Last measured jitter (mdev) in ms
    pub jitter_ms: f32,
}

impl PingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, ping_ms: f32, jitter_ms: f32) {
        self.ping_ms   = ping_ms;
        self.jitter_ms = jitter_ms;
    }
}

// ── Measurement ───────────────────────────────────────────────────────────────

/// Runs `ping -c 4 -W 1 1.1.1.1` synchronously and parses avg RTT + mdev.
///
/// Returns `(ping_ms, jitter_ms)`.  Both are 0.0 if the host is unreachable
/// or the output cannot be parsed.
///
/// This is intended to be called from a background subscription tick so it
/// does not block the UI thread.
pub fn measure() -> (f32, f32) {
    let output = std::process::Command::new("ping")
        .args(["-c", PING_COUNT, "-W", PING_WAIT, PING_HOST])
        .output();

    let output = match output {
        Ok(o)  => o,
        Err(_) => return (0.0, 0.0),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Typical line: "rtt min/avg/max/mdev = 1.234/2.345/3.456/0.500 ms"
    for line in stdout.lines() {
        if line.contains("rtt") && line.contains("mdev") {
            let after_eq = line.split('=').nth(1).unwrap_or("").trim();

            let parts: Vec<&str> = after_eq.split('/').collect();

            if parts.len() >= 4 {
                let avg  = parts[1].trim().parse::<f32>().unwrap_or(0.0);
                let mdev = parts[3]
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse::<f32>()
                    .unwrap_or(0.0);
                return (avg, mdev);
            }
        }
    }

    (0.0, 0.0)
}

// === DONE ===
// PingState: ping_ms + jitter_ms, updated via update() :: done
// measure(): shells ping -c 4 -W 1 1.1.1.1, parses rtt/mdev line :: done