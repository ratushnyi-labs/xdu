//! Size formatting and proportion bars.
//!
//! BR-006: default unit is IEC binary (1 KiB = 1024 B); `--si` switches to
//! decimal (1 KB = 1000 B). Used by UC-003.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SizeMode {
    Iec,
    Si,
}

const IEC_UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
const SI_UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB", "EB"];

pub fn format_size(bytes: u64, mode: SizeMode) -> String {
    let (divisor, units) = match mode {
        SizeMode::Iec => (1024_u64, IEC_UNITS),
        SizeMode::Si => (1000_u64, SI_UNITS),
    };
    if bytes < divisor {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut idx = 0_usize;
    while value >= divisor as f64 && idx < units.len() - 1 {
        value /= divisor as f64;
        idx += 1;
    }
    if value >= 100.0 {
        format!("{value:.0} {}", units[idx])
    } else {
        format!("{value:.1} {}", units[idx])
    }
}

const FULL_BLOCK: char = '\u{2588}';

// Index = N/8 of a cell filled. Index 0 is unused (caller skips); 1..=7
// are the 1/8..7/8 left-fill block-element glyphs U+258F..U+2589.
const PARTIAL_BLOCKS: [char; 8] = [
    ' ', '\u{258F}', '\u{258E}', '\u{258D}', '\u{258C}', '\u{258B}', '\u{258A}', '\u{2589}',
];

const MAX_BAR_WIDTH: usize = 256;

pub fn bar(fraction: f64, width: usize, utf8: bool) -> String {
    let width = width.min(MAX_BAR_WIDTH);
    if width == 0 {
        return String::new();
    }
    let frac = if fraction.is_nan() {
        0.0
    } else {
        fraction.clamp(0.0, 1.0)
    };

    if !utf8 {
        let filled = ((frac * width as f64).round() as usize).min(width);
        let mut s = String::with_capacity(width);
        for _ in 0..filled {
            s.push('#');
        }
        for _ in filled..width {
            s.push(' ');
        }
        return s;
    }

    let total_sub = width.saturating_mul(8);
    let fill_sub = (frac * total_sub as f64).round() as usize;
    let full_cells = (fill_sub / 8).min(width);
    let partial = fill_sub % 8;

    let mut s = String::with_capacity(width * 4);
    for _ in 0..full_cells {
        s.push(FULL_BLOCK);
    }
    let mut written = full_cells;
    if written < width && partial > 0 {
        s.push(PARTIAL_BLOCKS[partial]);
        written += 1;
    }
    for _ in written..width {
        s.push(' ');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_zero() {
        assert_eq!(format_size(0, SizeMode::Iec), "0 B");
        assert_eq!(format_size(0, SizeMode::Si), "0 B");
    }

    #[test]
    fn format_size_below_divisor_iec() {
        assert_eq!(format_size(1, SizeMode::Iec), "1 B");
        assert_eq!(format_size(1023, SizeMode::Iec), "1023 B");
    }

    #[test]
    fn format_size_below_divisor_si() {
        assert_eq!(format_size(999, SizeMode::Si), "999 B");
    }

    #[test]
    fn format_size_iec_steps() {
        assert_eq!(format_size(1024, SizeMode::Iec), "1.0 KiB");
        assert_eq!(format_size(1536, SizeMode::Iec), "1.5 KiB");
        assert_eq!(format_size(1024 * 1024, SizeMode::Iec), "1.0 MiB");
        assert_eq!(format_size(1024_u64.pow(3), SizeMode::Iec), "1.0 GiB");
        assert_eq!(format_size(1024_u64.pow(4), SizeMode::Iec), "1.0 TiB");
        assert_eq!(format_size(1024_u64.pow(5), SizeMode::Iec), "1.0 PiB");
        assert_eq!(format_size(1024_u64.pow(6), SizeMode::Iec), "1.0 EiB");
    }

    #[test]
    fn format_size_si_steps() {
        assert_eq!(format_size(1000, SizeMode::Si), "1.0 KB");
        assert_eq!(format_size(1500, SizeMode::Si), "1.5 KB");
        assert_eq!(format_size(1_000_000, SizeMode::Si), "1.0 MB");
        assert_eq!(format_size(1_000_000_000, SizeMode::Si), "1.0 GB");
    }

    #[test]
    fn format_size_three_digit_no_decimal() {
        // 500 KiB stays "500 KiB", not "500.0 KiB"
        assert_eq!(format_size(500 * 1024, SizeMode::Iec), "500 KiB");
    }

    #[test]
    fn format_size_clamps_at_max_unit() {
        let s = format_size(u64::MAX, SizeMode::Iec);
        assert!(s.ends_with(" EiB"), "got: {s}");
        let s = format_size(u64::MAX, SizeMode::Si);
        assert!(s.ends_with(" EB"), "got: {s}");
    }

    #[test]
    fn bar_zero_fraction_all_empty_utf8() {
        assert_eq!(bar(0.0, 10, true), "          ");
    }

    #[test]
    fn bar_zero_fraction_all_empty_ascii() {
        assert_eq!(bar(0.0, 10, false), "          ");
    }

    #[test]
    fn bar_full_fraction_all_full_utf8() {
        assert_eq!(bar(1.0, 10, true), "██████████");
    }

    #[test]
    fn bar_full_fraction_all_full_ascii() {
        assert_eq!(bar(1.0, 10, false), "##########");
    }

    #[test]
    fn bar_half_utf8() {
        assert_eq!(bar(0.5, 10, true), "█████     ");
    }

    #[test]
    fn bar_half_ascii_rounds() {
        assert_eq!(bar(0.5, 10, false), "#####     ");
    }

    #[test]
    fn bar_partial_block_at_55pct() {
        // 0.55 * 80 = 44 sub-cells = 5 full + 4/8 partial + 4 empty
        assert_eq!(bar(0.55, 10, true), "█████\u{258C}    ");
    }

    #[test]
    fn bar_zero_width() {
        assert_eq!(bar(0.5, 0, true), "");
        assert_eq!(bar(0.5, 0, false), "");
    }

    #[test]
    fn bar_clamps_negative_fraction() {
        assert_eq!(bar(-1.0, 10, true), "          ");
        assert_eq!(bar(f64::NEG_INFINITY, 10, true), "          ");
    }

    #[test]
    fn bar_clamps_overflow_fraction() {
        assert_eq!(bar(2.0, 10, true), "██████████");
        assert_eq!(bar(f64::INFINITY, 10, true), "██████████");
    }

    #[test]
    fn bar_clamps_excessive_width() {
        let s = bar(1.0, 10_000, false);
        assert_eq!(s.chars().count(), MAX_BAR_WIDTH);
    }

    #[test]
    fn sec_uc003_format_size_no_panic_on_max() {
        let _ = format_size(u64::MAX, SizeMode::Iec);
        let _ = format_size(u64::MAX, SizeMode::Si);
    }

    #[test]
    fn sec_uc003_bar_no_panic_on_extreme_inputs() {
        let _ = bar(f64::NAN, 10, true);
        let _ = bar(f64::INFINITY, 10, true);
        let _ = bar(f64::NEG_INFINITY, 10, true);
        let _ = bar(0.5, usize::MAX, true);
        let _ = bar(0.5, usize::MAX, false);
    }

    #[test]
    fn sec_uc003_bar_nan_yields_empty() {
        // Defined behavior: NaN treated as 0.0.
        assert_eq!(bar(f64::NAN, 10, true), "          ");
    }
}
