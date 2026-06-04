//! Product UI state transitions independent of hardware wiring.

use crate::{
    alarm::AlarmSnapshot,
    audio::{AudioSnapshot, AudioUiRequest},
    board_services::BoardSnapshot,
    buttons::ButtonEvent,
    calendar::CalendarUiState,
    network::NetworkSnapshot,
    orientation::DisplayOrientation,
    regional::RegionalPreferences,
    storage::StorageSnapshot,
    unit_converter::UnitConverterUiState,
    weather::WeatherSnapshot,
};

use super::{
    display::DisplayPreferences,
    menu::{category_entries, category_index, home_entries, CATEGORY_COUNT},
    router::{ScreenRoute, ScreenRouter},
};

/// Number of selectable rows in the playback overview screen.
pub const AUDIO_ACTION_COUNT: usize = 6;
/// Number of selectable rows in the Display settings screen.
pub const DISPLAY_ACTION_COUNT: usize = 2;
/// Number of selectable rows in the Weather overview screen.
pub const WEATHER_ACTION_COUNT: usize = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppState {
    pub home_selected: usize,
    category_selected: [usize; CATEGORY_COUNT],
    pub display_action_selected: usize,
    pub display: DisplayPreferences,
    /// Read-only monthly Calendar Foundation cursor and navigation mode.
    pub calendar: CalendarUiState,
    /// Offline fixed-point Unit Converter cursor and editable field.
    pub unit_converter: UnitConverterUiState,
    pub partial_refreshes: u8,
    pub panel_awake: bool,
    pub select_presses: u32,
    pub orientation: DisplayOrientation,
    pub regional: RegionalPreferences,
    pub router: ScreenRouter,
    pub board: BoardSnapshot,
    pub storage: StorageSnapshot,
    /// Password-free snapshot owned by the networking boundary.
    pub network: NetworkSnapshot,
    /// Cached weather snapshot retained across transient HTTP failures.
    pub weather: WeatherSnapshot,
    /// SD-backed alarm schedules and active-alarm UI snapshot.
    pub alarms: AlarmSnapshot,
    /// Playback-only ES8311 diagnostics snapshot.
    pub audio: AudioSnapshot,
    /// Selected Audio-overview action.
    pub audio_action_selected: usize,
    /// Selected Weather-overview action: refresh or details.
    pub weather_action_selected: usize,
    weather_refresh_requested: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            home_selected: 0,
            category_selected: [0; CATEGORY_COUNT],
            display_action_selected: 0,
            display: DisplayPreferences::default(),
            calendar: CalendarUiState::default(),
            unit_converter: UnitConverterUiState::default(),
            partial_refreshes: 0,
            panel_awake: true,
            select_presses: 0,
            orientation: DisplayOrientation::default(),
            regional: RegionalPreferences::default(),
            router: ScreenRouter::default(),
            board: BoardSnapshot::default(),
            storage: StorageSnapshot::default(),
            network: NetworkSnapshot::default(),
            weather: WeatherSnapshot::default(),
            alarms: AlarmSnapshot::default(),
            audio: AudioSnapshot::default(),
            audio_action_selected: 0,
            weather_action_selected: 0,
            weather_refresh_requested: false,
        }
    }
}

impl AppState {
    /// Apply one debounced button event to routes whose behavior is fully
    /// hardware-independent. Files, Alarms and Audio remain delegated to their
    /// existing owners from main.rs.
    pub fn apply(&mut self, event: ButtonEvent) {
        let route = self.router.current();
        if route == ScreenRoute::Home {
            self.apply_home(event);
        } else if route.is_category() {
            self.apply_category(route, event);
        } else if route == ScreenRoute::Display {
            self.apply_display(event);
        } else if route == ScreenRoute::Calendar {
            self.apply_calendar(event);
        } else if route == ScreenRoute::UnitConverter {
            self.apply_unit_converter(event);
        } else if route.is_placeholder() {
            // Placeholders are intentionally inert. Hierarchical navigation is
            // consistently handled by the dedicated GPIO0 BOOT long press.
        } else {
            match (route, event) {
                (ScreenRoute::Weather, ButtonEvent::Up) => {
                    self.weather_action_selected = self
                        .weather_action_selected
                        .checked_sub(1)
                        .unwrap_or(WEATHER_ACTION_COUNT - 1);
                }
                (ScreenRoute::Weather, ButtonEvent::Down) => {
                    self.weather_action_selected =
                        (self.weather_action_selected + 1) % WEATHER_ACTION_COUNT;
                }
                (ScreenRoute::Weather, ButtonEvent::Select) => {
                    self.note_select_press();
                    if self.weather_action_selected == 0 {
                        self.weather_refresh_requested = true;
                    } else {
                        self.router.navigate_to(ScreenRoute::WeatherDetails);
                    }
                }
                (ScreenRoute::Clock, ButtonEvent::Select) => {
                    self.note_select_press();
                    self.router.navigate_to(ScreenRoute::ClockDetails);
                }
                (ScreenRoute::Environment, ButtonEvent::Select) => {
                    self.note_select_press();
                    self.router.navigate_to(ScreenRoute::EnvironmentDetails);
                }
                (ScreenRoute::Motion, ButtonEvent::Select) => {
                    self.note_select_press();
                    self.router.navigate_to(ScreenRoute::MotionDetails);
                }
                (ScreenRoute::Network, ButtonEvent::Select) => {
                    self.note_select_press();
                    self.router.navigate_to(ScreenRoute::NetworkDetails);
                }
                (ScreenRoute::DeviceInfo, ButtonEvent::Select) => {
                    self.note_select_press();
                    self.router.navigate_to(ScreenRoute::DeviceInfoBoard);
                }
                (ScreenRoute::DeviceInfoBoard, ButtonEvent::Select) => {
                    self.note_select_press();
                    self.router.navigate_to(ScreenRoute::DeviceInfoRuntime);
                }
                (
                    ScreenRoute::Clock
                    | ScreenRoute::Environment
                    | ScreenRoute::Motion
                    | ScreenRoute::Network
                    | ScreenRoute::DeviceInfo
                    | ScreenRoute::DeviceInfoBoard,
                    ButtonEvent::Up | ButtonEvent::Down,
                )
                | (
                    ScreenRoute::AudioDetails
                    | ScreenRoute::ClockDetails
                    | ScreenRoute::DeviceInfoRuntime
                    | ScreenRoute::EnvironmentDetails
                    | ScreenRoute::MotionDetails
                    | ScreenRoute::NetworkDetails
                    | ScreenRoute::WeatherDetails,
                    _,
                )
                | (ScreenRoute::Files | ScreenRoute::Alarms | ScreenRoute::Audio, _) => {}
                _ => {}
            }
        }
    }

    fn apply_home(&mut self, event: ButtonEvent) {
        let count = home_entries().len();
        match event {
            ButtonEvent::Up => {
                self.home_selected = self.home_selected.checked_sub(1).unwrap_or(count - 1);
            }
            ButtonEvent::Down => self.home_selected = (self.home_selected + 1) % count,
            ButtonEvent::Select => {
                self.note_select_press();
                if let Some(entry) = home_entries().get(self.home_selected) {
                    self.router.navigate_to(entry.route);
                }
            }
        }
    }

    fn apply_category(&mut self, route: ScreenRoute, event: ButtonEvent) {
        let entries = category_entries(route);
        match event {
            ButtonEvent::Up => {
                let selected = self.category_selection_mut(route);
                *selected = selected.checked_sub(1).unwrap_or(entries.len() - 1);
            }
            ButtonEvent::Down => {
                let selected = self.category_selection_mut(route);
                *selected = (*selected + 1) % entries.len();
            }
            ButtonEvent::Select => {
                let target = entries[self.category_selection(route)].route;
                self.note_select_press();
                if target == ScreenRoute::Weather {
                    self.weather_action_selected = 0;
                }
                if target == ScreenRoute::Audio {
                    self.audio_action_selected = 0;
                }
                if target == ScreenRoute::Display {
                    self.display_action_selected = 0;
                }
                if target == ScreenRoute::Calendar {
                    self.initialize_calendar_if_needed();
                }
                self.router.navigate_to(target);
            }
        }
    }

    fn initialize_calendar_if_needed(&mut self) {
        let local = self.board.rtc.map(|rtc| self.regional.localize_rtc(rtc));
        self.calendar.initialize_if_needed(local);
    }

    fn apply_calendar(&mut self, event: ButtonEvent) {
        self.initialize_calendar_if_needed();
        match event {
            ButtonEvent::Up => self.calendar.move_previous(),
            ButtonEvent::Down => self.calendar.move_next(),
            ButtonEvent::Select => {
                self.note_select_press();
                self.calendar.toggle_mode();
            }
        }
    }

    fn apply_unit_converter(&mut self, event: ButtonEvent) {
        match event {
            ButtonEvent::Up => self.unit_converter.increase_active(),
            ButtonEvent::Down => self.unit_converter.decrease_active(),
            ButtonEvent::Select => {
                self.note_select_press();
                self.unit_converter.select_next_field();
            }
        }
    }

    fn apply_display(&mut self, event: ButtonEvent) {
        match event {
            ButtonEvent::Up => {
                self.display_action_selected = self
                    .display_action_selected
                    .checked_sub(1)
                    .unwrap_or(DISPLAY_ACTION_COUNT - 1);
            }
            ButtonEvent::Down => {
                self.display_action_selected =
                    (self.display_action_selected + 1) % DISPLAY_ACTION_COUNT;
            }
            ButtonEvent::Select => {
                self.note_select_press();
                match self.display_action_selected {
                    0 => self.display.cycle_font_family(),
                    _ => self.display.cycle_font_size(),
                }
            }
        }
    }

    /// Apply one Audio-overview event. Hardware requests are returned to
    /// main.rs so this product state remains independent of ESP-IDF handles.
    pub fn apply_audio_button(&mut self, event: ButtonEvent) -> Option<AudioUiRequest> {
        match event {
            ButtonEvent::Up => {
                self.audio_action_selected = self
                    .audio_action_selected
                    .checked_sub(1)
                    .unwrap_or(AUDIO_ACTION_COUNT - 1);
                None
            }
            ButtonEvent::Down => {
                self.audio_action_selected = (self.audio_action_selected + 1) % AUDIO_ACTION_COUNT;
                None
            }
            ButtonEvent::Select => {
                self.note_select_press();
                match self.audio_action_selected {
                    0 => Some(AudioUiRequest::PlayTestChime),
                    1 => Some(AudioUiRequest::StopPlayback),
                    2 => Some(AudioUiRequest::VolumeUp),
                    3 => Some(AudioUiRequest::VolumeDown),
                    4 => Some(AudioUiRequest::ToggleMute),
                    _ => {
                        self.router.navigate_to(ScreenRoute::AudioDetails);
                        None
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn category_selection(&self, route: ScreenRoute) -> usize {
        category_index(route)
            .map(|index| self.category_selected[index])
            .unwrap_or(0)
    }

    fn category_selection_mut(&mut self, route: ScreenRoute) -> &mut usize {
        let index = category_index(route).expect("category route required");
        &mut self.category_selected[index]
    }

    pub fn note_select_press(&mut self) {
        self.select_presses = self.select_presses.saturating_add(1);
    }

    /// Navigate one level toward Home. The hardware runtime calls this after a
    /// validated GPIO0 BOOT-button long press.
    pub fn back(&mut self) {
        self.router.back();
    }

    #[must_use]
    pub const fn active_route(&self) -> ScreenRoute {
        self.router.current()
    }

    pub fn update_board_snapshot(&mut self, board: BoardSnapshot) {
        self.board = board;
    }

    pub fn update_storage_snapshot(&mut self, storage: StorageSnapshot) {
        self.storage = storage;
    }

    pub fn update_network_snapshot(&mut self, network: NetworkSnapshot) {
        self.network = network;
    }

    pub fn update_weather_snapshot(&mut self, weather: WeatherSnapshot) {
        self.weather = weather;
    }

    pub fn update_alarm_snapshot(&mut self, alarms: AlarmSnapshot) {
        self.alarms = alarms;
    }

    pub fn update_audio_snapshot(&mut self, audio: AudioSnapshot) {
        self.audio = audio;
    }

    #[must_use]
    pub fn take_weather_refresh_request(&mut self) -> bool {
        core::mem::take(&mut self.weather_refresh_requested)
    }

    pub fn set_orientation(&mut self, orientation: DisplayOrientation) {
        self.orientation = orientation;
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use crate::{app::router::ScreenRoute, buttons::ButtonEvent};

    #[test]
    fn home_categories_wrap_and_open() {
        let mut state = AppState::default();
        state.apply(ButtonEvent::Up);
        assert_eq!(state.home_selected, 4);
        state.apply(ButtonEvent::Down);
        assert_eq!(state.home_selected, 0);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::Reader);
    }

    #[test]
    fn productivity_calendar_opens_and_toggles_navigation_mode() {
        use crate::calendar::CalendarNavigationMode;

        let mut state = AppState::default();
        state.home_selected = 1;
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::Productivity);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::Calendar);
        assert_eq!(state.calendar.mode, CalendarNavigationMode::Day);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.calendar.mode, CalendarNavigationMode::Month);
    }

    #[test]
    fn tools_file_browser_returns_to_tools() {
        let mut state = AppState::default();
        state.home_selected = 3;
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::Tools);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::Files);
        state.router.back();
        assert_eq!(state.active_route(), ScreenRoute::Tools);
    }

    #[test]
    fn settings_display_changes_persistent_preferences_without_a_back_row() {
        let mut state = AppState::default();
        state.home_selected = 4;
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::Settings);
        state.apply(ButtonEvent::Down);
        state.apply(ButtonEvent::Down);
        state.apply(ButtonEvent::Down);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::Display);
        let original = state.display;
        state.apply(ButtonEvent::Select);
        assert_ne!(state.display.font_family, original.font_family);
        state.apply(ButtonEvent::Down);
        state.apply(ButtonEvent::Select);
        assert_ne!(state.display.font_size, original.font_size);
        state.apply(ButtonEvent::Down);
        assert_eq!(state.display_action_selected, 0);
        assert_eq!(state.active_route(), ScreenRoute::Display);
        state.back();
        assert_eq!(state.active_route(), ScreenRoute::Settings);
    }

    #[test]
    fn weather_details_use_select_then_hierarchical_back() {
        let mut state = AppState::default();
        state.router.navigate_to(ScreenRoute::Weather);
        state.apply(ButtonEvent::Down);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::WeatherDetails);
        state.back();
        assert_eq!(state.active_route(), ScreenRoute::Weather);
    }

    #[test]
    fn clock_details_use_select_then_hierarchical_back() {
        let mut state = AppState::default();
        state.router.navigate_to(ScreenRoute::Clock);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::ClockDetails);
        state.back();
        assert_eq!(state.active_route(), ScreenRoute::Clock);
    }

    #[test]
    fn tools_unit_converter_opens_and_edits_without_hardware() {
        use crate::unit_converter::{ConverterField, UnitCategory};

        let mut state = AppState::default();
        state.home_selected = 3;
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::Tools);
        state.apply(ButtonEvent::Down);
        state.apply(ButtonEvent::Down);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::UnitConverter);
        assert_eq!(state.unit_converter.active_field, ConverterField::Category);
        state.apply(ButtonEvent::Up);
        assert_eq!(state.unit_converter.category, UnitCategory::Mass);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.unit_converter.active_field, ConverterField::FromUnit);
        state.back();
        assert_eq!(state.active_route(), ScreenRoute::Tools);
    }

    #[test]
    fn placeholder_uses_hierarchical_back_instead_of_select() {
        let mut state = AppState::default();
        state.apply(ButtonEvent::Select);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::ContinueReading);
        state.apply(ButtonEvent::Select);
        assert_eq!(state.active_route(), ScreenRoute::ContinueReading);
        state.back();
        assert_eq!(state.active_route(), ScreenRoute::Reader);
    }
}
