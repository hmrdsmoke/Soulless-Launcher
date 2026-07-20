// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/network_monitor/bandwidth.rs
// Network bandwidth sampling and up/down history state.

use std::time::Instant;

use crate::network_monitor::HISTORY;

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BandwidthState {
    pub down_history: Vec<f32>,
    pub up_history:   Vec<f32>,
    pub down_kbps:    f32,
    pub up_kbps:      f32,

    prev_rx:   u64,
    prev_tx:   u64,
    prev_time: Option<Instant>,
}

impl Default for BandwidthState {
    fn default() -> Self {
        Self {
            down_history: vec![0.0; HISTORY],
            up_history:   vec![0.0; HISTORY],
            down_kbps:    0.0,
            up_kbps:      0.0,
            prev_rx:      0,
            prev_tx:      0,
            prev_time:    None,
        }
    }
}

impl BandwidthState {
    pub fn new() -> Self {
        let mut s = Self::default();
        if let Some((rx, tx)) = read_net_bytes() {
            s.prev_rx   = rx;
            s.prev_tx   = tx;
            s.prev_time = Some(Instant::now());
        }
        s
    }

    /// Sample the current byte counters and push a new data point.
    pub fn tick(&mut self) {
        let now = Instant::now();

        if let Some((rx, tx)) = read_net_bytes() {
            if let Some(prev) = self.prev_time {
                let elapsed = now.duration_since(prev).as_secs_f32();

                if elapsed > 0.0 {
                    let down = rx.saturating_sub(self.prev_rx) as f32
                        / elapsed / 1024.0;
                    let up = tx.saturating_sub(self.prev_tx) as f32
                        / elapsed / 1024.0;

                    self.down_kbps = down.max(0.0);
                    self.up_kbps   = up.max(0.0);

                    push_capped(&mut self.down_history, self.down_kbps);
                    push_capped(&mut self.up_history,   self.up_kbps);
                }
            }

            self.prev_rx   = rx;
            self.prev_tx   = tx;
            self.prev_time = Some(now);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn push_capped(v: &mut Vec<f32>, value: f32) {
    v.push(value);
    if v.len() > HISTORY {
        v.remove(0);
    }
}

/// Sums RX and TX bytes across all non-loopback interfaces from /proc/net/dev.
fn read_net_bytes() -> Option<(u64, u64)> {
    let data = std::fs::read_to_string("/proc/net/dev").ok()?;
    let mut rx_total: u64 = 0;
    let mut tx_total: u64 = 0;

    for line in data.lines().skip(2) {
        let line  = line.trim();
        // continue, not `?` — one malformed line dropped the entire sample.
        let Some(colon) = line.find(':') else { continue };
        let iface = line[..colon].trim();

        if iface == "lo" {
            continue;
        }
        // Virtual interfaces double-count: VPN traffic appears on tun/wg
        // AND the physical NIC it tunnels through; containers add veth/br
        // noise. Sum physical-ish interfaces only.
        if iface.starts_with("tun") || iface.starts_with("tap")
            || iface.starts_with("wg") || iface.starts_with("docker")
            || iface.starts_with("veth") || iface.starts_with("br-")
            || iface.starts_with("virbr") || iface.starts_with("vnet")
        {
            continue;
        }

        let fields: Vec<&str> = line[colon + 1..]
            .split_whitespace()
            .collect();

        if fields.len() >= 9 {
            rx_total += fields[0].parse::<u64>().unwrap_or(0);
            tx_total += fields[8].parse::<u64>().unwrap_or(0);
        }
    }

    Some((rx_total, tx_total))
}

// === DONE ===
// BandwidthState: rolling HISTORY-sample down/up history :: done
// read_net_bytes(): sums /proc/net/dev, skips loopback :: done
// tick(): computes Kbps delta from prev sample, pushes to histories :: done