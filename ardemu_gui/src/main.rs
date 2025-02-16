#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

use std::sync::mpsc::{Receiver, TryRecvError};

use ardemu_core::{parse_asm, AsmParseError, Cpu, Instruction, Register};
use iced::{
	alignment::Vertical,
	border::rounded,
	widget::{button, column, container, responsive, row, scrollable, text, text_editor, Column},
	window, Border, Color, Element, Font,
	Length::{Fill, FillPortion},
	Subscription, Theme,
};

#[derive(Debug, Clone)]
enum CpuSimMessage {
	ResetAndLoadProgram(Vec<Instruction>),
	SetSimulating(bool),
	Step,
}

#[derive(Debug)]
struct App {
	simulate_cpu: bool,
	cpu: triple_buffer::Output<Cpu>,
	cpu_sim_subscription_action_sender: std::sync::mpsc::Sender<CpuSimMessage>,
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
			cpu_sim_subscription_action_sender: sender,
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

	fn simulation_pane(&self) -> Element<Message> {
		let cpu = self.cpu.peek_output_buffer();

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
						Err(e) =>
							Element::new(container(text!("Error: {e:?}").width(Fill)).padding(10)),
					})
					.style(panel_style),
				]
				.spacing(5),
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
				.spacing(5),
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
					Ok(continue_execution) => {
						if !continue_execution {
							println!("Program finished");
							break;
						}
					}
					Err(e) => {
						eprintln!("failed to step cpu: {e}");
					}
				}
			}
		}

		loop {
			match receiver.try_recv() {
				Ok(action) => match action {
					CpuSimMessage::ResetAndLoadProgram(program) => {
						cpu = Cpu::new(program);
					}
					CpuSimMessage::Step => match cpu.step() {
						Ok(continue_execution) => {
							if !continue_execution {
								println!("Program finished");
							}
						}
						Err(e) => {
							eprintln!("failed to step cpu: {e}");
						}
					},
					CpuSimMessage::SetSimulating(simulating) => {
						simulate_cpu = simulating;
					}
				},
				Err(e) => match e {
					TryRecvError::Empty => break,
					TryRecvError::Disconnected => return,
				},
			}
		}

		writable_cpu_buffer.write(cpu.clone());
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
		.run()
}
