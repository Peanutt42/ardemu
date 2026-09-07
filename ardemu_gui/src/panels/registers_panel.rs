use ardemu_core::{
	Cpu, Imm8,
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

		let first_half_registors = Register::ALL.iter().take(Register::COUNT / 2);
		let second_half_registors = Register::ALL.iter().skip(Register::COUNT / 2);

		column![
			text("Registers:"),
			container(scrollable(row![
				Column::with_children(first_half_registors.map(|reg| Self::register_view(
					cpu,
					&referenced_registers,
					*reg
				)))
				.spacing(5)
				.padding(Padding::new(10.0).right(20)),
				Column::with_children(second_half_registors.map(|reg| Self::register_view(
					cpu,
					&referenced_registers,
					*reg
				)))
				.spacing(5)
				.padding(Padding::new(10.0).right(20))
			]))
			.style(panel_style)
		]
		.spacing(5)
		.into()
	}

	fn register_view<'a>(
		cpu: &'a Cpu,
		referenced_registers: &[Register],
		register: Register,
	) -> iced::Element<'a, Message> {
		let referenced = referenced_registers.contains(&register);
		let value = Imm8(cpu.read_register(register));
		let padding_space = if register <= R9 { " " } else { "" };

		row![
			text!("{padding_space}{register}: ").style(if referenced {
				primary_text_style
			} else {
				secondary_text_style
			}),
			text!("{value}")
		]
		.align_y(Vertical::Center)
		.into()
	}
}
