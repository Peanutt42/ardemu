use ardemu_core::{Opcode, PointerRegister, Program, WordAddress};
use iced::{
	alignment::Vertical,
	widget::{
		button, checkbox, column, container, row, scrollable, scrollable::Direction, svg, text,
		tooltip, tooltip::Position, Column, Space,
	},
	Color, Element, Font,
	Length::{Fill, Fixed},
	Padding, Task, Theme,
};
use iced_aw::Spinner;

use crate::{
	assets::ARROW_RIGHT_SVG,
	style::{
		hidden_secondary_button_style, panel_style, primary_text_style, secondary_container_style,
		secondary_text_style, show_on_hover_button_style,
	},
	App, CpuSim, Message, ProgramState, INSTRUCTION_HEIGHT, INSTRUCTION_SCROLLABLE_ID,
	INSTRUCTION_SCROLLABLE_PADDING,
};

#[derive(Debug, Clone, Copy)]
pub enum InstructionsPanelMessage {
	SetStickToCurrentInstruction(bool),
}

impl From<InstructionsPanelMessage> for Message {
	fn from(value: InstructionsPanelMessage) -> Self {
		Message::InstructionsPanelMessage(value)
	}
}

#[derive(Debug, Clone)]
pub struct InstructionsPanel {
	stick_to_current_instruction: bool,
}

impl InstructionsPanel {
	pub fn new() -> Self {
		Self {
			stick_to_current_instruction: false,
		}
	}

	/// returns the index in the instructions list panel
	/// multiple the index by INSTRUCTION_HEIGHT to get the y position of the instruction (accounting for debug symbol info)
	fn get_instruction_index(program: &Program, address: WordAddress) -> Option<usize> {
		let mut instruction_index = 0;
		let mut program_address = WordAddress(0);
		while (program_address.0 as usize) < program.flash.len() {
			if program.get_debug_symbol(program_address).is_some() {
				// newline + debug symbol
				instruction_index += 2;
			}

			if program_address == address {
				return Some(instruction_index);
			}

			instruction_index += 1;

			if let Some(instruction) = program.get_instruction(program_address) {
				program_address += instruction.get_word_size();
			} else {
				program_address += 1;
			}
		}
		None
	}

	/// only sticks to instruction if enabled!
	pub fn stick_to_instruction(&self, cpu_sim: &CpuSim) -> Task<Message> {
		if !self.stick_to_current_instruction {
			return Task::none();
		}

		let cpu = &cpu_sim.cpu;
		match Self::get_instruction_index(cpu.get_program(), cpu.get_program_counter()) {
			Some(instruction_index) => scrollable::scroll_to(
				INSTRUCTION_SCROLLABLE_ID.clone(),
				scrollable::AbsoluteOffset {
					x: 0.0,
					y: INSTRUCTION_SCROLLABLE_PADDING
						+ INSTRUCTION_HEIGHT * instruction_index as f32,
				},
			),
			None => Task::none(),
		}
	}

	pub fn update(&mut self, message: InstructionsPanelMessage, cpu_sim: &CpuSim) -> Task<Message> {
		match message {
			InstructionsPanelMessage::SetStickToCurrentInstruction(stick) => {
				self.stick_to_current_instruction = stick;
				self.stick_to_instruction(cpu_sim)
			}
		}
	}

	pub fn view(&self, app: &App) -> Element<Message> {
		let cpu_sim = app.cpu_sim.peek_output_buffer();
		let cpu = &cpu_sim.cpu;
		let program_counter = cpu.get_program_counter();
		let potential_return_address = cpu.peek_return_address();

		let currently_referenced_program_address =
			cpu.get_current_instruction().and_then(|instruction| {
				instruction.get_referenced_program_address(
					program_counter,
					potential_return_address,
					cpu.read_register_pair16(PointerRegister::Z),
					true,
				)
			});

		column![
			row![
				text!(
					"Instructions:{}",
					if app.program_up_to_date {
						""
					} else {
						" (compile to reflect changes!)"
					}
				),
				Space::new(Fill, 0.0),
				checkbox("Stick", self.stick_to_current_instruction).on_toggle(|stick| {
					InstructionsPanelMessage::SetStickToCurrentInstruction(stick).into()
				})
			]
			.align_y(Vertical::Center),
			container(match &app.program {
				ProgramState::Compiled(program) => {
					let mut instructions: Vec<Element<Message>> = Vec::with_capacity(program.len());
					for (program_address, instruction) in program.iter() {
						let breakpoint_set_here = cpu.get_breakpoints().contains(&program_address);
						let instr_currently_executing = program_counter == program_address;

						let referenced_debug_symbol = instruction.and_then(|instruction| {
							instruction
								.get_referenced_program_address(
									program_address,
									potential_return_address,
									cpu.read_register_pair16(PointerRegister::Z),
									instr_currently_executing,
								)
								.and_then(|referenced_program_address| {
									let symbol =
										program.get_debug_symbol(referenced_program_address)?;

									Some(format!(" ; {referenced_program_address}: {symbol}"))
								})
						});
						let is_currently_referenced = match currently_referenced_program_address {
							Some(currently_referenced_program_address) => {
								currently_referenced_program_address == program_address
							}
							None => false,
						};

						if let Some(debug_symbol) = program.get_debug_symbol(program_address) {
							instructions.push(
								column![
									Space::new(0.0, INSTRUCTION_HEIGHT),
									text!("{debug_symbol}:")
										.font(Font::MONOSPACE)
										.height(INSTRUCTION_HEIGHT)
										.style(move |theme: &Theme| if is_currently_referenced {
											primary_text_style(theme)
										} else {
											text::Style::default()
										})
								]
								.into(),
							);
						}

						let instruction_view: Element<Message> = row![
							tooltip(
								button(
									svg(ARROW_RIGHT_SVG.clone())
										.style(|_t, s| svg::Style {
											color: Some(match s {
												svg::Status::Idle => Color::TRANSPARENT,
												svg::Status::Hovered => Color::WHITE,
											})
										})
										.width(16)
										.height(16)
								)
								.padding(Padding::default())
								.style(show_on_hover_button_style)
								.on_press(Message::SkipToInstruction(program_address)),
								container(text("Skip to instruction").size(12))
									.style(secondary_container_style)
									.padding(3),
								Position::Bottom,
							),
							button(text!("{program_address}:").font(Font::MONOSPACE).style(
								if is_currently_referenced {
									primary_text_style
								} else {
									secondary_text_style
								}
							))
							.style(move |t, s| {
								if breakpoint_set_here {
									if instr_currently_executing {
										button::danger(t, s)
									} else {
										button::primary(t, s)
									}
								} else {
									hidden_secondary_button_style(t, s)
								}
							})
							.padding(Padding::new(2.5).left(5.0).right(5.0))
							.on_press(
								if cpu.get_breakpoints().contains(&program_address) {
									Message::RemoveBreakpoint(program_address)
								} else {
									Message::AddBreakpoint(program_address)
								}
							),
							row![match instruction {
								Some(instruction) => text!("{instruction}").color_maybe(
									if instr_currently_executing {
										Some(Color::from_rgb(1.0, 0.0, 0.0))
									} else {
										None
									}
								),
								None => text("???"),
							}
							.font(Font::MONOSPACE)]
							.push_maybe(referenced_debug_symbol.as_ref().map(
								|referenced_debug_symbol| {
									text!("{referenced_debug_symbol}")
										.font(Font::MONOSPACE)
										.style(secondary_text_style)
								}
							))
						]
						.align_y(Vertical::Center)
						.height(INSTRUCTION_HEIGHT)
						.into();

						instructions.push(instruction_view);
					}

					scrollable(
						Column::from_vec(instructions).padding(INSTRUCTION_SCROLLABLE_PADDING),
					)
					.id(INSTRUCTION_SCROLLABLE_ID.clone())
					.direction(Direction::Both {
						vertical: scrollable::Scrollbar::default(),
						horizontal: scrollable::Scrollbar::default(),
					})
					.width(Fill)
					.height(Fill)
					.into()
				}
				ProgramState::Compiling => {
					container(
						row![
							Spinner::new()
								.width(Fixed(25.0))
								.height(Fixed(25.0))
								.circle_radius(3.0),
							text("Compiling"),
						]
						.spacing(20)
						.align_y(Vertical::Center),
					)
					.center(Fill)
					.padding(10)
					.into()
				}
				ProgramState::Error(e) => Element::new(
					scrollable(
						container(text!("Error:\n{e}").color(Color::from_rgb(1.0, 0.0, 0.0)))
							.padding(10)
					)
					.direction(Direction::Both {
						vertical: scrollable::Scrollbar::default(),
						horizontal: scrollable::Scrollbar::default(),
					})
					.width(Fill)
					.height(Fill)
				),
			})
			.style(panel_style),
		]
		.width(Fill)
		.spacing(5)
		.into()
	}
}
