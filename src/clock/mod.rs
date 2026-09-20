pub mod fonts;

use chrono::{DateTime, Local, Timelike};
use std::time::Duration;

/// Returns duration until the next whole second boundary.
pub fn duration_until_next_second(now: &DateTime<Local>) -> Duration {
    let nanos = now.nanosecond();
    let remaining_nanos = 1_000_000_000u64.saturating_sub(nanos as u64);
    // Add 2ms padding so we comfortably land within the new second
    Duration::from_nanos(remaining_nanos) + Duration::from_millis(2)
}

/// Creates a continuous stream firing exactly on the next second boundary.
/// Eliminates timer drift and skipped seconds.
pub fn next_second_tick() -> impl futures::Stream<Item = DateTime<Local>> {
    futures::stream::unfold((), |_| async {
        let now = Local::now();
        let delay = duration_until_next_second(&now);
        tokio::time::sleep(delay).await;
        let tick_time = Local::now();
        Some((tick_time, ()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_until_next_second() {
        let now = Local::now();
        let dur = duration_until_next_second(&now);
        // Duration should be positive and less than or equal to 1.1 seconds
        assert!(dur.as_millis() > 0);
        assert!(dur.as_millis() <= 1100);
    }
}
