#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

use std::sync::mpsc::{Receiver, TryRecvError};

use ardemu_core::{parse_asm, AsmParseError, Cpu, CpuStatus, Instruction, Register};
use iced::{
	alignment::Vertical,
	border::rounded,
	mouse::ScrollDelta,
	widget::{
		button, column, container, mouse_area, responsive, row, scrollable,
		scrollable::{Direction, Scrollbar},
		text, text_editor, text_input, Column, Row,
	},
	window, Border, Color, Element, Font,
	Length::{Fill, FillPortion},
	Padding, Subscription, Theme,
};

#[allow(clippy::expect_used)]
mod highlighter;

#[derive(Debug, Clone)]
enum CpuSimMessage {
	ResetAndLoadProgram(Vec<Instruction>),
	SetSimulating(bool),
	Step,
	AddBreakpoint(u16),
	RemoveBreakpoint(u16),
}

#[derive(Debug)]
struct App {
	simulate_cpu: bool,
	cpu: triple_buffer::Output<Cpu>,
	cpu_sim_message_sender: std::sync::mpsc::Sender<CpuSimMessage>,
	memory_view_start_address: u16,
	memory_view_start_address_input: Option<String>,
	asm_source_code_text_content: text_editor::Content,
	asm_output: Result<Vec<Instruction>, AsmParseError>,
}

impl Default for App {
	fn default() -> Self {
		let asm_source_code = include_str!("count_down.asm").to_string();
		let asm_output = parse_asm(&asm_source_code);
		let cpu = match asm_output.as_ref() {
			Ok(program) => Cpu::new(program.clone()),
			Err(_) => Cpu::default(),
		};
		let (writable_cpu_buffer, readable_cpu_buffer) = triple_buffer::triple_buffer(&cpu);
		let (sender, receiver) = std::sync::mpsc::channel();

		std::thread::spawn(|| cpu_simulation_thread(cpu, receiver, writable_cpu_buffer));

		Self {
			cpu: readable_cpu_buffer,
			simulate_cpu: false,
			cpu_sim_message_sender: sender,
			memory_view_start_address: 0,
			memory_view_start_address_input: None,
			asm_source_code_text_content: text_editor::Content::with_text(&asm_source_code),
			asm_output,
		}
	}
}

#[derive(Debug, Clone)]
enum Message {
	ResetCpu,
	SimulateCpu(bool),
	Step,
	AsmSourceCodeChanged(text_editor::Action),
	UpdateCpuState,
	AddBreakpoint(u16),
	RemoveBreakpoint(u16),
	ChangeMemoryViewStartAddressInput(String),
	ChangeMemoryViewStartAddressFromInput,
	ChangeMemoryViewStartAddress(u16),
}

impl App {
	fn title(&self) -> String {
		String::from("Arduino Emulator GUI")
	}

	fn subscription(&self) -> Subscription<Message> {
		match &self.asm_output {
			Ok(_) => window::frames().map(|_| Message::UpdateCpuState),
			_ => Subscription::none(),
		}
	}

	fn send_cpu_sim_message(&mut self, message: CpuSimMessage) {
		if let Err(e) = self.cpu_sim_message_sender.send(message) {
			eprintln!("Could not send CPU sim message: {e}");
		};
	}

	fn update(&mut self, message: Message) {
		match message {
			Message::SimulateCpu(simulate_cpu) => {
				self.simulate_cpu = simulate_cpu;
				self.send_cpu_sim_message(CpuSimMessage::SetSimulating(simulate_cpu));
			}
			Message::ResetCpu => {
				self.send_cpu_sim_message(CpuSimMessage::ResetAndLoadProgram(
					self.asm_output.clone().ok().unwrap_or_default(),
				));
			}
			Message::Step => self.send_cpu_sim_message(CpuSimMessage::Step),
			Message::AsmSourceCodeChanged(action) => {
				let is_edit = action.is_edit();
				self.asm_source_code_text_content.perform(action);
				if is_edit {
					self.asm_output = parse_asm(&self.asm_source_code_text_content.text());
					self.update(Message::ResetCpu);
				}
			}
			Message::UpdateCpuState => {
				self.cpu.update();
			}
			Message::AddBreakpoint(address) => {
				self.send_cpu_sim_message(CpuSimMessage::AddBreakpoint(address));
			}
			Message::RemoveBreakpoint(address) => {
				self.send_cpu_sim_message(CpuSimMessage::RemoveBreakpoint(address));
			}
			Message::ChangeMemoryViewStartAddressInput(new_input) => {
				self.memory_view_start_address_input = Some(new_input);
			}
			Message::ChangeMemoryViewStartAddress(address) => {
				self.memory_view_start_address =
					(address / Self::BYTES_PER_ROW) * Self::BYTES_PER_ROW;
			}
			Message::ChangeMemoryViewStartAddressFromInput => {
				if let Some(new_address) =
					self.memory_view_start_address_input
						.take()
						.and_then(|input| {
							if let Some(input) = input.strip_prefix("0x") {
								u16::from_str_radix(input, 16).ok()
							} else {
								input.parse::<u16>().ok()
							}
						}) {
					self.update(Message::ChangeMemoryViewStartAddress(new_address));
				}
			}
		}
	}

	fn view(&self) -> Element<Message> {
		container(responsive(|size| {
			if size.width > size.height {
				row![self.editor_pane(false), self.simulation_pane(false),]
					.spacing(20)
					.padding(10)
					.width(Fill)
					.height(Fill)
					.into()
			} else {
				column![self.editor_pane(true), self.simulation_pane(true),]
					.spacing(20)
					.padding(10)
					.width(Fill)
					.height(Fill)
					.into()
			}
		}))
		.style(background_style)
		.width(Fill)
		.height(Fill)
		.into()
	}

	fn editor_pane(&self, portrait: bool) -> Element<Message> {
		column![
			text("Assembly Editor:"),
			container(scrollable(
				text_editor(&self.asm_source_code_text_content)
					.highlight_with::<highlighter::Highlighter>(
						highlighter::Settings {},
						highlighter::Highlight::to_format,
					)
					.font(Font::MONOSPACE)
					.style(text_editor_style)
					.on_action(Message::AsmSourceCodeChanged),
			))
			.style(panel_style)
			.width(Fill)
			.height(if portrait { FillPortion(2) } else { Fill }),
		]
		.into()
	}

	fn instructions_pane(&self, cpu: &Cpu, portrait: bool) -> Element<Message> {
		let program_counter = cpu.get_program_counter();

		column![
			text("Instructions:"),
			container(match &self.asm_output {
				Ok(asm_instructions) => {
					scrollable(
						Column::with_children(
							asm_instructions
								.iter()
								.enumerate()
								.map(|(i, instr)| {
									let address = i as u16;
									let breakpoint_set_here =
										cpu.get_breakpoints().contains(&address);
									let instr_currently_executing = program_counter == address;

									row![
										button(text!("{address:#04x}:").font(Font::MONOSPACE))
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
												if cpu.get_breakpoints().contains(&address) {
													Message::RemoveBreakpoint(address)
												} else {
													Message::AddBreakpoint(address)
												}
											),
										text!("{instr}").font(Font::MONOSPACE).color_maybe(
											if instr_currently_executing {
												Some(Color::from_rgb(1.0, 0.0, 0.0))
											} else {
												None
											}
										)
									]
									.align_y(Vertical::Center)
									.spacing(15)
									.into()
								})
								.collect::<Vec<_>>(),
						)
						.spacing(5)
						.padding(10)
						.width(Fill),
					)
					.into()
				}
				Err(e) => Element::new(container(text!("Error: {e:?}").width(Fill)).padding(10)),
			})
			.style(panel_style),
		]
		.width(if portrait { FillPortion(2) } else { Fill })
		.spacing(5)
		.into()
	}

	fn registers_pane(&self, cpu: &Cpu) -> Element<Message> {
		column![
			text("Registers:"),
			container(scrollable(
				Column::with_children(Register::ALL.iter().map(|reg| {
					let value = cpu.read_register(*reg);

					text(format!("{reg} = {value:#04x}"))
						.font(Font::MONOSPACE)
						.into()
				}))
				.spacing(10)
				.padding(10)
				.width(Fill)
			))
			.style(panel_style)
		]
		.width(Fill)
		.spacing(5)
		.into()
	}

	const ROW_HEIGHT: f32 = 18.0;
	const DATA_COLUMN_SPACING: f32 = 5.0;
	const BYTES_PER_ROW: u16 = 16;
	fn memory_pane<'a>(&'a self, cpu: &'a Cpu, portrait: bool) -> Element<'a, Message> {
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
								.unwrap_or(&format!("{:#06x}", self.memory_view_start_address))
						)
						.on_input(Message::ChangeMemoryViewStartAddressInput)
						.on_submit(Message::ChangeMemoryViewStartAddressFromInput),
					]
					.width(200.0)
					.align_y(Vertical::Center),
					row![
						container(Column::with_children((-1..num_rows as i16).map(|index| {
							match index {
								-1 => text(""),
								_ => {
									let address = self.memory_view_start_address
										+ index as u16 * Self::BYTES_PER_ROW;
									text!("{address:#06x} ").font(Font::MONOSPACE)
								}
							}
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
										text!("{index:2x}").font(Font::MONOSPACE).into()
									}))
									.spacing(Self::DATA_COLUMN_SPACING)
									.padding(Padding::default().left(2.5).right(2.5))
								)
								.style(|_t| container::Style {
									background: Some(Color::from_rgb(0.25, 0.25, 0.25).into()),
									..Default::default()
								}),
								Column::with_children((0..num_rows).map(|row_index| {
									let start_address = self.memory_view_start_address
										+ row_index as u16 * Self::BYTES_PER_ROW;
									let end_address = start_address + Self::BYTES_PER_ROW - 1;

									let data_view: Element<Message> =
										match cpu.inspect_ram_range(start_address..=end_address) {
											Ok(data) => Row::with_children(
												(0..Self::BYTES_PER_ROW).map(|byte_index| {
													let byte_value = data[byte_index as usize];

													text!("{byte_value:2x}")
														.font(Font::MONOSPACE)
														.into()
												}),
											)
											.spacing(Self::DATA_COLUMN_SPACING)
											.into(),
											Err(e) => text!("{e}").font(Font::MONOSPACE).into(),
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
								Message::ChangeMemoryViewStartAddress(match delta {
									ScrollDelta::Lines { y, .. } => {
										self.memory_view_start_address.saturating_add_signed(
											-y as i16 * Self::BYTES_PER_ROW as i16,
										)
									}
									ScrollDelta::Pixels { y, .. } => {
										self.memory_view_start_address.saturating_add_signed(
											(-y / Self::ROW_HEIGHT) as i16
												* Self::BYTES_PER_ROW as i16,
										)
									}
								})
							}),
						)
						.direction(Direction::Horizontal(Scrollbar::new()))
					]
					.padding(10)
				]
				.into()
			}))
			.style(panel_style)
		]
		.width(if portrait { FillPortion(2) } else { Fill })
		.spacing(5)
		.into()
	}

	fn simulation_pane(&self, portrait: bool) -> Element<Message> {
		let cpu = self.cpu.peek_output_buffer();

		let instruction_pane = self.instructions_pane(cpu, portrait);
		let register_pane = self.registers_pane(cpu);
		let memory_pane = self.memory_pane(cpu, portrait);

		let panes: Element<Message> = if portrait {
			row![instruction_pane, register_pane, memory_pane,]
				.spacing(20)
				.height(FillPortion(1))
				.into()
		} else {
			column![
				row![instruction_pane, register_pane,]
					.spacing(20)
					.height(Fill),
				container(memory_pane).height(Fill),
			]
			.spacing(20)
			.height(FillPortion(1))
			.into()
		};

		column![
			row![
				button("Reset CPU")
					.style(button_style)
					.on_press(Message::ResetCpu),
				button(if self.simulate_cpu {
					"Stop CPU"
				} else {
					"Start CPU"
				})
				.style(button_style)
				.on_press(Message::SimulateCpu(!self.simulate_cpu)),
				button("Step").style(button_style).on_press(Message::Step),
			]
			.align_y(Vertical::Center)
			.spacing(10),
			panes,
		]
		.spacing(20)
		.padding(10)
		.into()
	}
}

fn cpu_simulation_thread(
	mut cpu: Cpu,
	receiver: Receiver<CpuSimMessage>,
	mut writable_cpu_buffer: triple_buffer::Input<Cpu>,
) {
	let mut simulate_cpu = false;

	loop {
		if simulate_cpu {
			const BULK_STEP_COUNT: usize = 1_000_000;
			for _ in 0..BULK_STEP_COUNT {
				match cpu.step() {
					Ok(cpu_status) => match cpu_status {
						CpuStatus::Normal => {}
						CpuStatus::BreakpointHit => {
							break;
						}
						CpuStatus::ProgramFinished => {
							println!("Program finished");
							break;
						}
					},
					Err(e) => {
						eprintln!("failed to step cpu: {e}");
					}
				}
			}
		}

		writable_cpu_buffer.write(cpu.clone());

		loop {
			let simulate_cpu_copy = simulate_cpu;

			let mut handle_message = |message: CpuSimMessage| {
				match message {
					CpuSimMessage::ResetAndLoadProgram(program) => {
						cpu = Cpu::new(program);
					}
					CpuSimMessage::Step => match cpu.step() {
						Ok(cpu_status) => match cpu_status {
							CpuStatus::Normal | CpuStatus::BreakpointHit => {}
							CpuStatus::ProgramFinished => {
								println!("Program finished");
							}
						},
						Err(e) => {
							eprintln!("failed to step cpu: {e}");
						}
					},
					CpuSimMessage::SetSimulating(simulating) => {
						simulate_cpu = simulating;
					}
					CpuSimMessage::AddBreakpoint(breakpoint_address) => {
						cpu.add_breakpoint(breakpoint_address);
					}
					CpuSimMessage::RemoveBreakpoint(breakpoint_address) => {
						cpu.remove_breakpoint(breakpoint_address);
					}
				}

				writable_cpu_buffer.write(cpu.clone());
			};

			if simulate_cpu_copy {
				match receiver.try_recv() {
					Ok(message) => handle_message(message),
					Err(e) => match e {
						TryRecvError::Empty => break,
						TryRecvError::Disconnected => return,
					},
				}
			} else {
				match receiver.recv() {
					Ok(message) => handle_message(message),
					Err(_) => return,
				}
			}
		}
	}
}

fn text_editor_style(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
	let palette = theme.extended_palette();

	let active = text_editor::Style {
		background: Color::from_rgb8(1, 4, 9).into(),
		border: Border {
			radius: 8.0.into(),
			width: 0.0,
			color: palette.background.strong.color,
		},
		icon: palette.background.weak.text,
		placeholder: palette.background.strong.color,
		value: palette.background.base.text,
		selection: palette.primary.weak.color,
	};

	match status {
		text_editor::Status::Active => active,
		text_editor::Status::Hovered => text_editor::Style {
			border: Border {
				color: palette.background.base.text,
				..active.border
			},
			..active
		},
		text_editor::Status::Focused => text_editor::Style {
			border: Border {
				color: palette.primary.strong.color,
				..active.border
			},
			..active
		},
		text_editor::Status::Disabled => text_editor::Style {
			background: palette.background.weak.color.into(),
			value: active.placeholder,
			..active
		},
	}
}

fn button_style(theme: &Theme, status: button::Status) -> button::Style {
	let color_pair = match status {
		button::Status::Active => theme.extended_palette().primary.strong,
		button::Status::Hovered | button::Status::Pressed | button::Status::Disabled => {
			theme.extended_palette().primary.weak
		}
	};

	button::Style {
		background: Some(color_pair.color.into()),
		text_color: color_pair.text,
		border: rounded(8),
		..Default::default()
	}
}

fn hidden_secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
	match status {
		button::Status::Active | button::Status::Disabled => button::Style {
			background: None,
			..button::secondary(theme, status)
		},
		button::Status::Hovered | button::Status::Pressed => button::secondary(theme, status),
	}
}

fn panel_style(_theme: &Theme) -> container::Style {
	container::Style {
		background: Some(Color::from_rgb8(1, 4, 9).into()),
		border: rounded(8),
		..Default::default()
	}
}

fn background_style(_theme: &Theme) -> container::Style {
	container::Style {
		background: Some(Color::from_rgb8(5, 9, 21).into()),
		..Default::default()
	}
}

fn main() -> iced::Result {
	iced::application(App::title, App::update, App::view)
		.subscription(App::subscription)
		.run()
}
