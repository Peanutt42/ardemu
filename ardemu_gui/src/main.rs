#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

use ardemu_core::{
	assemble, load_elf, load_ihex_str, AsmParseError, Cpu, CpuStatus, FlagType, Imm16, Imm8,
	Program,
	Register::{self, R9},
	WordAddress,
};
use iced::{
	alignment::Vertical,
	border::rounded,
	keyboard,
	mouse::ScrollDelta,
	widget::{
		button, checkbox, column, container, mouse_area, pick_list, responsive, row, scrollable,
		scrollable::Direction, text, text_editor, text_editor::Content, text_input, Column, Row,
		Space,
	},
	window, Color, Element, Font,
	Length::{Fill, FillPortion},
	Padding, Subscription, Task, Theme,
};
use std::{
	sync::{
		mpsc::{Receiver, TryRecvError},
		LazyLock,
	},
	time::Instant,
};

#[allow(clippy::expect_used)]
mod highlighter;

mod style;
use style::{
	background_style, button_style, hidden_secondary_button_style, panel_style,
	pick_list_menu_style, pick_list_style, primary_text_style, secondary_text_style,
	text_editor_style,
};

mod code_editor;
use code_editor::{code_editor_keybindings, unindent_text};

static INSTRUCTION_SCROLLABLE_ID: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);
const INSTRUCTION_SCROLLABLE_PADDING: f32 = 10.0;
const INSTRUCTION_HEIGHT: f32 = 25.0;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum CodeSample {
	#[default]
	Fib8,
	Fib16,
	RecursiveFib,
	RustFibIHex,
	RustFibElf,
	BlinkLED,
}
impl CodeSample {
	const ALL: &'static [CodeSample] = &[
		CodeSample::Fib8,
		CodeSample::Fib16,
		CodeSample::RecursiveFib,
		CodeSample::RustFibIHex,
		CodeSample::RustFibElf,
		CodeSample::BlinkLED,
	];

	fn get_source_code(&self) -> String {
		match self {
			Self::Fib8 => format!(
				"ldi r16, 10 ; n = 10\n\n{}",
				include_str!("../../sample_programs/fib.asm")
			),
			Self::Fib16 => format!(
				"ldi r16, 10 ; n = 10\n\n{}",
				include_str!("../../sample_programs/fib16.asm")
			),
			Self::RecursiveFib => format!(
				"ldi r16, 10 ; n = 10\n\n{}",
				include_str!("../../sample_programs/recursive_fib.asm")
			),
			Self::RustFibIHex | Self::RustFibElf => {
				include_str!("../../sample_programs/rust_fib.asm").to_string()
			}
			Self::BlinkLED => include_str!("../../sample_programs/blink.asm").to_string(),
		}
	}

	#[allow(clippy::unwrap_used)]
	fn get_program(&self) -> Program {
		match self {
			Self::RustFibIHex => {
				load_ihex_str(include_str!("../../sample_programs/rust_fib.hex")).unwrap()
			}
			Self::RustFibElf => {
				load_elf(include_bytes!("../../sample_programs/rust_fib.elf")).unwrap()
			}
			_ => assemble(&self.get_source_code()).unwrap(),
		}
	}
}
impl std::fmt::Display for CodeSample {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Fib8 => "Fib 8-bit",
				Self::Fib16 => "Fib 16-bit",
				Self::RecursiveFib => "Recursive Fib",
				Self::RustFibIHex => "Rust Fib (.hex)",
				Self::RustFibElf => "Rust Fib (.elf)",
				Self::BlinkLED => "Blink LED",
			}
		)
	}
}

#[derive(Debug, Clone)]
struct CpuSim {
	cpu: Cpu,
	cycles_per_second: f64,
}

#[derive(Debug, Clone)]
enum CpuSimMessage {
	ResetAndLoadProgram(Program),
	SetSimulating(bool),
	Step,
	Skip,
	AddBreakpoint(WordAddress),
	RemoveBreakpoint(WordAddress),
}

#[derive(Debug)]
struct App {
	simulate_cpu: bool,
	cpu_sim: triple_buffer::Output<CpuSim>,
	cpu_sim_message_sender: std::sync::mpsc::Sender<CpuSimMessage>,
	cpu_sim_dirty: bool,
	stick_to_current_instruction: bool,
	memory_view_start_address: u32,
	memory_view_start_address_input: Option<String>,
	asm_source_code_text_content: text_editor::Content,
	asm_program: Result<Program, AsmParseError>,
}

impl Default for App {
	fn default() -> Self {
		let code_sample = CodeSample::default();
		let asm_program = code_sample.get_program();
		let cpu = Cpu::new(asm_program.clone());
		let cpu_sim = CpuSim {
			cpu: cpu.clone(),
			cycles_per_second: 0.0,
		};
		let (writable_cpu_sim, readable_cpu_sim) = triple_buffer::triple_buffer(&cpu_sim);
		let (sender, receiver) = std::sync::mpsc::channel();

		std::thread::spawn(move || cpu_simulation_thread(receiver, cpu, writable_cpu_sim));

		Self {
			simulate_cpu: false,
			cpu_sim: readable_cpu_sim,
			cpu_sim_message_sender: sender,
			cpu_sim_dirty: false,
			stick_to_current_instruction: false,
			memory_view_start_address: 0,
			memory_view_start_address_input: None,
			asm_source_code_text_content: text_editor::Content::with_text(
				&code_sample.get_source_code(),
			),
			asm_program: Ok(asm_program),
		}
	}
}

#[derive(Debug, Clone)]
enum Message {
	ResetCpu,
	SimulateCpu(bool),
	ToggleSimulateCpu,
	Step,
	Skip,
	SetStickToCurrentInstruction(bool),
	AsmSourceCodeChanged(text_editor::Action),
	AsmSourceCodeUnindent,
	LoadAsmCodeSample(CodeSample),
	UpdateCpuState,
	AddBreakpoint(WordAddress),
	RemoveBreakpoint(WordAddress),
	ChangeMemoryViewStartAddressInput(String),
	ChangeMemoryViewStartAddressFromInput,
	ChangeMemoryViewStartAddress(u32),
}

impl App {
	fn title(&self) -> String {
		String::from("Arduino Emulator GUI")
	}

	fn subscription(&self) -> Subscription<Message> {
		let update_cpu_sim_subscription = match &self.asm_program {
			Ok(_) if self.simulate_cpu => window::frames().map(|_| Message::UpdateCpuState),
			_ => {
				if self.cpu_sim_dirty {
					window::frames().map(|_| Message::UpdateCpuState)
				} else {
					Subscription::none()
				}
			}
		};

		let keyboard_shortcuts = keyboard::on_key_press(move |key, _modifiers| match key {
			keyboard::Key::Named(keyboard::key::Named::F5) => Some(Message::ToggleSimulateCpu),
			keyboard::Key::Named(keyboard::key::Named::F6) => Some(Message::Step),
			keyboard::Key::Named(keyboard::key::Named::F7) => Some(Message::Skip),
			keyboard::Key::Named(keyboard::key::Named::F8) => Some(Message::ResetCpu),
			_ => None,
		});

		Subscription::batch([update_cpu_sim_subscription, keyboard_shortcuts])
	}

	fn theme(&self) -> Theme {
		Theme::Dark
	}

	fn send_cpu_sim_message(&mut self, message: CpuSimMessage) -> Task<Message> {
		if let Err(e) = self.cpu_sim_message_sender.send(message) {
			eprintln!("Could not send CPU sim message: {e}");
		};
		self.cpu_sim_dirty = true;
		if self.stick_to_current_instruction {
			self.stick_to_current_instruction()
		} else {
			Task::none()
		}
	}

	fn stick_to_current_instruction(&mut self) -> Task<Message> {
		let cpu = &self.cpu_sim.read().cpu;
		match cpu
			.get_program()
			.get_instruction_index(cpu.get_program_counter())
		{
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

	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::SimulateCpu(simulate_cpu) => {
				self.simulate_cpu = simulate_cpu;
				self.send_cpu_sim_message(CpuSimMessage::SetSimulating(simulate_cpu))
			}
			Message::ToggleSimulateCpu => self.update(Message::SimulateCpu(!self.simulate_cpu)),
			Message::ResetCpu => self.send_cpu_sim_message(CpuSimMessage::ResetAndLoadProgram(
				self.asm_program.clone().ok().unwrap_or_default(),
			)),
			Message::Step => self.send_cpu_sim_message(CpuSimMessage::Step),
			Message::Skip => self.send_cpu_sim_message(CpuSimMessage::Skip),
			Message::SetStickToCurrentInstruction(stick) => {
				self.stick_to_current_instruction = stick;
				if stick {
					self.stick_to_current_instruction()
				} else {
					Task::none()
				}
			}
			Message::AsmSourceCodeChanged(action) => {
				let is_edit = action.is_edit();
				self.asm_source_code_text_content.perform(action);
				if is_edit {
					self.asm_program = assemble(&self.asm_source_code_text_content.text());
					self.update(Message::ResetCpu)
				} else {
					Task::none()
				}
			}
			Message::AsmSourceCodeUnindent => {
				unindent_text(&mut self.asm_source_code_text_content);
				self.asm_program = assemble(&self.asm_source_code_text_content.text());
				self.update(Message::ResetCpu)
			}
			Message::LoadAsmCodeSample(code_sample) => {
				self.asm_source_code_text_content =
					Content::with_text(&code_sample.get_source_code());
				self.asm_program = Ok(code_sample.get_program());
				self.update(Message::ResetCpu)
			}
			Message::UpdateCpuState => {
				if self.cpu_sim.update() {
					self.cpu_sim_dirty = false;
					if self.stick_to_current_instruction {
						return self.stick_to_current_instruction();
					}
				}
				Task::none()
			}
			Message::AddBreakpoint(address) => {
				self.send_cpu_sim_message(CpuSimMessage::AddBreakpoint(address))
			}
			Message::RemoveBreakpoint(address) => {
				self.send_cpu_sim_message(CpuSimMessage::RemoveBreakpoint(address))
			}
			Message::ChangeMemoryViewStartAddressInput(new_input) => {
				self.memory_view_start_address_input = Some(new_input);
				Task::none()
			}
			Message::ChangeMemoryViewStartAddress(address) => {
				self.memory_view_start_address =
					(address / Self::BYTES_PER_ROW) * Self::BYTES_PER_ROW;
				Task::none()
			}
			Message::ChangeMemoryViewStartAddressFromInput => {
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
					self.update(Message::ChangeMemoryViewStartAddress(new_address))
				} else {
					Task::none()
				}
			}
		}
	}

	fn view(&self) -> Element<Message> {
		container(responsive(|size| {
			if size.width > size.height {
				row![self.editor_pane(), self.simulation_pane(false),]
					.spacing(20)
					.padding(10)
					.width(Fill)
					.height(Fill)
					.into()
			} else {
				column![self.editor_pane(), self.simulation_pane(true),]
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

	fn editor_pane(&self) -> Element<Message> {
		column![
			row![
				text("Assembly Editor:"),
				Space::new(Fill, 0.0),
				pick_list(
					CodeSample::ALL,
					None::<CodeSample>,
					Message::LoadAsmCodeSample
				)
				.placeholder("Load Code Sample")
				.style(pick_list_style)
				.menu_style(pick_list_menu_style),
			]
			.align_y(Vertical::Center),
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
			.width(Fill)
			.height(Fill),
		]
		.into()
	}

	fn instructions_pane(&self, cpu_sim: &CpuSim) -> Element<Message> {
		let cpu = &cpu_sim.cpu;
		let program_counter = cpu.get_program_counter();
		let potential_return_address = cpu.peek_return_address();

		let currently_referenced_program_address =
			cpu.get_current_instruction().and_then(|instruction| {
				instruction.get_referenced_program_address(
					program_counter,
					potential_return_address,
					true,
				)
			});

		column![
			row![
				text("Instructions:"),
				Space::new(Fill, 0.0),
				checkbox("Stick", self.stick_to_current_instruction)
					.on_toggle(Message::SetStickToCurrentInstruction)
			]
			.align_y(Vertical::Center),
			container(match &self.asm_program {
				Ok(asm_program) => {
					scrollable(
						Column::with_children(
							asm_program
								.iter()
								.map(|(program_address, instruction)| {
									let breakpoint_set_here =
										cpu.get_breakpoints().contains(&program_address);
									let instr_currently_executing =
										program_counter == program_address;
									let debug_symbol =
										asm_program.get_debug_symbol(program_address);
									let referenced_debug_symbol = instruction
										.get_referenced_program_address(
											program_address,
											potential_return_address,
											instr_currently_executing,
										)
										.and_then(|referenced_program_address| {
											let symbol = asm_program
												.get_debug_symbol(referenced_program_address)?;

											Some(format!("{referenced_program_address}: {symbol}"))
										});
									let debug_info = match (debug_symbol, referenced_debug_symbol) {
										(Some(debug_symbol), Some(referenced_debug_symbol)) => {
											format!(" ; {debug_symbol}, {referenced_debug_symbol}")
										}
										(Some(debug_symbol), None) => {
											format!(" ; {debug_symbol}")
										}
										(None, Some(referenced_debug_symbol)) => {
											format!(" ; {referenced_debug_symbol}")
										}
										(None, None) => String::new(),
									};
									let is_currently_referenced =
										match currently_referenced_program_address {
											Some(currently_referenced_program_address) => {
												currently_referenced_program_address
													== program_address
											}
											None => false,
										};

									row![
										button(
											text!("{program_address}:")
												.font(Font::MONOSPACE)
												.style(if is_currently_referenced {
													primary_text_style
												} else {
													secondary_text_style
												})
										)
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
										row![
											text!("{instruction}")
												.font(Font::MONOSPACE)
												.color_maybe(if instr_currently_executing {
													Some(Color::from_rgb(1.0, 0.0, 0.0))
												} else {
													None
												}),
											text(debug_info)
												.font(Font::MONOSPACE)
												.style(secondary_text_style)
										]
									]
									.align_y(Vertical::Center)
									.height(INSTRUCTION_HEIGHT)
									.into()
								})
								.collect::<Vec<_>>(),
						)
						.padding(INSTRUCTION_SCROLLABLE_PADDING),
					)
					.id(INSTRUCTION_SCROLLABLE_ID.clone())
					.direction(Direction::Both {
						vertical: scrollable::Scrollbar::default(),
						horizontal: scrollable::Scrollbar::default(),
					})
					.width(Fill)
					.into()
				}
				Err(e) => Element::new(
					container(
						text!("Error: {e}")
							.width(Fill)
							.color(Color::from_rgb(1.0, 0.0, 0.0))
					)
					.padding(10)
				),
			})
			.style(panel_style),
		]
		.width(FillPortion(2))
		.spacing(5)
		.into()
	}

	fn registers_pane(&self, cpu: &Cpu) -> Element<Message> {
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
						text!("{reg}: {padding_space}").font(Font::MONOSPACE).style(
							if referenced {
								primary_text_style
							} else {
								secondary_text_style
							}
						),
						text!("{value}").font(Font::MONOSPACE)
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

	fn flags_pane(&self, cpu: &Cpu) -> Element<Message> {
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

	const ROW_HEIGHT: f32 = 18.0;
	const DATA_COLUMN_SPACING: f32 = 5.0;
	const BYTES_PER_ROW: u32 = 16;
	fn memory_pane<'a>(&'a self, cpu: &'a Cpu, portrait: bool) -> Element<'a, Message> {
		let referenced_memory_address_range = match cpu.get_current_instruction() {
			Some(instruction) => {
				instruction.get_referenced_memory_address_range(cpu.get_stack_pointer())
			}
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
									let address = Imm16(
										self.memory_view_start_address
											.saturating_add(index as u32 * Self::BYTES_PER_ROW)
											as u16,
									);
									text!("{address} ")
										.font(Font::MONOSPACE)
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
											.font(Font::MONOSPACE)
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
														.style(if referenced {
															primary_text_style
														} else {
															move |_t: &Theme| text::Style::default()
														}),
													None => text("--"),
												}
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

		let instruction_pane = self.instructions_pane(cpu_sim);
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
				container(instruction_pane).height(Fill),
				row![register_pane, flags_pane, memory_pane]
					.spacing(20)
					.height(Fill),
			]
			.spacing(20)
			.height(FillPortion(1))
			.into()
		};

		column![
			row![
				button("Reset")
					.style(button_style)
					.on_press(Message::ResetCpu),
				button(if self.simulate_cpu { "Stop" } else { "Start" })
					.style(button_style)
					.on_press(Message::SimulateCpu(!self.simulate_cpu)),
				button("Step").style(button_style).on_press(Message::Step),
				button("Skip").style(button_style).on_press(Message::Skip),
				container(
					text!("{:.1} MHz", cpu_sim.cycles_per_second / 1_000_000.0)
						.font(Font::MONOSPACE)
				)
				.padding(5)
				.style(move |t: &Theme| container::Style {
					background: Some(t.extended_palette().background.weak.color.into()),
					border: rounded(8),
					..Default::default()
				}),
				Space::new(Fill, 0.0),
				text!(
					"Builtin LED: {}",
					if cpu.is_builtin_led_on() {
						"HIGH"
					} else {
						"LOW"
					}
				),
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

		if simulate_cpu {
			let start_cycle = cpu.get_cycle();

			const BULK_STEP_COUNT: usize = 1_000_000;
			for _ in 0..BULK_STEP_COUNT {
				match cpu.step() {
					Ok(cpu_status) => match cpu_status {
						CpuStatus::Normal => {}
						CpuStatus::BreakpointHit | CpuStatus::BreakHit => {
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
			let cycles_per_second =
				(cpu.get_cycle() - start_cycle) as f64 / start.elapsed().as_secs_f64();
			writable_cpu_sim.write(CpuSim {
				cpu: cpu.clone(),
				cycles_per_second,
			});
		}

		loop {
			let simulate_cpu_copy = simulate_cpu;

			let mut handle_message = |message: CpuSimMessage| {
				match message {
					CpuSimMessage::ResetAndLoadProgram(program) => {
						cpu = Cpu::new(program);
					}
					CpuSimMessage::Step => match cpu.step() {
						Ok(cpu_status) => match cpu_status {
							CpuStatus::Normal | CpuStatus::BreakpointHit | CpuStatus::BreakHit => {}
							CpuStatus::ProgramFinished => {
								println!("Program finished");
							}
						},
						Err(e) => {
							eprintln!("failed to step cpu: {e}");
						}
					},
					CpuSimMessage::Skip => {
						cpu.skip();
					}
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
					cycles_per_second: 0.0,
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
		.theme(App::theme)
		.subscription(App::subscription)
		.run()
}
