#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

use ardemu::Cpu;
use iced::{
	alignment::Vertical,
	time,
	widget::{button, column, row, text, text_editor, text_input, Column},
	Color, Element, Font,
	Length::Fill,
	Subscription,
};

#[derive(Debug)]
struct App {
	cpu: Cpu,
	simulate_cpu: bool,
	simulation_frequency: u64,
	asm_source_code_text_content: text_editor::Content,
	asm_output: Result<Vec<ardemu::Instruction>, ardemu::AsmParseError>,
}

impl Default for App {
	fn default() -> Self {
		let asm_source_code = include_str!("../../src/blink.asm").to_string();
		Self {
			cpu: Cpu::default(),
			simulate_cpu: false,
			simulation_frequency: 10,
			asm_source_code_text_content: text_editor::Content::with_text(&asm_source_code),
			asm_output: ardemu::parse_asm(&asm_source_code),
		}
	}
}

#[derive(Debug, Clone)]
enum Message {
	ResetCpu,
	SimulateCpu(bool),
	SetSimulationFrequency(u64),
	InvalidSimulationFrequencyInput,
	Step,
	AsmSourceCodeChanged(text_editor::Action),
}

impl App {
	fn title(&self) -> String {
		String::from("Arduino Emulator GUI")
	}

	fn subscription(&self) -> Subscription<Message> {
		if self.simulate_cpu {
			time::every(std::time::Duration::from_secs_f64(
				1.0 / self.simulation_frequency as f64,
			))
			.map(|_| Message::Step)
		} else {
			Subscription::none()
		}
	}

	fn update(&mut self, message: Message) {
		match message {
			Message::InvalidSimulationFrequencyInput => {}
			Message::SetSimulationFrequency(frequency) => {
				self.simulation_frequency = frequency;
			}
			Message::SimulateCpu(simulate_cpu) => {
				self.simulate_cpu = simulate_cpu;
			}
			Message::ResetCpu => {
				self.cpu = Cpu::default();
			}
			Message::Step => {
				if let Ok(asm_instructions) = self.asm_output.as_ref() {
					match self.cpu.get_current_instruction(asm_instructions) {
						Some(instr) => {
							if let Err(e) = self.cpu.execute(instr) {
								eprintln!("failed to execute instruction: {e}");
							}
						}
						None => {
							println!("Program finished");
						}
					}
				}
			}
			Message::AsmSourceCodeChanged(action) => {
				let is_edit = action.is_edit();
				self.asm_source_code_text_content.perform(action);
				if is_edit {
					self.asm_output = ardemu::parse_asm(&self.asm_source_code_text_content.text());
					self.cpu = Cpu::default();
				}
			}
		}
	}

	fn view(&self) -> Element<Message> {
		column![
			text_editor(&self.asm_source_code_text_content)
				.font(Font::MONOSPACE)
				.on_action(Message::AsmSourceCodeChanged),
			row![
				button("Reset CPU").on_press(Message::ResetCpu),
				button(if self.simulate_cpu {
					"Stop CPU"
				} else {
					"Start CPU"
				})
				.on_press(Message::SimulateCpu(!self.simulate_cpu)),
				button("Step").on_press(Message::Step),
				text("Simulation Frequency (Hz)"),
				text_input("100", &self.simulation_frequency.to_string()).on_input(|input| {
					match input.parse::<u64>() {
						Ok(frequency) => Message::SetSimulationFrequency(frequency),
						Err(_) => Message::InvalidSimulationFrequencyInput,
					}
				}),
			]
			.align_y(Vertical::Center)
			.spacing(10),
			row![
				match &self.asm_output {
					Ok(asm_instructions) => {
						let program_counter = self.cpu.program_counter;

						Column::with_children(
							asm_instructions
								.iter()
								.enumerate()
								.map(|(i, instr)| {
									text(format!("{i}: {instr:?}"))
										.font(Font::MONOSPACE)
										.color_maybe(if program_counter == i {
											Some(Color::from_rgb(1.0, 0.0, 0.0))
										} else {
											None
										})
										.into()
								})
								.collect::<Vec<_>>(),
						)
						.spacing(10)
						.width(Fill)
						.into()
					}
					Err(e) => Element::new(text(format!("Error: {e:?}")).width(Fill)),
				},
				Column::with_children(self.cpu.registers.iter().enumerate().map(|(i, reg)| {
					text(format!("r{i} = {reg:#04x}"))
						.font(Font::MONOSPACE)
						.into()
				}))
				.spacing(10)
				.width(Fill),
				text!(
					"LED is {}",
					if self.cpu.is_builtin_led_on() {
						"on"
					} else {
						"off"
					}
				)
				.font(Font::MONOSPACE)
				.width(Fill),
			]
			.spacing(10)
		]
		.spacing(10)
		.into()
	}
}

fn main() -> iced::Result {
	iced::application(App::title, App::update, App::view)
		.subscription(App::subscription)
		.run()
}
