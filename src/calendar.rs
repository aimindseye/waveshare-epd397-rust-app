//! Hardware-independent Gregorian calendar math and read-only month-view state.
//!
//! The Waveshare RTC stores years in the 2000..=2099 range. Keep Calendar
//! Foundation deliberately bounded to that same range so month navigation,
//! leap-year handling and selected-day clamping remain predictable on-device.

use crate::rtc::RtcDateTime;

/// First year supported by the RTC-backed Calendar Foundation.
pub const CALENDAR_MIN_YEAR: u16 = 2000;
/// Last year supported by the RTC-backed Calendar Foundation.
pub const CALENDAR_MAX_YEAR: u16 = 2099;

/// One valid Gregorian date inside the RTC-backed supported range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CalendarDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Default for CalendarDate {
    fn default() -> Self {
        Self {
            year: CALENDAR_MIN_YEAR,
            month: 1,
            day: 1,
        }
    }
}

impl CalendarDate {
    /// Build a bounded date when all fields are valid.
    #[must_use]
    pub const fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        if year < CALENDAR_MIN_YEAR
            || year > CALENDAR_MAX_YEAR
            || month == 0
            || month > 12
            || day == 0
            || day > days_in_month(year, month)
        {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// Use a localized RTC snapshot as the Calendar cursor when possible.
    #[must_use]
    pub fn from_rtc(value: RtcDateTime) -> Self {
        Self::new(value.year, value.month, value.day).unwrap_or_default()
    }

    /// Sunday-zero weekday index used by the month grid.
    #[must_use]
    pub fn weekday(self) -> u8 {
        weekday(self.year, self.month, self.day)
    }

    /// Move by signed days while clamping at the RTC-supported boundaries.
    #[must_use]
    pub fn shifted_days(mut self, delta: i32) -> Self {
        let mut remaining = delta;
        while remaining > 0 {
            if self == Self::maximum() {
                break;
            }
            self = self.next_day();
            remaining -= 1;
        }
        while remaining < 0 {
            if self == Self::minimum() {
                break;
            }
            self = self.previous_day();
            remaining += 1;
        }
        self
    }

    /// Move by signed months while retaining the selected day when possible.
    #[must_use]
    pub fn shifted_months(self, delta: i32) -> Self {
        let current = i32::from(self.year) * 12 + i32::from(self.month) - 1;
        let minimum = i32::from(CALENDAR_MIN_YEAR) * 12;
        let maximum = i32::from(CALENDAR_MAX_YEAR) * 12 + 11;
        let shifted = current.saturating_add(delta).clamp(minimum, maximum);
        let year = (shifted / 12) as u16;
        let month = (shifted % 12 + 1) as u8;
        let day = self.day.min(days_in_month(year, month));
        Self { year, month, day }
    }

    #[must_use]
    pub const fn minimum() -> Self {
        Self {
            year: CALENDAR_MIN_YEAR,
            month: 1,
            day: 1,
        }
    }

    #[must_use]
    pub const fn maximum() -> Self {
        Self {
            year: CALENDAR_MAX_YEAR,
            month: 12,
            day: 31,
        }
    }

    fn next_day(self) -> Self {
        let days = days_in_month(self.year, self.month);
        if self.day < days {
            return Self {
                day: self.day + 1,
                ..self
            };
        }
        if self.month < 12 {
            return Self {
                year: self.year,
                month: self.month + 1,
                day: 1,
            };
        }
        Self {
            year: self.year + 1,
            month: 1,
            day: 1,
        }
    }

    fn previous_day(self) -> Self {
        if self.day > 1 {
            return Self {
                day: self.day - 1,
                ..self
            };
        }
        if self.month > 1 {
            let month = self.month - 1;
            return Self {
                year: self.year,
                month,
                day: days_in_month(self.year, month),
            };
        }
        Self {
            year: self.year - 1,
            month: 12,
            day: 31,
        }
    }
}

/// Navigation axis selected on the read-only monthly Calendar page.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CalendarNavigationMode {
    #[default]
    Day,
    Month,
}

impl CalendarNavigationMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Day => "DAY MODE",
            Self::Month => "MONTH MODE",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Day => Self::Month,
            Self::Month => Self::Day,
        }
    }
}

/// UI-owned cursor for Calendar Foundation. No persistent events are stored.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CalendarUiState {
    pub cursor: CalendarDate,
    pub mode: CalendarNavigationMode,
    initialized: bool,
}

impl CalendarUiState {
    /// Initialize the cursor once from a localized RTC snapshot. Returning to
    /// Calendar later keeps the user's previous month and selected day.
    pub fn initialize_if_needed(&mut self, local: Option<RtcDateTime>) {
        if self.initialized {
            return;
        }
        if let Some(value) = local {
            self.cursor = CalendarDate::from_rtc(value);
        }
        self.initialized = true;
    }

    /// Move backward according to the active axis.
    pub fn move_previous(&mut self) {
        self.cursor = match self.mode {
            CalendarNavigationMode::Day => self.cursor.shifted_days(-1),
            CalendarNavigationMode::Month => self.cursor.shifted_months(-1),
        };
    }

    /// Move forward according to the active axis.
    pub fn move_next(&mut self) {
        self.cursor = match self.mode {
            CalendarNavigationMode::Day => self.cursor.shifted_days(1),
            CalendarNavigationMode::Month => self.cursor.shifted_months(1),
        };
    }

    /// Toggle between selected-day and selected-month navigation.
    pub fn toggle_mode(&mut self) {
        self.mode = self.mode.next();
    }
}

/// Gregorian leap-year policy used by Calendar Foundation.
#[must_use]
pub const fn is_leap_year(year: u16) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Number of days in one month. Invalid months safely return zero.
#[must_use]
pub const fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Sunday-zero weekday for one valid Gregorian date.
#[must_use]
pub fn weekday(year: u16, month: u8, day: u8) -> u8 {
    const OFFSETS: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut adjusted_year = i32::from(year);
    if month < 3 {
        adjusted_year -= 1;
    }
    let index = usize::from(month.saturating_sub(1).min(11));
    (adjusted_year + adjusted_year / 4 - adjusted_year / 100
        + adjusted_year / 400
        + OFFSETS[index]
        + i32::from(day))
    .rem_euclid(7) as u8
}

#[cfg(test)]
mod tests {
    use super::{
        days_in_month, is_leap_year, weekday, CalendarDate, CalendarNavigationMode, CalendarUiState,
    };
    use crate::rtc::RtcDateTime;

    #[test]
    fn handles_rtc_range_leap_years() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2025));
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2025, 2), 28);
    }

    #[test]
    fn weekday_matches_known_thursday() {
        assert_eq!(weekday(2026, 6, 4), 4);
    }

    #[test]
    fn day_navigation_crosses_month_boundary() {
        assert_eq!(
            CalendarDate::new(2026, 6, 30).unwrap().shifted_days(1),
            CalendarDate::new(2026, 7, 1).unwrap()
        );
        assert_eq!(
            CalendarDate::new(2024, 3, 1).unwrap().shifted_days(-1),
            CalendarDate::new(2024, 2, 29).unwrap()
        );
    }

    #[test]
    fn month_navigation_clamps_selected_day() {
        assert_eq!(
            CalendarDate::new(2026, 1, 31).unwrap().shifted_months(1),
            CalendarDate::new(2026, 2, 28).unwrap()
        );
    }

    #[test]
    fn initializes_once_from_local_rtc_and_toggles_mode() {
        let mut state = CalendarUiState::default();
        state.initialize_if_needed(Some(RtcDateTime {
            year: 2026,
            month: 6,
            day: 4,
            weekday: 4,
            hour: 8,
            minute: 13,
            second: 0,
        }));
        assert_eq!(state.cursor, CalendarDate::new(2026, 6, 4).unwrap());
        state.toggle_mode();
        assert_eq!(state.mode, CalendarNavigationMode::Month);
    }
}
