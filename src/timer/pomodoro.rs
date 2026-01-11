use std::time::{Duration, Instant};

use crate::config::{
    DEFAULT_LONG_BREAK, DEFAULT_SHORT_BREAK, DEFAULT_WORK_DURATION, SESSIONS_UNTIL_LONG_BREAK,
};
use crate::models::Config;

/// The current phase of the pomodoro cycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimerPhase {
    #[default]
    Work,
    ShortBreak,
    LongBreak,
}

impl TimerPhase {
    pub fn label(&self) -> &'static str {
        match self {
            TimerPhase::Work => "WORK SESSION",
            TimerPhase::ShortBreak => "SHORT BREAK",
            TimerPhase::LongBreak => "LONG BREAK",
        }
    }

    pub fn is_break(&self) -> bool {
        matches!(self, TimerPhase::ShortBreak | TimerPhase::LongBreak)
    }
}

/// The current state of the timer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimerState {
    #[default]
    Idle,
    Running {
        started: Instant,
        elapsed_before_pause: Duration,
    },
    Paused {
        elapsed: Duration,
    },
}

/// The pomodoro timer state machine
#[derive(Debug)]
pub struct PomodoroTimer {
    pub phase: TimerPhase,
    pub state: TimerState,
    pub work_duration: Duration,
    pub short_break: Duration,
    pub long_break: Duration,
    pub sessions_until_long: u8,
    pub sessions_completed: u8,
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        Self {
            phase: TimerPhase::Work,
            state: TimerState::Idle,
            work_duration: DEFAULT_WORK_DURATION,
            short_break: DEFAULT_SHORT_BREAK,
            long_break: DEFAULT_LONG_BREAK,
            sessions_until_long: SESSIONS_UNTIL_LONG_BREAK,
            sessions_completed: 0,
        }
    }
}

impl PomodoroTimer {
    /// Create a new pomodoro timer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the duration of the current phase
    pub fn current_phase_duration(&self) -> Duration {
        match self.phase {
            TimerPhase::Work => self.work_duration,
            TimerPhase::ShortBreak => self.short_break,
            TimerPhase::LongBreak => self.long_break,
        }
    }

    /// Get the elapsed time in the current phase
    pub fn elapsed(&self) -> Duration {
        match self.state {
            TimerState::Idle => Duration::ZERO,
            TimerState::Running {
                started,
                elapsed_before_pause,
            } => elapsed_before_pause + started.elapsed(),
            TimerState::Paused { elapsed } => elapsed,
        }
    }

    /// Get the remaining time in the current phase
    pub fn remaining(&self) -> Duration {
        self.current_phase_duration().saturating_sub(self.elapsed())
    }

    /// Check if the current phase is finished
    pub fn is_finished(&self) -> bool {
        self.elapsed() >= self.current_phase_duration()
    }

    /// Check if the timer is currently running
    pub fn is_running(&self) -> bool {
        matches!(self.state, TimerState::Running { .. })
    }

    /// Check if the timer is paused
    pub fn is_paused(&self) -> bool {
        matches!(self.state, TimerState::Paused { .. })
    }

    /// Check if the timer is idle (not started)
    pub fn is_idle(&self) -> bool {
        matches!(self.state, TimerState::Idle)
    }

    /// Start or resume the timer
    pub fn start(&mut self) {
        match self.state {
            TimerState::Idle => {
                self.state = TimerState::Running {
                    started: Instant::now(),
                    elapsed_before_pause: Duration::ZERO,
                };
            }
            TimerState::Paused { elapsed } => {
                self.state = TimerState::Running {
                    started: Instant::now(),
                    elapsed_before_pause: elapsed,
                };
            }
            TimerState::Running { .. } => {
                // Already running, do nothing
            }
        }
    }

    /// Pause the timer
    pub fn pause(&mut self) {
        if let TimerState::Running { .. } = self.state {
            self.state = TimerState::Paused {
                elapsed: self.elapsed(),
            };
        }
    }

    /// Reset the timer to idle state
    pub fn reset(&mut self) {
        self.state = TimerState::Idle;
    }

    /// Advance to the next phase
    pub fn advance_phase(&mut self) {
        match self.phase {
            TimerPhase::Work => {
                self.sessions_completed += 1;
                if self.sessions_completed >= self.sessions_until_long {
                    self.phase = TimerPhase::LongBreak;
                    self.sessions_completed = 0;
                } else {
                    self.phase = TimerPhase::ShortBreak;
                }
            }
            TimerPhase::ShortBreak | TimerPhase::LongBreak => {
                self.phase = TimerPhase::Work;
            }
        }
        self.state = TimerState::Idle;
    }

    /// Skip the current break and start a new work session
    pub fn skip_break(&mut self) {
        if self.phase.is_break() {
            self.phase = TimerPhase::Work;
            self.state = TimerState::Idle;
        }
    }

    /// Apply configuration settings to the timer
    pub fn apply_config(&mut self, config: &Config) {
        self.work_duration = Duration::from_secs(config.work_duration_secs as u64);
        self.short_break = Duration::from_secs(config.short_break_secs as u64);
        self.long_break = Duration::from_secs(config.long_break_secs as u64);
        self.sessions_until_long = config.sessions_until_long_break as u8;
    }

    /// Get the progress as a ratio (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        let total = self.current_phase_duration().as_secs_f64();
        if total == 0.0 {
            return 1.0;
        }
        let elapsed = self.elapsed().as_secs_f64();
        (elapsed / total).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_timer_is_idle() {
        let timer = PomodoroTimer::new();
        assert!(timer.is_idle());
        assert_eq!(timer.phase, TimerPhase::Work);
        assert_eq!(timer.elapsed(), Duration::ZERO);
    }

    #[test]
    fn test_start_changes_state_to_running() {
        let mut timer = PomodoroTimer::new();
        timer.start();
        assert!(timer.is_running());
    }

    #[test]
    fn test_pause_and_resume() {
        let mut timer = PomodoroTimer::new();
        timer.start();
        std::thread::sleep(Duration::from_millis(10));
        timer.pause();
        assert!(timer.is_paused());
        let elapsed_paused = timer.elapsed();
        std::thread::sleep(Duration::from_millis(10));
        // Elapsed should not increase while paused
        assert_eq!(timer.elapsed(), elapsed_paused);
        timer.start();
        assert!(timer.is_running());
    }

    #[test]
    fn test_reset_returns_to_idle() {
        let mut timer = PomodoroTimer::new();
        timer.start();
        timer.reset();
        assert!(timer.is_idle());
        assert_eq!(timer.elapsed(), Duration::ZERO);
    }

    #[test]
    fn test_phase_advancement() {
        let mut timer = PomodoroTimer::new();
        timer.advance_phase();
        assert_eq!(timer.phase, TimerPhase::ShortBreak);
        assert_eq!(timer.sessions_completed, 1);

        timer.advance_phase();
        assert_eq!(timer.phase, TimerPhase::Work);
    }

    #[test]
    fn test_long_break_after_four_sessions() {
        let mut timer = PomodoroTimer::new();
        for _ in 0..3 {
            timer.advance_phase(); // Work -> ShortBreak
            timer.advance_phase(); // ShortBreak -> Work
        }
        timer.advance_phase(); // 4th work session complete
        assert_eq!(timer.phase, TimerPhase::LongBreak);
        assert_eq!(timer.sessions_completed, 0);
    }
}
