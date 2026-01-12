use chrono::{DateTime, Local};
use std::time::Instant;

/// Trait for abstracting time operations, enabling testability
pub trait Clock: Send + Sync {
    /// Get the current Unix timestamp in seconds
    fn now_timestamp(&self) -> i64;

    /// Get the current local datetime
    fn now_datetime(&self) -> DateTime<Local>;

    /// Get a monotonic instant for elapsed time tracking
    fn instant(&self) -> Instant;
}

/// System clock implementation using real time
#[derive(Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_timestamp(&self) -> i64 {
        Local::now().timestamp()
    }

    fn now_datetime(&self) -> DateTime<Local> {
        Local::now()
    }

    fn instant(&self) -> Instant {
        Instant::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clock_now_timestamp() {
        let clock = SystemClock;
        let ts = clock.now_timestamp();
        // Timestamp should be positive and reasonable (after year 2000)
        assert!(ts > 946684800); // Jan 1, 2000
    }

    #[test]
    fn test_system_clock_now_datetime() {
        let clock = SystemClock;
        let dt = clock.now_datetime();
        // Should be a reasonable year
        assert!(dt.format("%Y").to_string().parse::<i32>().unwrap() >= 2024);
    }

    #[test]
    fn test_system_clock_instant_elapsed() {
        let clock = SystemClock;
        let start = clock.instant();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }
}
