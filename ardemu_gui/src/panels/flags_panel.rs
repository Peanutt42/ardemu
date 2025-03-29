use ardemu_core::FlagType;
use iced::{
	widget::{column, container, scrollable, text, Column},
	Color, Font, Theme,
};

use crate::{
	style::{panel_style, secondary_text_style},
	Message,
};

#[derive(Debug, Clone, Copy)]
pub struct FlagsPanel {}

impl FlagsPanel {
	pub fn new() -> Self {
		Self {}
	}

	pub fn view<'a>(&'a self, app: &'a crate::App) -> iced::Element<'a, Message> {
		let cpu = &app.cpu_sim.peek_output_buffer().cpu;

		column![
			text("Flags:"),
			container(scrollable(
				Column::with_children(FlagType::ALL.iter().map(|flag_type| {
					let flag_set = cpu.flags().get(*flag_type);

					container(
						text!("{flag_type}: {}", flag_set as u8)
							.font(Font::MONOSPACE)
							.style(if flag_set {
								move |_t: &Theme| text::Style {
									color: Some(Color::WHITE),
								}
							} else {
								secondary_text_style
							}),
					)
					.style(move |t: &Theme| {
						if flag_set {
							container::Style {
								background: Some(t.extended_palette().primary.base.color.into()),
								..Default::default()
							}
						} else {
							container::Style::default()
						}
					})
					.into()
				}))
				.spacing(10)
				.padding(10) //.width(Fill)
			))
			.style(panel_style)
		]
		//.width(Fill)
		.spacing(5)
		.into()
	}
}
