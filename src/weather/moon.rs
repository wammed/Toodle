use chrono::{DateTime, NaiveDate, TimeZone, Utc};

/// Reference New Moon: 2000-01-06 18:14:00 UTC (Unix timestamp: 947182440)
const REFERENCE_NEW_MOON_TIMESTAMP: f64 = 947182440.0;

/// Synodic month in seconds (29.530588853 days * 86400 seconds/day)
const SYNODIC_MONTH_SECONDS: f64 = 29.530588853 * 86400.0;

/// 28 Weather Icons glyphs covering the full lunar cycle in 28 distinct steps.
const MOON_GLYPHS_28: [char; 28] = [
    '\u{f095}', // New
    '\u{f096}', '\u{f097}', '\u{f098}', '\u{f099}', '\u{f09a}',
    '\u{f09b}', // Waxing Crescent 1-6
    '\u{f09c}', // First Quarter
    '\u{f09d}', '\u{f09e}', '\u{f09f}', '\u{f0a0}', '\u{f0a1}',
    '\u{f0a2}', // Waxing Gibbous 1-6
    '\u{f0a3}', // Full
    '\u{f0a4}', '\u{f0a5}', '\u{f0a6}', '\u{f0a7}', '\u{f0a8}',
    '\u{f0a9}', // Waning Gibbous 1-6
    '\u{f0aa}', // Third Quarter
    '\u{f0ab}', '\u{f0ac}', '\u{f0ad}', '\u{f0ae}', '\u{f0af}',
    '\u{f0b0}', // Waning Crescent 1-6
];

/// Represents the moon phase calculated from a timestamp or date.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoonPhase {
    /// Normalized phase value in range [0.0, 1.0), where 0.0 is New Moon, 0.5 is Full Moon.
    pub phase: f64,
}

impl MoonPhase {
    /// Calculates the moon phase from a Unix timestamp in seconds.
    pub fn from_utc_timestamp(timestamp_secs: i64) -> Self {
        let diff = (timestamp_secs as f64) - REFERENCE_NEW_MOON_TIMESTAMP;
        let cycles = diff / SYNODIC_MONTH_SECONDS;
        let mut phase = cycles - cycles.floor();
        if phase < 0.0 {
            phase += 1.0;
        }
        Self { phase }
    }

    /// Calculates the moon phase from any `chrono::DateTime`.
    pub fn from_datetime<Tz: TimeZone>(dt: &DateTime<Tz>) -> Self {
        Self::from_utc_timestamp(dt.timestamp())
    }

    /// Calculates the moon phase for noon UTC of a date given as "YYYY-MM-DD".
    pub fn from_ymd_str(ymd: &str) -> Option<Self> {
        let date = NaiveDate::parse_from_str(ymd, "%Y-%m-%d").ok()?;
        let dt = date.and_hms_opt(12, 0, 0)?;
        let utc_dt = Utc.from_utc_datetime(&dt);
        Some(Self::from_datetime(&utc_dt))
    }

    /// Returns the moon's age in days (0.0 to ~29.53).
    pub fn age_days(&self) -> f64 {
        self.phase * 29.530588853
    }

    /// Returns the corresponding 28-phase Weather Icons font glyph.
    pub fn glyph(&self) -> char {
        let index = ((self.phase * 28.0 + 0.5).floor() as usize) % 28;
        MOON_GLYPHS_28[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_new_moon() {
        let moon = MoonPhase::from_utc_timestamp(947182440);
        assert!((moon.phase - 0.0).abs() < 1e-6);
        assert_eq!(moon.glyph(), '\u{f095}'); // wi-moon-new
    }

    #[test]
    fn test_quarters_and_full() {
        // Halfway through synodic month -> Full Moon
        let full_ts = 947182440 + (SYNODIC_MONTH_SECONDS * 0.5) as i64;
        let moon_full = MoonPhase::from_utc_timestamp(full_ts);
        assert!((moon_full.phase - 0.5).abs() < 1e-4);
        assert_eq!(moon_full.glyph(), '\u{f0a3}'); // wi-moon-full

        // First quarter (~0.25)
        let fq_ts = 947182440 + (SYNODIC_MONTH_SECONDS * 0.25) as i64;
        let moon_fq = MoonPhase::from_utc_timestamp(fq_ts);
        assert!((moon_fq.phase - 0.25).abs() < 1e-4);
        assert_eq!(moon_fq.glyph(), '\u{f09c}'); // wi-moon-first-quarter

        // Third quarter (~0.75)
        let tq_ts = 947182440 + (SYNODIC_MONTH_SECONDS * 0.75) as i64;
        let moon_tq = MoonPhase::from_utc_timestamp(tq_ts);
        assert!((moon_tq.phase - 0.75).abs() < 1e-4);
        assert_eq!(moon_tq.glyph(), '\u{f0aa}'); // wi-moon-third-quarter
    }

    #[test]
    fn test_from_ymd_str() {
        let moon = MoonPhase::from_ymd_str("2026-09-21");
        assert!(moon.is_some());
        let m = moon.unwrap();
        assert!(m.phase >= 0.0 && m.phase < 1.0);
        assert!(MOON_GLYPHS_28.contains(&m.glyph()));
    }

    #[test]
    fn test_all_28_glyphs_mapped() {
        for (i, expected_glyph) in MOON_GLYPHS_28.iter().enumerate() {
            let phase = i as f64 / 28.0;
            let moon = MoonPhase { phase };
            assert_eq!(moon.glyph(), *expected_glyph);
        }
    }
}
