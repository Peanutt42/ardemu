#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

use std::{
	collections::HashMap,
	sync::mpsc::{Receiver, TryRecvError},
};

use ardemu_core::{parse_asm, AsmParseError, Cpu, CpuStatus, Instruction, Register};
use iced::{
	alignment::Vertical,
	border::rounded,
	widget::{
		button, checkbox, column, container, responsive, row, scrollable, text, text_editor,
		text_input, Column, Space,
	},
	window, Border, Color, Element, Font,
	Length::{Fill, FillPortion},
	Padding, Subscription, Theme,
};
use iced_fonts::{
	required::{icon_to_string, RequiredIcons},
	REQUIRED_FONT, REQUIRED_FONT_BYTES,
};

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
	cpu_sim_subscription_action_sender: std::sync::mpsc::Sender<CpuSimMessage>,
	asm_source_code_text_content: text_editor::Content,
	asm_output: Result<Vec<Instruction>, AsmParseError>,
	new_breakpoint_address_input: String,
	/// map of breakpoints and their enabled status
	breakpoints: HashMap<u16, bool>,
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
			cpu_sim_subscription_action_sender: sender,
			asm_source_code_text_content: text_editor::Content::with_text(&asm_source_code),
			asm_output,
			new_breakpoint_address_input: String::new(),
			breakpoints: HashMap::new(),
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
	ChangeNewBreakpointAddressInput(String),
	AddBreakpoint,
	RemoveBreakpoint(u16),
	SetBreakpointEnabled(u16, bool),
}

impl App {
	fn title(&self) -> String {
		String::from("Arduino Emulator GUI")
	}

	fn subscription(&self) -> Subscription<Message> {
		match &self.asm_output {
			Ok(_) => window::frames().map(|_| Message::UpdateCpuState),
			Err(_) => Subscription::none(),
		}
	}

	fn send_cpu_sim_subscription_action(&mut self, action: CpuSimMessage) {
		if let Err(e) = self.cpu_sim_subscription_action_sender.send(action) {
			eprintln!("Could not send CPU sim subscription action: {e}");
		};
	}

	fn update(&mut self, message: Message) {
		match message {
			Message::SimulateCpu(simulate_cpu) => {
				self.simulate_cpu = simulate_cpu;
				self.send_cpu_sim_subscription_action(CpuSimMessage::SetSimulating(simulate_cpu));
			}
			Message::ResetCpu => {
				self.send_cpu_sim_subscription_action(CpuSimMessage::ResetAndLoadProgram(
					self.asm_output.clone().ok().unwrap_or_default(),
				));
			}
			Message::Step => self.send_cpu_sim_subscription_action(CpuSimMessage::Step),
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
			Message::ChangeNewBreakpointAddressInput(new_breakpoint_address_input) => {
				self.new_breakpoint_address_input = new_breakpoint_address_input;
			}
			Message::AddBreakpoint => {
				let parsed_address =
					if let Some(s) = self.new_breakpoint_address_input.strip_prefix("0x") {
						u16::from_str_radix(s, 16)
					} else {
						self.new_breakpoint_address_input.parse::<u16>()
					};
				if let Ok(address) = parsed_address {
					self.breakpoints.insert(address, true);
					self.send_cpu_sim_subscription_action(CpuSimMessage::AddBreakpoint(address));
				}
				self.new_breakpoint_address_input.clear();
			}
			Message::RemoveBreakpoint(address) => {
				self.breakpoints.remove(&address);
				self.send_cpu_sim_subscription_action(CpuSimMessage::RemoveBreakpoint(address));
			}
			Message::SetBreakpointEnabled(address, enabled) => {
				self.breakpoints.insert(address, enabled);
				if enabled {
					self.send_cpu_sim_subscription_action(CpuSimMessage::AddBreakpoint(address));
				} else {
					self.send_cpu_sim_subscription_action(CpuSimMessage::RemoveBreakpoint(address));
				}
			}
		}
	}

	fn view(&self) -> Element<Message> {
		container(responsive(|size| {
			if size.width > size.height {
				row![self.editor_pane(false), self.simulation_pane(),]
					.spacing(20)
					.padding(10)
					.width(Fill)
					.height(Fill)
					.into()
			} else {
				column![self.editor_pane(true), self.simulation_pane(),]
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
					.font(Font::MONOSPACE)
					.style(text_editor_style)
					.on_action(Message::AsmSourceCodeChanged),
			))
			.style(panel_style)
			.height(if portrait { FillPortion(2) } else { Fill })
		]
		.into()
	}

	fn instructions_pane(&self, cpu: &Cpu) -> Element<Message> {
		column![
			text("Instructions:"),
			container(match &self.asm_output {
				Ok(asm_instructions) => {
					let program_counter = cpu.get_program_counter();

					scrollable(
						Column::with_children(
							asm_instructions
								.iter()
								.enumerate()
								.map(|(i, instr)| {
									text(format!("{i:#04x}: {instr}"))
										.font(Font::MONOSPACE)
										.color_maybe(if program_counter == i as u16 {
											Some(Color::from_rgb(1.0, 0.0, 0.0))
										} else {
											None
										})
										.into()
								})
								.collect::<Vec<_>>(),
						)
						.spacing(10)
						.padding(10)
						.width(Fill),
					)
					.into()
				}
				Err(e) => Element::new(container(text!("Error: {e:?}").width(Fill)).padding(10)),
			})
			.style(panel_style),
		]
		.width(FillPortion(2))
		.spacing(5)
		.into()
	}

	fn breakpoints_pane(&self, breakpoint_hit: Option<u16>) -> Element<Message> {
		column![
			text("Breakpoints:"),
			text_input("breakpoint address", &self.new_breakpoint_address_input)
				.on_input(Message::ChangeNewBreakpointAddressInput)
				.on_submit(Message::AddBreakpoint),
			container(scrollable(if self.breakpoints.is_empty() {
				Element::new(
					container(text("No breakpoints set"))
						.padding(5.0)
						.width(Fill),
				)
			} else {
				Column::with_children(self.breakpoints.iter().map(
					|(breakpoint_address, enabled)| {
						let this_breakpoint_was_hit = breakpoint_hit
							.map(|addr| addr == *breakpoint_address)
							.unwrap_or(false);

						row![
							checkbox(format!("{breakpoint_address:#04x}"), *enabled)
								.font(Font::MONOSPACE)
								.on_toggle(move |enabled| {
									Message::SetBreakpointEnabled(*breakpoint_address, enabled)
								})
								.style(move |t, s| checkbox::Style {
									text_color: if this_breakpoint_was_hit {
										Some(Color::from_rgb(1.0, 0.0, 0.0))
									} else {
										None
									},
									..checkbox::primary(t, s)
								}),
							Space::new(Fill, 0.0),
							button(text(icon_to_string(RequiredIcons::X)).font(REQUIRED_FONT))
								.padding(Padding::default().left(2.5).right(2.5))
								.on_press(Message::RemoveBreakpoint(*breakpoint_address))
						]
						.align_y(Vertical::Center)
						.into()
					},
				))
				.spacing(10)
				.padding(10)
				.width(Fill)
				.into()
			}))
			.style(panel_style),
		]
		.width(FillPortion(1))
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
		.width(FillPortion(1))
		.spacing(5)
		.into()
	}

	fn simulation_pane(&self) -> Element<Message> {
		let cpu = self.cpu.peek_output_buffer();

		let breakpoint_hit = self
			.breakpoints
			.get_key_value(&cpu.get_program_counter())
			.and_then(|(addr, enabled)| if *enabled { Some(*addr) } else { None });

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
			row![
				self.instructions_pane(cpu),
				column![
					self.breakpoints_pane(breakpoint_hit),
					self.registers_pane(cpu),
				]
				.spacing(20)
			]
			.spacing(20)
			.height(FillPortion(1))
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
			match receiver.try_recv() {
				Ok(action) => match action {
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
				},
				Err(e) => match e {
					TryRecvError::Empty => break,
					TryRecvError::Disconnected => return,
				},
			}
		}
	}
}

fn text_editor_style(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
	let palette = theme.extended_palette();

	let active = text_editor::Style {
		background: Color::from_rgb8(37, 37, 37).into(),
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

fn panel_style(_theme: &Theme) -> container::Style {
	container::Style {
		background: Some(Color::from_rgb8(37, 37, 37).into()),
		border: rounded(8),
		..Default::default()
	}
}

fn background_style(_theme: &Theme) -> container::Style {
	container::Style {
		background: Some(Color::from_rgb8(26, 26, 26).into()),
		..Default::default()
	}
}

fn main() -> iced::Result {
	iced::application(App::title, App::update, App::view)
		.subscription(App::subscription)
		.font(REQUIRED_FONT_BYTES)
		.run()
}
