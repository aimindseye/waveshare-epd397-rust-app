//! Wi-Fi and SNTP overview with a separate provisioning-details page.

use core::convert::Infallible;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{Drawable, Point, Primitive, Size},
    primitives::{PrimitiveStyle, Rectangle},
};

use crate::{
    app::{
        state::AppState,
        typography::{Text, UiTextStyle},
        widgets::{
            footer::draw_footer,
            header::draw_header,
            status_row::{draw_status_row, StatusRow},
        },
    },
    network::NetworkSnapshot,
    orientation::OrientedFrameBuffer,
};

pub fn render_network(
    display: &mut OrientedFrameBuffer<'_>,
    state: &AppState,
) -> Result<(), Infallible> {
    let heading = state.display.heading_style();
    let body = state.display.body_style();
    let network = &state.network;
    let rssi = network.rssi_label();
    let last_sync = network.last_sync_label();

    draw_header(display, state.display, "NETWORK", "WI-FI AND TIME SYNC")?;
    draw_status_row(
        display,
        state.display,
        StatusRow {
            left: network.wifi_state.label(),
            middle: network.ntp_state.label(),
            right: network.home_badge(),
        },
    )?;

    Text::new("Wi-Fi station", Point::new(22, 158), heading).draw(display)?;
    line(display, 206, "State", network.wifi_state.label(), body)?;
    line(display, 246, "SSID", network.ssid_label(), body)?;
    line(display, 286, "IPv4", network.ipv4_label(), body)?;
    line(display, 326, "RSSI", &rssi, body)?;

    Text::new("Time sync", Point::new(22, 398), heading).draw(display)?;
    line(display, 446, "State", network.ntp_state.label(), body)?;
    line(display, 486, "Server", &network.ntp_server, body)?;
    line(display, 526, "Last sync", &last_sync, body)?;

    draw_action(display, 640, "Provisioning details", body)?;
    draw_footer(display, state.display, "SELECT DETAILS  HOLD BOOT BACK")?;
    Ok(())
}

pub fn render_network_details(
    display: &mut OrientedFrameBuffer<'_>,
    state: &AppState,
) -> Result<(), Infallible> {
    let heading = state.display.heading_style();
    let body = state.display.body_style();
    let detail = state.display.detail_style();
    let network = &state.network;
    let zone = state.regional.timezone_label_for_rtc(state.board.rtc);
    let error = network.error.as_deref().unwrap_or("none");

    draw_header(
        display,
        state.display,
        "NETWORK DETAILS",
        "SD-CARD PROVISIONING",
    )?;
    draw_status_row(
        display,
        state.display,
        StatusRow {
            left: network.wifi_state.label(),
            middle: network.ntp_state.label(),
            right: "DETAILS",
        },
    )?;

    Text::new("Configuration file", Point::new(22, 164), heading).draw(display)?;
    Text::new(NetworkSnapshot::config_path(), Point::new(22, 212), body).draw(display)?;
    Text::new(
        "Edit the SD-card file and reboot",
        Point::new(22, 264),
        body,
    )
    .draw(display)?;
    Text::new("to apply Wi-Fi changes.", Point::new(22, 304), body).draw(display)?;

    Text::new("Regional settings", Point::new(22, 382), heading).draw(display)?;
    line(display, 430, "Timezone", &zone, body)?;
    line(
        display,
        470,
        "RTC storage",
        &state.regional.rtc_storage_label(),
        body,
    )?;
    line(display, 510, "NTP server", &network.ntp_server, body)?;

    Text::new("Last error", Point::new(22, 588), heading).draw(display)?;
    Text::new(error, Point::new(22, 628), detail).draw(display)?;
    draw_footer(display, state.display, "HOLD BOOT BACK")?;
    Ok(())
}

fn line(
    display: &mut OrientedFrameBuffer<'_>,
    y: i32,
    label: &str,
    value: &str,
    style: UiTextStyle,
) -> Result<(), Infallible> {
    Text::new(label, Point::new(22, y), style).draw(display)?;
    Text::new(value, Point::new(176, y), style).draw(display)?;
    Ok(())
}

fn draw_action(
    display: &mut OrientedFrameBuffer<'_>,
    top: i32,
    label: &str,
    style: UiTextStyle,
) -> Result<(), Infallible> {
    Rectangle::new(Point::new(22, top), Size::new(436, 52))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 4))
        .draw(display)?;
    Text::new(">", Point::new(38, top + 34), style).draw(display)?;
    Text::new(label, Point::new(68, top + 34), style).draw(display)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{render_network, render_network_details};
    use crate::{app::AppState, framebuffer::FrameBuffer, orientation::OrientedFrameBuffer};

    #[test]
    fn network_overview_and_details_render_without_configuration() {
        let mut frame = FrameBuffer::new_white();
        let mut display = OrientedFrameBuffer::new(&mut frame, Default::default());
        let state = AppState::default();
        render_network(&mut display, &state).unwrap();
        render_network_details(&mut display, &state).unwrap();
    }
}
