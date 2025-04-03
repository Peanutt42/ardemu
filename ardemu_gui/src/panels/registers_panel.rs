use ardemu_core::{
	Imm8,
	Register::{self, R9},
};
use iced::{
	alignment::Vertical,
	widget::{column, container, row, scrollable, text, Column},
	Padding,
};

use crate::{
	style::{panel_style, primary_text_style, secondary_text_style},
	Message,
};

#[derive(Debug, Clone, Copy)]
pub struct RegistersPanel {}

impl RegistersPanel {
	pub fn new() -> Self {
		Self {}
	}

	pub fn view<'a>(&'a self, app: &'a crate::App) -> iced::Element<'a, Message> {
		let cpu = &app.cpu_sim.peek_output_buffer().cpu;

		let referenced_registers = match cpu.get_current_instruction() {
			Some(instruction) => instruction.get_referenced_registers(),
			None => Vec::new(),
		};

		column![
			text("Registers:"),
			container(scrollable(
				Column::with_children(Register::ALL.iter().map(|reg| {
					let referenced = referenced_registers.contains(reg);
					let value = Imm8(cpu.read_register(*reg));
					let padding_space = if *reg <= R9 { " " } else { "" };

					row![
						text!("{reg}: {padding_space}").style(if referenced {
							primary_text_style
						} else {
							secondary_text_style
						}),
						text!("{value}")
					]
					.align_y(Vertical::Center)
					.into()
				}))
				.spacing(10)
				.padding(Padding::new(10.0).right(20))
			))
			.style(panel_style)
		]
		.spacing(5)
		.into()
	}
}
