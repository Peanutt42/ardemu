use ardemu_core::{Imm16, PointerRegister};
use iced::{
	alignment::Vertical,
	mouse::ScrollDelta,
	widget::{
		column, container, mouse_area, responsive, row, scrollable, text, text_input, Column, Row,
	},
	Color, Element,
	Length::Fill,
	Padding, Task, Theme,
};

use crate::{
	style::{panel_style, primary_text_style, secondary_text_style},
	App, CpuSim, Message,
};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum MemoryPanelMessage {
	ChangeStartAddressInput(String),
	ChangeStartAddressFromInput,
	ChangeStartAddress(u32),
}

impl From<MemoryPanelMessage> for Message {
	fn from(value: MemoryPanelMessage) -> Self {
		Message::MemoryPanelMessage(value)
	}
}

#[derive(Debug, Clone)]
pub struct MemoryPanel {
	memory_view_start_address: u32,
	memory_view_start_address_input: Option<String>,
}

impl MemoryPanel {
	const ROW_HEIGHT: f32 = 18.0;
	const DATA_COLUMN_SPACING: f32 = 5.0;
	const BYTES_PER_ROW: u32 = 16;
	const SMALL_FONT_SIZE: f32 = 15.0;

	pub fn new() -> Self {
		Self {
			memory_view_start_address: 0,
			memory_view_start_address_input: None,
		}
	}

	pub fn update(&mut self, message: MemoryPanelMessage, _cpu_sim: &CpuSim) -> Task<Message> {
		match message {
			MemoryPanelMessage::ChangeStartAddressInput(new_input) => {
				self.memory_view_start_address_input = Some(new_input);
				Task::none()
			}
			MemoryPanelMessage::ChangeStartAddress(address) => {
				self.memory_view_start_address =
					(address / Self::BYTES_PER_ROW) * Self::BYTES_PER_ROW;
				Task::none()
			}
			MemoryPanelMessage::ChangeStartAddressFromInput => {
				if let Some(new_address) =
					self.memory_view_start_address_input
						.take()
						.and_then(|input| {
							if let Some(input) = input.strip_prefix("0x") {
								u32::from_str_radix(input, 16).ok()
							} else {
								input.parse::<u32>().ok()
							}
						}) {
					self.update(
						MemoryPanelMessage::ChangeStartAddress(new_address),
						_cpu_sim,
					)
				} else {
					Task::none()
				}
			}
		}
	}

	pub fn view<'a>(&'a self, app: &'a App) -> Element<'a, Message> {
		let cpu = &app.cpu_sim.peek_output_buffer().cpu;

		let referenced_memory_address_range = match cpu.get_current_instruction() {
			Some(instruction) => instruction.get_referenced_memory_address_range(
				cpu.get_stack_pointer(),
				cpu.read_register_pair16(PointerRegister::X),
				cpu.read_register_pair16(PointerRegister::Y),
				cpu.read_register_pair16(PointerRegister::Z),
			),
			None => None,
		};

		column![
			text("Memory:"),
			container(responsive(move |size| -> Element<Message> {
				let num_rows = (size.height / Self::ROW_HEIGHT).ceil() as usize;

				column![
					row![
						text("Go to memory: "),
						text_input(
							"0x0000",
							self.memory_view_start_address_input
								.as_ref()
								.unwrap_or(&format!(
									"{}",
									Imm16(self.memory_view_start_address as u16)
								))
						)
						.on_input(|input| MemoryPanelMessage::ChangeStartAddressInput(input).into())
						.on_submit(MemoryPanelMessage::ChangeStartAddressFromInput.into()),
					]
					.width(250.0)
					.align_y(Vertical::Center),
					row![
						container(Column::with_children((-1..num_rows as i16).map(|index| {
							match index {
								-1 => text(""),
								_ => {
									let address = Imm16(
										self.memory_view_start_address
											.saturating_add(index as u32 * Self::BYTES_PER_ROW)
											as u16,
									);
									text!("{address} ")
										.size(Self::SMALL_FONT_SIZE)
										.style(secondary_text_style)
								}
							}
							.height(Self::ROW_HEIGHT)
							.into()
						})))
						.style(|_t| container::Style {
							background: Some(Color::from_rgb(0.25, 0.25, 0.25).into()),
							..Default::default()
						}),
						scrollable(
							mouse_area(column![
								container(
									Row::with_children((0..Self::BYTES_PER_ROW).map(|index| {
										text!("{index:2x}")
											.size(Self::SMALL_FONT_SIZE)
											.style(secondary_text_style)
											.into()
									}))
									.spacing(Self::DATA_COLUMN_SPACING)
									.padding(Padding::default().left(2.5).right(2.5))
								)
								.style(|_t| container::Style {
									background: Some(Color::from_rgb(0.25, 0.25, 0.25).into()),
									..Default::default()
								}),
								Column::with_children((0..num_rows).map(|row_index| {
									let start_address = self
										.memory_view_start_address
										.saturating_add(row_index as u32 * Self::BYTES_PER_ROW);
									let end_address =
										start_address.saturating_add(Self::BYTES_PER_ROW - 1);

									let data_view: Element<Message> = match cpu
										.read_ram_range(start_address as u16..=end_address as u16)
									{
										Ok(data) => Row::with_children(
											(0..Self::BYTES_PER_ROW).map(|byte_index| {
												let memory_address = start_address + byte_index;

												let referenced =
													match &referenced_memory_address_range {
														Some(referenced_memory_address_range) => {
															referenced_memory_address_range
																.includes_address(memory_address)
														}
														None => false,
													};

												match data.get(byte_index as usize) {
													Some(byte_value) => text!("{byte_value:2x}")
														.size(Self::SMALL_FONT_SIZE)
														.style(if referenced {
															primary_text_style
														} else {
															move |_t: &Theme| text::Style::default()
														}),
													None => text("--").size(Self::SMALL_FONT_SIZE),
												}
												.into()
											}),
										)
										.spacing(Self::DATA_COLUMN_SPACING)
										.into(),
										Err(e) => text!("{e}").size(Self::SMALL_FONT_SIZE).into(),
									};

									container(data_view)
										.padding(Padding::default().left(2.5).right(2.5))
										.height(Self::ROW_HEIGHT)
										.style(move |_t| {
											if row_index % 2 == 0 {
												container::Style {
													background: Some(
														Color::from_rgb(0.1, 0.1, 0.1).into(),
													),
													..Default::default()
												}
											} else {
												container::Style::default()
											}
										})
										.into()
								}))
							])
							.on_scroll(|delta| {
								MemoryPanelMessage::ChangeStartAddress(match delta {
									ScrollDelta::Lines { y, .. } => {
										self.memory_view_start_address.saturating_add_signed(
											-y as i32 * Self::BYTES_PER_ROW as i32,
										)
									}
									ScrollDelta::Pixels { y, .. } => {
										self.memory_view_start_address.saturating_add_signed(
											(-y / Self::ROW_HEIGHT) as i32
												* Self::BYTES_PER_ROW as i32,
										)
									}
								})
								.into()
							}),
						)
						.direction(scrollable::Direction::Horizontal(
							scrollable::Scrollbar::new()
						))
					]
					.padding(10)
				]
				.into()
			}))
			.style(panel_style)
		]
		.width(Fill)
		.spacing(5)
		.into()
	}
}
