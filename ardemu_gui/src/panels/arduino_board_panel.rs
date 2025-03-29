use iced::{
	widget::{container, stack, svg},
	Color,
	Length::{Fill, FillPortion},
};

use crate::{
	assets::{ARDUINO_UNO_LED_BUILTIN_ON_SVG, ARDUINO_UNO_LED_POWER_ON_SVG, ARDUINO_UNO_SVG},
	style::panel_style,
	Message,
};

#[derive(Debug, Clone, Copy)]
pub struct ArduinoBoardPanel {}

impl ArduinoBoardPanel {
	pub fn new() -> Self {
		Self {}
	}

	pub fn view<'a>(&'a self, app: &'a crate::App) -> iced::Element<'a, Message> {
		let cpu = &app.cpu_sim.peek_output_buffer().cpu;

		container(
			stack![svg(ARDUINO_UNO_SVG.clone())
				.width(FillPortion(2))
				.height(Fill)]
			.push_maybe(if cpu.is_builtin_led_on() {
				Some(
					svg(ARDUINO_UNO_LED_BUILTIN_ON_SVG.clone())
						.style(|_, _| svg::Style {
							color: Some(Color::from_rgb(1.0, 1.0, 0.0)),
						})
						.width(FillPortion(2))
						.height(Fill),
				)
			} else {
				None
			})
			.push_maybe(if app.simulate_cpu {
				Some(
					svg(ARDUINO_UNO_LED_POWER_ON_SVG.clone())
						.style(|_, _| svg::Style {
							color: Some(Color::from_rgb(0.0, 1.0, 0.0)),
						})
						.width(FillPortion(2))
						.height(Fill),
				)
			} else {
				None
			}),
		)
		.style(panel_style)
		.padding(10)
		.into()
	}
}
