use std::fmt::Write;

use ardemu_core::{Cpu, Instruction, Opcode, PointerRegister, Program, WordAddress};
use iced::{
	alignment::Vertical,
	widget::{
		button, checkbox, column, container, mouse_area, rich_text, row, scrollable,
		scrollable::Direction, svg, text, text::Span, tooltip, tooltip::Position, Column, Space,
	},
	Color, Element,
	Length::Fill,
	Padding, Task, Theme,
};

use crate::{
	assets::ARROW_RIGHT_SVG,
	style::{
		hidden_secondary_button_style, panel_style, primary_text_style, secondary_container_style,
		secondary_text_style,
	},
	App, CpuSim, Message, ProgramState, INSTRUCTION_HEIGHT, INSTRUCTION_SCROLLABLE_ID,
	INSTRUCTION_SCROLLABLE_PADDING,
};

#[derive(Debug, Clone, Copy)]
pub enum InstructionsPanelMessage {
	SetStickToCurrentInstruction(bool),
	InstructionHovered(Option<WordAddress>),
	GoToInstruction(WordAddress),
}

impl From<InstructionsPanelMessage> for Message {
	fn from(value: InstructionsPanelMessage) -> Self {
		Message::InstructionsPanelMessage(value)
	}
}

#[derive(Debug, Clone)]
pub struct InstructionsPanel {
	stick_to_current_instruction: bool,
	hovered_program_address: Option<WordAddress>,
}

impl InstructionsPanel {
	pub fn new() -> Self {
		Self {
			stick_to_current_instruction: false,
			hovered_program_address: None,
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

	fn scroll_to_instruction(&self, instruction_index: usize) -> Task<Message> {
		scrollable::scroll_to(
			INSTRUCTION_SCROLLABLE_ID.clone(),
			scrollable::AbsoluteOffset {
				x: 0.0,
				y: INSTRUCTION_SCROLLABLE_PADDING + INSTRUCTION_HEIGHT * instruction_index as f32,
			},
		)
	}

	/// only sticks to instruction if enabled!
	pub fn stick_to_instruction(&self, cpu: &Cpu) -> Task<Message> {
		if !self.stick_to_current_instruction {
			return Task::none();
		}

		match Self::get_instruction_index(cpu.get_program(), cpu.get_program_counter()) {
			Some(instruction_index) => self.scroll_to_instruction(instruction_index),
			None => Task::none(),
		}
	}

	pub fn update(&mut self, message: InstructionsPanelMessage, cpu_sim: &CpuSim) -> Task<Message> {
		match message {
			InstructionsPanelMessage::SetStickToCurrentInstruction(stick) => {
				self.stick_to_current_instruction = stick;
				self.stick_to_instruction(&cpu_sim.cpu)
			}
			InstructionsPanelMessage::InstructionHovered(program_address) => {
				self.hovered_program_address = program_address;
				Task::none()
			}
			InstructionsPanelMessage::GoToInstruction(program_address) => {
				match Self::get_instruction_index(cpu_sim.cpu.get_program(), program_address) {
					Some(instruction_index) => {
						self.scroll_to_instruction(instruction_index.saturating_sub(1))
					}
					None => Task::none(),
				}
			}
		}
	}

	pub fn view<'a>(&'a self, app: &'a App) -> Element<'a, Message> {
		const OPCODE_WIDTH: f32 = 120.0;

		let cpu_sim = app.cpu_sim.peek_output_buffer();
		let cpu = &cpu_sim.cpu;
		let program_counter = cpu.get_program_counter();
		let potential_return_address = cpu.peek_return_address().ok();

		let currently_referenced_program_address =
			cpu.get_current_instruction().and_then(|instruction| {
				potential_return_address.and_then(|potential_return_address| {
					instruction.get_referenced_program_address(
						program_counter,
						potential_return_address,
						cpu.read_register_pair16(PointerRegister::Z),
						true,
					)
				})
			});

		column![
			row![
				text!(
					"Instructions:{}",
					if app.program_up_to_date {
						""
					} else if matches!(app.program, ProgramState::Compiling { .. }) {
						" compiling..."
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

						// (program_address, symbol)
						let referenced_debug_symbol = instruction.and_then(|instruction| {
							potential_return_address.and_then(|potential_return_address| {
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

										Some((referenced_program_address, symbol))
									})
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
									text!("{debug_symbol}:").height(INSTRUCTION_HEIGHT).style(
										move |theme: &Theme| if is_currently_referenced {
											primary_text_style(theme)
										} else {
											text::Style::default()
										}
									)
								]
								.into(),
							);
						}

						let opcode_view: Element<Message> = {
							let first_opcode = program.flash[program_address.0 as usize];
							let is_32bit = instruction
								.as_ref()
								.map(Instruction::is_32bit)
								.unwrap_or(false);
							let hex_opcode = if is_32bit {
								let second_opcode = program.flash[program_address.0 as usize + 1];
								format!(
									"{}{}",
									format_opcode(first_opcode),
									format_opcode(second_opcode)
								)
							} else {
								format_opcode(first_opcode)
							};

							container(
								text(hex_opcode)
									.width(OPCODE_WIDTH)
									.style(secondary_text_style),
							)
							.padding(Padding::default().left(15.0).right(10.0))
							.into()
						};

						let instruction_view: Element<Message> = mouse_area(
							row![
								match self.hovered_program_address {
									Some(hovered_program_address)
										if program_address == hovered_program_address =>
									{
										tooltip(
											button(
												svg(ARROW_RIGHT_SVG.clone())
													.style(|_t, _s| -> svg::Style {
														svg::Style {
															color: Some(Color::WHITE),
														}
													})
													.width(16)
													.height(16),
											)
											.padding(Padding::default())
											.style(hidden_secondary_button_style)
											.on_press(Message::SkipToInstruction(program_address)),
											container(text("Skip to instruction").size(12))
												.style(secondary_container_style)
												.padding(3),
											Position::Bottom,
										)
										.into()
									}
									_ => Element::new(Space::new(16, 16)),
								},
								button(text!("{program_address}:").style(
									if is_currently_referenced {
										primary_text_style
									} else {
										secondary_text_style
									},
								),)
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
								.on_press(if cpu.get_breakpoints().contains(&program_address) {
									Message::RemoveBreakpoint(program_address)
								} else {
									Message::AddBreakpoint(program_address)
								},),
								opcode_view,
								row![match instruction {
									Some(instruction) => text!("{instruction}").color_maybe(
										if instr_currently_executing {
											Some(Color::from_rgb(1.0, 0.0, 0.0))
										} else {
											None
										}
									),
									None => text("???"),
								}]
								.push_maybe(referenced_debug_symbol.as_ref().map(
									|(referenced_program_address, referenced_debug_symbol)| {
										row![
											text!(" ; {referenced_program_address}: ")
												.style(secondary_text_style),
											rich_text![Span::new(
												(*referenced_debug_symbol).clone()
											)
											.link(Message::from(
												InstructionsPanelMessage::GoToInstruction(
													*referenced_program_address,
												)
											))]
											.style(secondary_text_style)
										]
									},
								)),
							]
							.align_y(Vertical::Center)
							.height(INSTRUCTION_HEIGHT),
						)
						.on_move(move |_point| {
							InstructionsPanelMessage::InstructionHovered(Some(program_address))
								.into()
						})
						.on_exit(InstructionsPanelMessage::InstructionHovered(None).into())
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
				ProgramState::Compiling { cli_output } => {
					scrollable(
						container(match &cli_output {
							Some(cli_output) => text(cli_output),
							None => text(""),
						})
						.padding(10),
					)
					.width(Fill)
					.height(Fill)
					.anchor_bottom()
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

fn format_opcode(opcode: u16) -> String {
	format!("{:04x}", opcode)
		.chars()
		.collect::<Vec<char>>()
		.chunks(2)
		.rev()
		.fold(String::new(), |mut result, chunk| {
			let _ = write!(&mut result, "{} ", chunk.iter().collect::<String>());
			result
		})
}
