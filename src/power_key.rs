//! AXP2101 PMIC power-key event interpretation.
//!
//! The Waveshare board routes the physical power button through the AXP2101
//! PMIC rather than through one of the three application-button GPIOs. The
//! register-level I2C access remains in [`crate::power`]; this module keeps the
//! short-press product policy host-testable and separate from PMIC transport.

/// Polling cadence for the PMIC power-key short-press status bit.
pub const POWER_KEY_POLL_MS: u64 = 100;

/// AXP2101 IRQ2 bit used for a POWERON short press.
///
/// XPowers names the source `XPOWERS_AXP2101_PKEY_SHORT_IRQ` at global bit 11,
/// which maps to bit 3 of AXP2101 `INTEN2` / `INTSTS2`.
pub const POWER_KEY_SHORT_PRESS_MASK: u8 = 1 << 3;

/// Minimum quiet interval after sleep-image entry before a new PMIC short
/// press can wake the device. This suppresses a queued PEK event emitted by
/// the same physical press that initiated the sleep transition.
pub const POWER_KEY_WAKE_GUARD_QUIET_MS: u64 = 900;

/// Decision returned when a PMIC short-press event arrives while the sleep
/// image is visible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SleepWakeGuardDecision {
    /// Ignore an entry-press event observed before the quiet interval elapsed.
    SuppressStalePress,
    /// Accept the event as a deliberate second physical Power press.
    AllowWake,
}

/// Host-testable PMIC sleep-entry wake guard.
///
/// The AXP2101 exposes a sticky short-press interrupt rather than a raw button
/// level. The display transition takes several seconds, so a second sticky
/// event from the entry press can appear just after sleep mode is recorded.
/// This guard suppresses such events until one full quiet interval has elapsed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SleepWakeGuard {
    waiting_for_quiet_window: bool,
    armed: bool,
    suppressed_events: u32,
}

impl SleepWakeGuard {
    /// Start a fresh guard window after the sleep image is committed.
    pub fn begin_sleep_entry(&mut self) {
        self.waiting_for_quiet_window = true;
        self.armed = false;
    }

    /// Clear guard state after a successful Power-key or RTC-alarm wake.
    pub fn reset_after_wake(&mut self) {
        self.waiting_for_quiet_window = false;
        self.armed = false;
    }

    /// Arm wake processing once the quiet window has elapsed. Returns `true`
    /// only for the transition from waiting to armed.
    #[must_use]
    pub fn arm_after_quiet_window(&mut self, elapsed_ms: u64) -> bool {
        if self.waiting_for_quiet_window
            && !self.armed
            && elapsed_ms >= POWER_KEY_WAKE_GUARD_QUIET_MS
        {
            self.waiting_for_quiet_window = false;
            self.armed = true;
            true
        } else {
            false
        }
    }

    /// Classify one short-press event observed while the sleep image is active.
    #[must_use]
    pub fn on_short_press(&mut self, elapsed_ms: u64) -> SleepWakeGuardDecision {
        let _ = self.arm_after_quiet_window(elapsed_ms);
        if self.armed {
            SleepWakeGuardDecision::AllowWake
        } else {
            self.suppressed_events = self.suppressed_events.saturating_add(1);
            SleepWakeGuardDecision::SuppressStalePress
        }
    }

    #[must_use]
    pub const fn suppressed_events(&self) -> u32 {
        self.suppressed_events
    }
}

/// Product-facing power-key events.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PowerKeyEvent {
    /// A short physical power-key press toggles sleep-image mode.
    ShortPress,
}

/// Interpret one AXP2101 `INTSTS2` byte.
#[must_use]
pub const fn short_press_from_irq_status(status2: u8) -> Option<PowerKeyEvent> {
    if status2 & POWER_KEY_SHORT_PRESS_MASK != 0 {
        Some(PowerKeyEvent::ShortPress)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        short_press_from_irq_status, PowerKeyEvent, SleepWakeGuard, SleepWakeGuardDecision,
        POWER_KEY_SHORT_PRESS_MASK, POWER_KEY_WAKE_GUARD_QUIET_MS,
    };

    #[test]
    fn detects_axp2101_short_press_bit() {
        assert_eq!(POWER_KEY_SHORT_PRESS_MASK, 0x08);
        assert_eq!(
            short_press_from_irq_status(0x08),
            Some(PowerKeyEvent::ShortPress)
        );
        assert_eq!(
            short_press_from_irq_status(0x88),
            Some(PowerKeyEvent::ShortPress)
        );
    }

    #[test]
    fn ignores_unrelated_axp2101_irq2_bits() {
        assert_eq!(short_press_from_irq_status(0x00), None);
        assert_eq!(short_press_from_irq_status(0x04), None);
        assert_eq!(short_press_from_irq_status(0x10), None);
    }

    #[test]
    fn wake_guard_suppresses_entry_press_until_quiet_window() {
        let mut guard = SleepWakeGuard::default();
        guard.begin_sleep_entry();
        assert_eq!(POWER_KEY_WAKE_GUARD_QUIET_MS, 900);
        assert_eq!(
            guard.on_short_press(120),
            SleepWakeGuardDecision::SuppressStalePress
        );
        assert_eq!(guard.suppressed_events(), 1);
        assert!(!guard.arm_after_quiet_window(899));
        assert!(guard.arm_after_quiet_window(900));
        assert_eq!(guard.on_short_press(901), SleepWakeGuardDecision::AllowWake);
    }

    #[test]
    fn wake_guard_allows_first_press_after_elapsed_quiet_window() {
        let mut guard = SleepWakeGuard::default();
        guard.begin_sleep_entry();
        assert_eq!(
            guard.on_short_press(1_200),
            SleepWakeGuardDecision::AllowWake
        );
        guard.reset_after_wake();
        guard.begin_sleep_entry();
        assert_eq!(
            guard.on_short_press(0),
            SleepWakeGuardDecision::SuppressStalePress
        );
    }
}
