//! Read-only monthly Calendar Foundation screen.

use core::convert::Infallible;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{Drawable, Point, Primitive, Size},
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
};

use crate::{
    app::{
        state::AppState,
        typography::{Text, UiTextRole},
        widgets::{
            footer::draw_footer,
            header::draw_header,
            status_row::{draw_status_row, StatusRow},
        },
    },
    calendar::{days_in_month, weekday, CalendarDate},
    orientation::OrientedFrameBuffer,
};

const GRID_LEFT: i32 = 26;
const GRID_TOP: i32 = 264;
const CELL_WIDTH: i32 = 61;
const CELL_HEIGHT: i32 = 48;
const WEEKDAY_LABELS: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
const MONTH_LABELS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Render the RTC-localized, read-only monthly Calendar page.
pub fn render_calendar(
    display: &mut OrientedFrameBuffer<'_>,
    state: &AppState,
) -> Result<(), Infallible> {
    let cursor = state.calendar.cursor;
    let month = month_label(cursor.month);
    let month_year = format!("{month} {}", cursor.year);
    let selected = selected_date_label(cursor);
    let today = state
        .board
        .rtc
        .map(|rtc| CalendarDate::from_rtc(state.regional.localize_rtc(rtc)));

    draw_header(display, state.display, "CALENDAR", "READ-ONLY MONTH VIEW")?;
    draw_status_row(
        display,
        state.display,
        StatusRow {
            left: state.calendar.mode.label(),
            middle: &month_year,
            right: "LOCAL RTC",
        },
    )?;

    Text::new(
        &month_year,
        Point::new(24, 172),
        state.display.heading_style(),
    )
    .draw(display)?;
    Text::new(
        "SELECT changes DAY / MONTH navigation.",
        Point::new(24, 204),
        state.display.body_style(),
    )
    .draw(display)?;

    for (column, label) in WEEKDAY_LABELS.iter().enumerate() {
        Text::new(
            label,
            Point::new(GRID_LEFT + column as i32 * CELL_WIDTH + 8, 244),
            state.display.detail_style(),
        )
        .draw(display)?;
    }

    draw_month_grid(display, state, cursor, today)?;

    Rectangle::new(Point::new(22, 584), Size::new(436, 108))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(display)?;
    Text::new(
        "Selected day",
        Point::new(40, 620),
        state.display.body_style(),
    )
    .draw(display)?;
    Text::new(
        &selected,
        Point::new(40, 654),
        state.display.heading_style(),
    )
    .draw(display)?;
    Text::new(
        "No events  |  Calendar is read-only",
        Point::new(40, 684),
        state.display.body_style(),
    )
    .draw(display)?;

    draw_footer(
        display,
        state.display,
        "UP/DOWN MOVE  SELECT MODE  HOLD BOOT BACK",
    )?;
    Ok(())
}

fn draw_month_grid(
    display: &mut OrientedFrameBuffer<'_>,
    state: &AppState,
    cursor: CalendarDate,
    today: Option<CalendarDate>,
) -> Result<(), Infallible> {
    let first_weekday = usize::from(weekday(cursor.year, cursor.month, 1));
    let month_days = days_in_month(cursor.year, cursor.month);

    for day in 1..=month_days {
        let index = first_weekday + usize::from(day - 1);
        let column = (index % 7) as i32;
        let row = (index / 7) as i32;
        let left = GRID_LEFT + column * CELL_WIDTH;
        let top = GRID_TOP + row * CELL_HEIGHT;
        let cell_date = CalendarDate {
            year: cursor.year,
            month: cursor.month,
            day,
        };
        let is_selected = cell_date == cursor;
        let is_today = today.is_some_and(|value| value == cell_date);
        let border = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(if is_selected { 3 } else { 1 })
            .fill_color(if is_selected {
                BinaryColor::On
            } else {
                BinaryColor::Off
            })
            .build();
        let ink = if is_selected {
            BinaryColor::Off
        } else {
            BinaryColor::On
        };

        Rectangle::new(Point::new(left, top), Size::new(54, 40))
            .into_styled(border)
            .draw(display)?;
        Text::new(
            &format!("{day:>2}"),
            Point::new(left + 14, top + 28),
            state.display.text_style(UiTextRole::Body, ink),
        )
        .draw(display)?;
        if is_today {
            Rectangle::new(Point::new(left + 43, top + 6), Size::new(5, 5))
                .into_styled(PrimitiveStyle::with_fill(ink))
                .draw(display)?;
        }
    }
    Ok(())
}

fn month_label(month: u8) -> &'static str {
    month
        .checked_sub(1)
        .and_then(|index| MONTH_LABELS.get(usize::from(index)))
        .copied()
        .unwrap_or("Unknown")
}

fn selected_date_label(date: CalendarDate) -> String {
    const WEEKDAYS: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    let weekday = WEEKDAYS
        .get(usize::from(date.weekday()))
        .copied()
        .unwrap_or("Unknown");
    format!(
        "{weekday}, {} {}, {}",
        month_label(date.month),
        date.day,
        date.year
    )
}

#[cfg(test)]
mod tests {
    use super::{month_label, selected_date_label};
    use crate::calendar::CalendarDate;

    #[test]
    fn renders_readable_selected_date() {
        let date = CalendarDate::new(2026, 6, 4).unwrap();
        assert_eq!(month_label(6), "June");
        assert_eq!(selected_date_label(date), "Thursday, June 4, 2026");
    }
}
