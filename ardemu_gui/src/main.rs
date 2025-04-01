#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

use ardemu_core::{Cpu, Program, WordAddress};
use assets::APP_ICON_PNG_BYTES;
#[cfg(target_os = "linux")]
use iced::window::settings::PlatformSpecific;
use iced::{
	alignment::Vertical,
	border::rounded,
	keyboard,
	widget::{
		button, checkbox, column, container, pick_list, responsive, row, scrollable, text,
		text_editor, tooltip, tooltip::Position, Space,
	},
	window::{self, icon},
	Element, Font,
	Length::{Fill, FillPortion},
	Subscription, Task, Theme,
};
use iced_aw::style::colors::RED;
use std::sync::LazyLock;

mod cpu_sim_thread;
use cpu_sim_thread::cpu_simulation_thread;

#[allow(clippy::expect_used)]
mod highlighter;

mod style;
use style::{
	background_style, button_style, panel_style, pick_list_menu_style, pick_list_style,
	secondary_container_style,
};

mod code_editor;

mod assets;

mod arduino_sketch;

mod code_sample;
use code_sample::CodeSample;

mod program_source;
use program_source::{ProgramSource, ProgramSourceMessage, ProgramSourceType};

mod panels;
use panels::{
	ArduinoBoardPanel, FlagsPanel, InstructionsPanel, InstructionsPanelMessage, MemoryPanel,
	MemoryPanelMessage, RegistersPanel,
};

static INSTRUCTION_SCROLLABLE_ID: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);
const INSTRUCTION_SCROLLABLE_PADDING: f32 = 10.0;
const INSTRUCTION_HEIGHT: f32 = 25.0;

#[derive(Debug, Clone)]
struct CpuSim {
	cpu: Cpu,
	cycles_per_second: f64,
}

#[derive(Debug, Clone)]
enum CpuSimMessage {
	ResetAndLoadProgram(Program),
	SetSimulating(bool),
	SetSimRealtimeSpeed(bool),
	Step,
	Skip,
	SkipToInstruction(WordAddress),
	AddBreakpoint(WordAddress),
	RemoveBreakpoint(WordAddress),
}

#[derive(Debug, Clone)]
enum ProgramState {
	Compiling,
	Compiled(Program),
	Error(String),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
enum Message {
	ResetCpu,
	SimulateCpu(bool),
	ToggleSimulateCpu,
	SetCpuSimRealtimeSpeed(bool),
	Step,
	Skip,
	SkipToInstruction(WordAddress),
	LoadProgram(Result<Program, String>),
	ChangeProgramSourceType(ProgramSourceType),
	ChangeProgramSource(ProgramSource),
	ProgramSourceMessage(ProgramSourceMessage),
	LoadCodeSample(CodeSample),
	UpdateCpuState,
	AddBreakpoint(WordAddress),
	RemoveBreakpoint(WordAddress),
	InstructionsPanelMessage(InstructionsPanelMessage),
	MemoryPanelMessage(MemoryPanelMessage),
}

#[derive(Debug)]
struct App {
	simulate_cpu: bool,
	cpu_sim_realtime_speed: bool,
	cpu_sim: triple_buffer::Output<CpuSim>,
	cpu_sim_message_sender: std::sync::mpsc::Sender<CpuSimMessage>,
	cpu_sim_dirty: bool,
	program_source: ProgramSource,
	/// whether the program is up to date with the source code
	program_up_to_date: bool,
	program: ProgramState,
	instructions_panel: InstructionsPanel,
	memory_panel: MemoryPanel,
	arduino_board_panel: ArduinoBoardPanel,
	registers_panel: RegistersPanel,
	flags_panel: FlagsPanel,
}

impl Default for App {
	fn default() -> Self {
		let code_sample = CodeSample::Fib8;
		let program = Program::default();
		let cpu = Cpu::new(program.clone());
		let cpu_sim = CpuSim {
			cpu: cpu.clone(),
			cycles_per_second: 0.0,
		};
		let (writable_cpu_sim, readable_cpu_sim) = triple_buffer::triple_buffer(&cpu_sim);
		let (sender, receiver) = std::sync::mpsc::channel();

		std::thread::spawn(move || cpu_simulation_thread(receiver, cpu, writable_cpu_sim));

		Self {
			simulate_cpu: false,
			cpu_sim_realtime_speed: false,
			cpu_sim: readable_cpu_sim,
			cpu_sim_message_sender: sender,
			cpu_sim_dirty: false,
			program_source: code_sample.get_program_source(),
			program_up_to_date: false,
			program: ProgramState::Compiled(program),
			instructions_panel: InstructionsPanel::new(),
			memory_panel: MemoryPanel::new(),
			arduino_board_panel: ArduinoBoardPanel::new(),
			registers_panel: RegistersPanel::new(),
			flags_panel: FlagsPanel::new(),
		}
	}
}

impl App {
	fn new() -> (Self, Task<Message>) {
		let mut app = App::default();
		let task = app.update(ProgramSourceMessage::Compile.into());
		(app, task)
	}

	fn title(&self) -> String {
		String::from("Arduino Emulator")
	}

	fn subscription(&self) -> Subscription<Message> {
		let update_cpu_sim_subscription = match &self.program {
			ProgramState::Compiled(_) if self.simulate_cpu => {
				window::frames().map(|_| Message::UpdateCpuState)
			}
			ProgramState::Compiling => window::frames().map(|_| Message::UpdateCpuState),
			_ => {
				if self.cpu_sim_dirty {
					window::frames().map(|_| Message::UpdateCpuState)
				} else {
					Subscription::none()
				}
			}
		};

		let keyboard_shortcuts = keyboard::on_key_press(move |key, modifiers| match key {
			keyboard::Key::Character(c) if c == "b" && modifiers.command() => {
				Some(ProgramSourceMessage::Compile.into())
			}
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
		self.instructions_panel
			.stick_to_instruction(self.cpu_sim.read())
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::SimulateCpu(simulate_cpu) => {
				self.simulate_cpu = simulate_cpu;
				self.send_cpu_sim_message(CpuSimMessage::SetSimulating(simulate_cpu))
			}
			Message::ToggleSimulateCpu => self.update(Message::SimulateCpu(!self.simulate_cpu)),
			Message::ResetCpu => self.send_cpu_sim_message(CpuSimMessage::ResetAndLoadProgram(
				match self.program.clone() {
					ProgramState::Compiled(program) => program,
					_ => Program::default(),
				},
			)),
			Message::SetCpuSimRealtimeSpeed(realtime_speed) => {
				self.cpu_sim_realtime_speed = realtime_speed;
				self.send_cpu_sim_message(CpuSimMessage::SetSimRealtimeSpeed(realtime_speed))
			}
			Message::Step => self.send_cpu_sim_message(CpuSimMessage::Step),
			Message::Skip => self.send_cpu_sim_message(CpuSimMessage::Skip),
			Message::SkipToInstruction(program_address) => {
				self.send_cpu_sim_message(CpuSimMessage::SkipToInstruction(program_address))
			}
			Message::LoadProgram(program_result) => {
				self.program = match program_result {
					Ok(program) => {
						self.program_up_to_date = true;
						ProgramState::Compiled(program)
					}
					Err(e) => ProgramState::Error(e),
				};
				self.update(Message::ResetCpu)
			}
			Message::ChangeProgramSourceType(new_program_source_type) => {
				let previous_program_source = self.program_source.clone();
				self.program_up_to_date = false;
				let change_program_source_task = match new_program_source_type {
					ProgramSourceType::Assembly => self.update(Message::ChangeProgramSource(
						ProgramSource::Assembly(text_editor::Content::new()),
					)),
					ProgramSourceType::Arduino => {
						self.update(Message::ChangeProgramSource(ProgramSource::Arduino {
							source_code_content: text_editor::Content::new(),
							arduino_cli_filepath: None,
						}))
					}
					ProgramSourceType::ElfFile => Task::perform(
						rfd::AsyncFileDialog::new()
							.add_filter("Elf (.elf)", &["elf"])
							.pick_file(),
						move |result| match result {
							Some(file_handle) => Message::ChangeProgramSource(
								ProgramSource::ElfFilepath(file_handle.path().to_path_buf()),
							),
							None => Message::ChangeProgramSource(previous_program_source.clone()),
						},
					),
					ProgramSourceType::IHexFile => Task::perform(
						rfd::AsyncFileDialog::new()
							.add_filter("IHex (.hex)", &["hex"])
							.pick_file(),
						move |result| match result {
							Some(file_handle) => Message::ChangeProgramSource(
								ProgramSource::IHexFilepath(file_handle.path().to_path_buf()),
							),
							None => Message::ChangeProgramSource(previous_program_source.clone()),
						},
					),
				};
				change_program_source_task.chain(self.update(ProgramSourceMessage::Compile.into()))
			}
			Message::ChangeProgramSource(new_program_source) => {
				self.program_source = new_program_source;
				self.update(Message::ResetCpu)
			}
			Message::ProgramSourceMessage(message) => {
				self.program_source
					.update(message, &mut self.program_up_to_date, &mut self.program)
			}
			Message::LoadCodeSample(code_sample) => {
				self.program_source = code_sample.get_program_source();
				self.program_up_to_date = false;
				self.update(ProgramSourceMessage::Compile.into())
			}
			Message::UpdateCpuState => {
				if self.cpu_sim.update() {
					self.cpu_sim_dirty = false;
					self.instructions_panel
						.stick_to_instruction(self.cpu_sim.peek_output_buffer())
				} else {
					Task::none()
				}
			}
			Message::AddBreakpoint(address) => {
				self.send_cpu_sim_message(CpuSimMessage::AddBreakpoint(address))
			}
			Message::RemoveBreakpoint(address) => {
				self.send_cpu_sim_message(CpuSimMessage::RemoveBreakpoint(address))
			}
			Message::InstructionsPanelMessage(message) => {
				self.instructions_panel.update(message, self.cpu_sim.read())
			}
			Message::MemoryPanelMessage(message) => {
				self.memory_panel.update(message, self.cpu_sim.read())
			}
		}
	}

	fn view(&self) -> Element<Message> {
		let cpu_sim = self.cpu_sim.peek_output_buffer();

		container(responsive(move |size| {
			if size.width > size.height {
				column![
					self.simulation_controls(cpu_sim),
					row![self.program_panels(false), self.simulation_panel(false),].spacing(20)
				]
				.spacing(20)
				.padding(10)
				.width(Fill)
				.height(Fill)
				.into()
			} else {
				column![
					self.simulation_controls(cpu_sim),
					self.program_panels(true),
					self.simulation_panel(true),
				]
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

	fn program_panels(&self, portrait: bool) -> Element<Message> {
		let instructions_panel_view = self.instructions_panel.view(self);

		match self.program_source.view() {
			(Some(editor_view), optional_extra_view) => {
				let compile_message_maybe = self.program_source.compile_message_maybe();

				let editor_panel: Element<Message> = column![
					row![
						text("Code Editor:  "),
						Space::new(Fill, 0.0),
						if self.program_up_to_date
							|| matches!(self.program, ProgramState::Compiling)
						{
							Element::new(Space::new(0, 0))
						} else {
							let compile_button_disabled = compile_message_maybe.is_none();
							let compile_button = button("Compile (Ctrl+B)")
								.style(button_style)
								.on_press_maybe(compile_message_maybe)
								.into();
							if compile_button_disabled {
								tooltip(
									compile_button,
									container(
										text("Set the Arduino CLI path!").size(16).color(RED),
									)
									.style(secondary_container_style)
									.padding(5),
									Position::Bottom,
								)
								.into()
							} else {
								compile_button
							}
						},
					]
					.align_y(Vertical::Center),
					container(scrollable(editor_view))
						.style(panel_style)
						.width(FillPortion(2))
						.height(Fill),
				]
				.push_maybe(optional_extra_view)
				.into();

				if portrait {
					row![editor_panel, instructions_panel_view]
						.spacing(20)
						.into()
				} else {
					column![editor_panel, instructions_panel_view]
						.spacing(20)
						.into()
				}
			}
			(None, _optional_extra_view) => instructions_panel_view,
		}
	}

	fn simulation_controls(&self, cpu_sim: &CpuSim) -> Element<Message> {
		row![
			button(if self.simulate_cpu {
				"Stop (F5)"
			} else {
				"Start (F5)"
			})
			.style(button_style)
			.on_press(Message::SimulateCpu(!self.simulate_cpu)),
			button("Step (F6)")
				.style(button_style)
				.on_press(Message::Step),
			button("Skip (F7)")
				.style(button_style)
				.on_press(Message::Skip),
			button("Reset (F8)")
				.style(button_style)
				.on_press(Message::ResetCpu),
			container(
				text!("{:.1} MHz", cpu_sim.cycles_per_second / 1_000_000.0).font(Font::MONOSPACE)
			)
			.padding(5)
			.style(move |t: &Theme| container::Style {
				background: Some(t.extended_palette().background.weak.color.into()),
				border: rounded(8),
				..Default::default()
			}),
			checkbox(
				format!("Realtime ({}MHz)", Cpu::FREQUENCY / 1_000_000),
				self.cpu_sim_realtime_speed
			)
			.on_toggle(Message::SetCpuSimRealtimeSpeed),
			Space::new(Fill, 0.0),
			pick_list(
				ProgramSourceType::ALL,
				Some(self.program_source.get_type()),
				Message::ChangeProgramSourceType
			)
			.style(pick_list_style)
			.menu_style(pick_list_menu_style),
			pick_list(CodeSample::ALL, None::<CodeSample>, Message::LoadCodeSample)
				.placeholder("Load Code Sample")
				.style(pick_list_style)
				.menu_style(pick_list_menu_style),
		]
		.align_y(Vertical::Center)
		.spacing(10)
		.padding(10)
		.into()
	}

	fn simulation_panel(&self, portrait: bool) -> Element<Message> {
		let arduino_board_panel = self.arduino_board_panel.view(self);
		let register_panel = self.registers_panel.view(self);
		let flags_panel = self.flags_panel.view(self);
		let memory_panel = self.memory_panel.view(self);

		if portrait {
			row![
				arduino_board_panel,
				register_panel,
				flags_panel,
				memory_panel,
			]
			.spacing(20)
			.height(FillPortion(1))
			.into()
		} else {
			column![
				container(arduino_board_panel).height(Fill),
				row![register_panel, flags_panel, memory_panel]
					.spacing(20)
					.height(Fill),
			]
			.spacing(20)
			.height(FillPortion(1))
			.into()
		}
	}
}

fn main() -> iced::Result {
	iced::application(App::title, App::update, App::view)
		.theme(App::theme)
		.subscription(App::subscription)
		.window(window::Settings {
			icon: icon::from_file_data(APP_ICON_PNG_BYTES, Some(image::ImageFormat::Png)).ok(),
			#[cfg(target_os = "linux")]
			platform_specific: PlatformSpecific {
				application_id: "ardemu".to_string(),
				..Default::default()
			},
			..Default::default()
		})
		.run_with(App::new)
}
