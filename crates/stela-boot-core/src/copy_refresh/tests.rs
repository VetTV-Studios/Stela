use super::{frame_interval_ns, paced_millihertz};
use crate::options::{COPY_REFRESH_HZ, Error, MAX_COPY_REFRESH_HZ, System};
use alloc::format;

#[test]
fn valid_copy_refresh_hz_values_parse() {
    for (hz, cap) in [
        (0, None),
        (1, Some(1_000)),
        (60, Some(60_000)),
        (144, Some(144_000)),
        (480, Some(480_000)),
    ] {
        let line = format!("--system {COPY_REFRESH_HZ} {hz}");
        let options = System::parse(&line).unwrap();
        assert_eq!(options.copy_refresh_hz, Some(hz), "{line}");
        assert_eq!(options.copy_refresh_cap_millihertz(), cap, "{line}");
    }
}

#[test]
fn invalid_copy_refresh_hz_values_are_refused() {
    for line in [
        format!("--system {COPY_REFRESH_HZ}"),
        format!("--system {COPY_REFRESH_HZ} fast"),
        format!("--system {COPY_REFRESH_HZ} -1"),
        format!("--system {COPY_REFRESH_HZ} 60.5"),
        format!("--system {COPY_REFRESH_HZ} 481"),
        format!("--system {COPY_REFRESH_HZ} 4294967296"),
    ] {
        assert_eq!(
            System::parse(&line).unwrap_err(),
            Error::CopyRefresh,
            "{line}"
        );
    }
    let duplicate = format!("--system {COPY_REFRESH_HZ} 60 {COPY_REFRESH_HZ} 72");
    assert_eq!(
        System::parse(&duplicate).unwrap_err(),
        Error::DuplicateCopyRefresh
    );
}

#[test]
fn zero_keeps_the_panels_own_rate() {
    let options = System::parse(&format!("--system {COPY_REFRESH_HZ} 0")).unwrap();
    assert_eq!(options.copy_refresh_cap_millihertz(), None);
    assert_eq!(paced_millihertz(144_149, None), 144_149);
    assert_eq!(paced_millihertz(144_149, Some(0)), 144_149);
    assert_eq!(
        paced_millihertz(144_149, options.copy_refresh_cap_millihertz()),
        144_149
    );
}

#[test]
fn unset_copy_refresh_caps_at_60_hz() {
    let options = System::parse("--system").unwrap();
    assert_eq!(options.copy_refresh_hz, None);
    assert_eq!(options.copy_refresh_cap_millihertz(), Some(60_000));
    assert_eq!(paced_millihertz(144_149, Some(60_000)), 60_000);
    assert_eq!(
        paced_millihertz(144_149, options.copy_refresh_cap_millihertz()),
        60_000
    );
}

#[test]
fn a_cap_at_or_above_the_panel_rate_keeps_the_panel_rate() {
    assert_eq!(paced_millihertz(60_000, Some(72_000)), 60_000);
    assert_eq!(paced_millihertz(144_149, Some(480_000)), 144_149);
    assert_eq!(paced_millihertz(60_000, Some(60_000)), 60_000);
    assert_eq!(paced_millihertz(144_149, Some(144_149)), 144_149);
    assert_eq!(paced_millihertz(144_149, Some(1_000)), 1_000);
    let options = System::parse(&format!("--system {COPY_REFRESH_HZ} 1")).unwrap();
    assert_eq!(options.copy_refresh_cap_millihertz(), Some(1_000));
    assert_eq!(
        paced_millihertz(144_149, options.copy_refresh_cap_millihertz()),
        1_000
    );
}

#[test]
fn the_highest_cap_in_millihertz_fits_u32() {
    assert_eq!(MAX_COPY_REFRESH_HZ * 1000, 480_000);
    assert_eq!(MAX_COPY_REFRESH_HZ.checked_mul(1000), Some(480_000));
}

#[test]
fn frame_intervals_are_rounded_to_nearest() {
    assert_eq!(frame_interval_ns(60_000), Some(16_666_667));
    assert_eq!(frame_interval_ns(144_149), Some(6_937_266));
    assert_eq!(frame_interval_ns(1_000), Some(1_000_000_000));
    assert_eq!(frame_interval_ns(480_000), Some(2_083_333));
    assert_eq!(frame_interval_ns(0), None);
    assert_eq!(frame_interval_ns(u32::MAX), Some(233));
}
