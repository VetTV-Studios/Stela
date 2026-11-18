//! The pace of the frames the copy path puts on the firmware's
//! framebuffer: a bound GPU's frames copied by the CPU to another
//! adapter's panel (a muxless laptop's panel on the integrated GPU).
//! VetTV GL66 experimental.

/// The rate the copy path runs at in millihertz, VetTV GL66 experimental:
/// `cap_millihertz` (the `--copy-refresh-hz` cap in millihertz, `None`
/// for no cap) when it is nonzero and below the panel's own
/// `panel_millihertz`, so a 144 Hz panel is copied at the cap instead of
/// at its own rate; the panel's rate otherwise, which a cap at or above
/// it never raises. `Some(0)` is no cap, as upstream behaves.
pub fn paced_millihertz(panel_millihertz: u32, cap_millihertz: Option<u32>) -> u32 {
    match cap_millihertz {
        Some(cap) if cap != 0 && cap < panel_millihertz => cap,
        _ => panel_millihertz,
    }
}

/// Nanoseconds in one frame at `millihertz`, VetTV GL66 experimental:
/// `1_000_000_000_000 / millihertz`, rounded to nearest with ties up, in
/// `u64` arithmetic; `None` at 0, which has no frame interval.
pub fn frame_interval_ns(millihertz: u32) -> Option<u64> {
    if millihertz == 0 {
        return None;
    }
    let millihertz = u64::from(millihertz);
    Some((1_000_000_000_000 + millihertz / 2) / millihertz)
}

#[cfg(test)]
mod tests;
