#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

use ardemu_core::{parse_asm, AsmParseError, Cpu, CpuStatus, Instruction, Register};
use iced::{
	alignment::Vertical,
	border::rounded,
	mouse::ScrollDelta,
	widget::{
		button, column, container, mouse_area, responsive, row, scrollable, text, text_editor,
		text_input, Column, Row,
	},
	window, Color, Element, Font,
	Length::{Fill, FillPortion},
	Padding, Subscription, Theme,
};
use std::{
	sync::mpsc::{Receiver, TryRecvError},
	time::Instant,
};

#[allow(clippy::expect_used)]
mod highlighter;

mod style;
use style::{
	background_style, button_style, format_big_number, hidden_secondary_button_style, panel_style,
	text_editor_style,
};

mod code_editor;
use code_editor::{code_editor_keybindings, unindent_text};

#[derive(Debug, Clone)]
struct CpuSim {
	cpu: Cpu,
	/// Instructions processed per second
	instr_per_second: usize,
}

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
	cpu_sim: triple_buffer::Output<CpuSim>,
	cpu_sim_message_sender: std::sync::mpsc::Sender<CpuSimMessage>,
	memory_view_start_address: u16,
	memory_view_start_address_input: Option<String>,
	asm_source_code_text_content: text_editor::Content,
	asm_output: Result<Vec<Instruction>, AsmParseError>,
}

impl Default for App {
	fn default() -> Self {
		let asm_source_code = include_str!("fib16.asm").to_string();
		let asm_output = parse_asm(&asm_source_code);
		let cpu = match asm_output.as_ref() {
			Ok(program) => Cpu::new(program.clone()),
			Err(_) => Cpu::default(),
		};
		let cpu_sim = CpuSim {
			cpu: cpu.clone(),
			instr_per_second: 0,
		};
		let (writable_cpu_sim, readable_cpu_sim) = triple_buffer::triple_buffer(&cpu_sim);
		let (sender, receiver) = std::sync::mpsc::channel();

		std::thread::spawn(move || cpu_simulation_thread(receiver, cpu, writable_cpu_sim));

		Self {
			cpu_sim: readable_cpu_sim,
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
	AsmSourceCodeUnindent,
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
			Message::AsmSourceCodeUnindent => {
				unindent_text(&mut self.asm_source_code_text_content);
				self.asm_output = parse_asm(&self.asm_source_code_text_content.text());
				self.update(Message::ResetCpu);
			}
			Message::UpdateCpuState => {
				self.cpu_sim.update();
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
					.on_action(Message::AsmSourceCodeChanged)
					.key_binding(move |key_press| code_editor_keybindings(
						key_press,
						Message::AsmSourceCodeUnindent
					)),
			))
			.style(panel_style)
			.width(if portrait { Fill } else { FillPortion(2) })
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
		.width(if portrait {
			FillPortion(3)
		} else {
			FillPortion(2)
		})
		.spacing(5)
		.into()
	}

	fn registers_pane(&self, cpu: &Cpu) -> Element<Message> {
		column![
			text("Registers:"),
			container(scrollable(
				Column::with_children(Register::ALL.iter().map(|reg| {
					let value = cpu.read_register(*reg);

					text(format!("{reg}: {value:#04x}"))
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

	fn flags_pane(&self, cpu: &Cpu) -> Element<Message> {
		column![
			text("Flags:"),
			container(scrollable(
				column![
					row![text!("Z: {}", cpu.flags().zero() as u8).font(Font::MONOSPACE)],
					row![text!("N: {}", cpu.flags().negative() as u8).font(Font::MONOSPACE)],
					row![text!("S: {}", cpu.flags().sign() as u8).font(Font::MONOSPACE)],
					row![text!("V: {}", cpu.flags().overflow() as u8).font(Font::MONOSPACE)],
					row![text!("H: {}", cpu.flags().half_carry() as u8).font(Font::MONOSPACE)],
					row![text!("C: {}", cpu.flags().carry() as u8).font(Font::MONOSPACE)]
				]
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
		.width(if portrait { FillPortion(3) } else { Fill })
		.spacing(5)
		.into()
	}

	fn simulation_pane(&self, portrait: bool) -> Element<Message> {
		let cpu_sim = self.cpu_sim.peek_output_buffer();
		let cpu = &cpu_sim.cpu;

		let instruction_pane = self.instructions_pane(cpu, portrait);
		let register_pane = self.registers_pane(cpu);
		let flags_pane = self.flags_pane(cpu);
		let memory_pane = self.memory_pane(cpu, portrait);

		let panes: Element<Message> = if portrait {
			row![instruction_pane, register_pane, flags_pane, memory_pane,]
				.spacing(20)
				.height(FillPortion(1))
				.into()
		} else {
			column![
				row![instruction_pane, register_pane, flags_pane]
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
				container(
					text!(
						"Instructions/sec = {}",
						format_big_number(cpu_sim.instr_per_second)
					)
					.font(Font::MONOSPACE)
				)
				.padding(5)
				.style(move |t: &Theme| container::Style {
					background: Some(t.extended_palette().background.weak.color.into()),
					border: rounded(8),
					..Default::default()
				})
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
	receiver: Receiver<CpuSimMessage>,
	mut cpu: Cpu,
	mut writable_cpu_sim: triple_buffer::Input<CpuSim>,
) {
	let mut simulate_cpu = false;

	loop {
		let start = Instant::now();
		let mut instructions_processed = 0;

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
			instructions_processed += BULK_STEP_COUNT;
		}

		writable_cpu_sim.write(CpuSim {
			cpu: cpu.clone(),
			instr_per_second: (instructions_processed as f64 / start.elapsed().as_secs_f64())
				as usize,
		});

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

				writable_cpu_sim.write(CpuSim {
					cpu: cpu.clone(),
					// single instruction is not recorded
					instr_per_second: 0,
				});
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

fn main() -> iced::Result {
	iced::application(App::title, App::update, App::view)
		.subscription(App::subscription)
		.run()
}
