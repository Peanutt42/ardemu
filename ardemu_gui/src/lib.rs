#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

//! The iced GUI for ardemu, buildable for both native desktop (wgpu) and the
//! browser (wasm, wgpu rendered via WebGL2). Platform-specific behaviour
//! (window icon, arduino-cli, file dialogs, background CPU thread) is gated to
//! native, and the browser gets a no-thread inline simulator. The browser entry
//! (`index.html`) shows a loading spinner until the wasm app mounts and renders.

use ardemu_core::{Cpu, Program, WordAddress};
#[cfg(target_arch = "wasm32")]
use iced::time::Instant;
use iced::{
	alignment::Vertical,
	border::rounded,
	event, keyboard,
	widget::{
		button, checkbox, column, container, pick_list, responsive, row, scrollable, space, text,
		tooltip, tooltip::Position,
	},
	window, Element, Event,
	Length::{Fill, FillPortion},
	Subscription, Task, Theme,
};
use iced_aw::style::colors::RED;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::LazyLock;

mod style;
use style::{
	background_style, button_style, panel_style, pick_list_menu_style, pick_list_style,
	secondary_container_style,
};

#[cfg(not(target_arch = "wasm32"))]
mod cpu_sim_thread;
#[cfg(not(target_arch = "wasm32"))]
use cpu_sim_thread::cpu_simulation_thread;

#[cfg(target_arch = "wasm32")]
mod wasm_file_picker;
#[cfg(target_arch = "wasm32")]
use wasm_file_picker::FileKind;

#[allow(clippy::expect_used)]
pub mod highlighter;

mod code_editor;

pub mod assets;

#[cfg(not(target_arch = "wasm32"))]
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

mod settings;
use settings::Settings;

static INSTRUCTION_SCROLLABLE_ID: LazyLock<iced::widget::Id> =
	LazyLock::new(iced::widget::Id::unique);
const INSTRUCTION_SCROLLABLE_PADDING: f32 = 10.0;
const INSTRUCTION_HEIGHT: f32 = 25.0;

#[derive(Debug, Clone)]
pub struct CpuSim {
	cpu: Cpu,
	cycles_per_second: f64,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub enum CpuSimMessage {
	ResetAndLoadProgram(Program),
	SetSimulating(bool),
	SetSimRealtimeSpeed(bool),
	Step,
	Skip,
	SkipToInstruction(WordAddress),
	AddBreakpoint(WordAddress),
	RemoveBreakpoint(WordAddress),
}

/// holds the [`Cpu`] directly (no background thread on wasm), but provides the
/// same accessor API as the native `triple_buffer::Output` so the panels can be reused.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
struct CpuSimBuffer {
	state: CpuSim,
}
#[cfg(target_arch = "wasm32")]
impl CpuSimBuffer {
	fn new(cpu: Cpu) -> Self {
		Self {
			state: CpuSim {
				cpu,
				cycles_per_second: 0.0,
			},
		}
	}

	fn peek_output_buffer(&self) -> &CpuSim {
		&self.state
	}

	fn read(&self) -> &CpuSim {
		&self.state
	}

	fn write(&mut self, cpu_sim: CpuSim) {
		self.state = cpu_sim;
	}
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum ProgramState {
	Compiling { cli_output: Option<String> },
	Compiled(Program),
	Error(String),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum Message {
	/// used to fix the window not updating after clicking on a referenced symbol
	Empty,
	#[cfg(not(target_arch = "wasm32"))]
	SetArduinoCliPath(PathBuf),
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
	ChangeAndCompileProgramSource(ProgramSource),
	ProgramSourceMessage(ProgramSourceMessage),
	LoadCodeSample(CodeSample),
	UpdateCpuState,
	AddBreakpoint(WordAddress),
	RemoveBreakpoint(WordAddress),
	InstructionsPanelMessage(InstructionsPanelMessage),
	MemoryPanelMessage(MemoryPanelMessage),
}

#[derive(Debug)]
pub struct App {
	settings: Settings,
	simulate_cpu: bool,
	cpu_sim_realtime_speed: bool,
	#[cfg(not(target_arch = "wasm32"))]
	cpu_sim: triple_buffer::Output<CpuSim>,
	#[cfg(not(target_arch = "wasm32"))]
	cpu_sim_message_sender: std::sync::mpsc::Sender<CpuSimMessage>,
	#[cfg(target_arch = "wasm32")]
	cpu_sim: CpuSimBuffer,
	#[cfg(target_arch = "wasm32")]
	last_cpu_sim_update: Option<Instant>,
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

impl App {
	pub fn new() -> (Self, Task<Message>) {
		let code_sample = CodeSample::Fib8;
		let program = Program::default();

		#[cfg(not(target_arch = "wasm32"))]
		let mut app = {
			let settings = Settings::load().unwrap_or_default();

			let cpu = Cpu::new(program.clone());
			let cpu_sim = CpuSim {
				cpu: cpu.clone(),
				cycles_per_second: 0.0,
			};
			let (writable_cpu_sim, readable_cpu_sim) = triple_buffer::triple_buffer(&cpu_sim);
			let (sender, receiver) = std::sync::mpsc::channel();

			std::thread::spawn(move || cpu_simulation_thread(receiver, cpu, writable_cpu_sim));

			Self {
				settings,
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
		};

		#[cfg(target_arch = "wasm32")]
		let mut app = {
			let cpu = Cpu::new(program.clone());

			Self {
				settings: Settings::default(),
				simulate_cpu: false,
				cpu_sim_realtime_speed: false,
				cpu_sim: CpuSimBuffer::new(cpu),
				cpu_sim_dirty: false,
				last_cpu_sim_update: None,
				program_source: code_sample.get_program_source(),
				program_up_to_date: false,
				program: ProgramState::Compiled(program),
				instructions_panel: InstructionsPanel::new(),
				memory_panel: MemoryPanel::new(),
				arduino_board_panel: ArduinoBoardPanel::new(),
				registers_panel: RegistersPanel::new(),
				flags_panel: FlagsPanel::new(),
			}
		};

		let task = app.update(ProgramSourceMessage::Compile.into());
		(app, task)
	}

	pub fn title(&self) -> String {
		String::from("Arduino Emulator")
	}

	pub fn subscription(&self) -> Subscription<Message> {
		let update_cpu_sim_subscription = match &self.program {
			ProgramState::Compiled(_) if self.simulate_cpu => {
				window::frames().map(|_| Message::UpdateCpuState)
			}
			ProgramState::Compiling { .. } => window::frames().map(|_| Message::UpdateCpuState),
			_ => {
				if self.cpu_sim_dirty {
					window::frames().map(|_| Message::UpdateCpuState)
				} else {
					Subscription::none()
				}
			}
		};

		let keyboard_shortcuts = keyboard::listen().filter_map(|event| match event {
			keyboard::Event::KeyPressed { key, modifiers, .. } => match key {
				keyboard::Key::Character(c) if c == "b" && modifiers.command() => {
					Some(ProgramSourceMessage::Compile.into())
				}
				keyboard::Key::Named(keyboard::key::Named::F5) => Some(Message::ToggleSimulateCpu),
				keyboard::Key::Named(keyboard::key::Named::F6) => Some(Message::Step),
				keyboard::Key::Named(keyboard::key::Named::F7) => Some(Message::Skip),
				keyboard::Key::Named(keyboard::key::Named::F8) => Some(Message::ResetCpu),
				_ => None,
			},
			_ => None,
		});

		// used to update window after clicking on a referenced symbol
		let update_window_on_mouse_click = event::listen_with(|event, _status, _window_id| {
			if let Event::Mouse(_) = event {
				Some(Message::Empty)
			} else {
				None
			}
		});

		Subscription::batch([
			update_cpu_sim_subscription,
			keyboard_shortcuts,
			update_window_on_mouse_click,
		])
	}

	pub fn theme(&self) -> Theme {
		Theme::Dark
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::Empty => Task::none(),
			#[cfg(not(target_arch = "wasm32"))]
			Message::SetArduinoCliPath(filepath) => {
				self.settings.arduino_cli_filepath = Some(filepath);
				self.settings.save();
				Task::none()
			}
			Message::SimulateCpu(simulate_cpu) => self.set_simulating(simulate_cpu),
			Message::ToggleSimulateCpu => self.update(Message::SimulateCpu(!self.simulate_cpu)),
			Message::ResetCpu => self.reset_cpu(),
			Message::SetCpuSimRealtimeSpeed(realtime_speed) => {
				self.set_realtime_speed(realtime_speed)
			}
			Message::Step => self.do_step(),
			Message::Skip => self.do_skip(),
			Message::SkipToInstruction(program_address) => {
				self.skip_to_instruction(program_address)
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
				match new_program_source_type {
					ProgramSourceType::Assembly => {
						self.update(Message::ChangeAndCompileProgramSource(
							ProgramSource::default_assembly_source_code(),
						))
					}
					#[cfg(not(target_arch = "wasm32"))]
					ProgramSourceType::Arduino => self.update(Message::ChangeAndCompileProgramSource(
						ProgramSource::default_arduino_sketch_source_code(),
					)),
					#[cfg(not(target_arch = "wasm32"))]
					ProgramSourceType::ElfFile => Task::perform(
						rfd::AsyncFileDialog::new()
							.add_filter("Elf (.elf)", &["elf"])
							.pick_file(),
						move |result| match result {
							Some(file_handle) => Message::ChangeAndCompileProgramSource(
								ProgramSource::ElfFilepath(file_handle.path().to_path_buf()),
							),
							None => Message::ChangeProgramSource(previous_program_source.clone()),
						},
					),
					#[cfg(not(target_arch = "wasm32"))]
					ProgramSourceType::IHexFile => Task::perform(
						rfd::AsyncFileDialog::new()
							.add_filter("IHex (.hex)", &["hex"])
							.pick_file(),
						move |result| match result {
							Some(file_handle) => Message::ChangeAndCompileProgramSource(
								ProgramSource::IHexFilepath(file_handle.path().to_path_buf()),
							),
							None => Message::ChangeProgramSource(previous_program_source.clone()),
						},
					),
					#[cfg(target_arch = "wasm32")]
					ProgramSourceType::ElfFile => wasm_file_picker::pick_and_load_file(
						FileKind::Elf,
						previous_program_source.clone(),
					),
					#[cfg(target_arch = "wasm32")]
					ProgramSourceType::IHexFile => wasm_file_picker::pick_and_load_file(
						FileKind::IHex,
						previous_program_source.clone(),
					),
				}
			}
			Message::ChangeProgramSource(new_program_source) => {
				self.program_source = new_program_source;
				self.update(Message::ResetCpu)
			}
			Message::ChangeAndCompileProgramSource(new_program_source) => self
				.update(Message::ChangeProgramSource(new_program_source))
				.chain(self.update(ProgramSourceMessage::Compile.into())),
			Message::ProgramSourceMessage(message) => self.program_source.update(
				message,
				&mut self.program_up_to_date,
				&mut self.program,
				&self.settings,
			),
			Message::LoadCodeSample(code_sample) => {
				self.program_source = code_sample.get_program_source();
				self.program_up_to_date = false;
				self.update(ProgramSourceMessage::Compile.into())
			}
			Message::UpdateCpuState => self.update_cpu_state(),
			Message::AddBreakpoint(address) => self.add_breakpoint(address),
			Message::RemoveBreakpoint(address) => self.remove_breakpoint(address),
			Message::InstructionsPanelMessage(message) => {
				self.instructions_panel.update(message, self.cpu_sim.read())
			}
			Message::MemoryPanelMessage(message) => {
				self.memory_panel.update(message, self.cpu_sim.read())
			}
		}
	}

	pub fn view(&self) -> Element<'_, Message> {
		let cpu_sim = self.cpu_sim.peek_output_buffer();

		container(responsive(move |size| {
			if size.width > size.height {
				column![
					self.simulation_controls(cpu_sim),
					row![
						container(self.program_panels(false))
							.width(FillPortion(1))
							.height(Fill),
						container(self.simulation_panel(false))
							.width(FillPortion(1))
							.height(Fill),
					]
					.spacing(20)
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

	fn program_panels(&self, portrait: bool) -> Element<'_, Message> {
		let instructions_panel_view = self.instructions_panel.view(self);

		match self.program_source.view(&self.settings) {
			(Some(editor_view), optional_extra_view) => {
				let compile_message_maybe =
					self.program_source.compile_message_maybe(&self.settings);

				let mut editor_panel = column![
					row![
						text("Code Editor:  "),
						space().width(Fill).height(1.0),
						if self.program_up_to_date
							|| matches!(self.program, ProgramState::Compiling { .. })
						{
							Element::new(space().width(0).height(0))
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
					.align_y(Vertical::Center)
					.width(Fill),
					container(scrollable(editor_view))
						.style(panel_style)
						.width(FillPortion(2))
						.height(Fill),
				];
				if let Some(extra_view) = optional_extra_view {
					editor_panel = editor_panel.push(extra_view);
				}
				let editor_panel: Element<Message> = editor_panel.into();

				if portrait {
					row![
						container(editor_panel).width(FillPortion(1)).height(Fill),
						container(instructions_panel_view)
							.width(FillPortion(1))
							.height(Fill),
					]
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

	fn simulation_controls(&self, cpu_sim: &CpuSim) -> Element<'_, Message> {
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
			container(text!(
				"{:>5.1} MHz",
				cpu_sim.cycles_per_second / 1_000_000.0
			))
			.padding(5)
			.style(move |t: &Theme| container::Style {
				background: Some(t.extended_palette().background.weak.color.into()),
				border: rounded(8),
				..Default::default()
			}),
			checkbox(self.cpu_sim_realtime_speed)
				.label(format!("Realtime ({}MHz)", Cpu::FREQUENCY / 1_000_000))
				.on_toggle(Message::SetCpuSimRealtimeSpeed),
			space().width(Fill).height(1.0),
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
		.width(Fill)
		.into()
	}

	fn simulation_panel(&self, portrait: bool) -> Element<'_, Message> {
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

#[cfg(not(target_arch = "wasm32"))]
impl App {
	fn send_cpu_sim_message(&mut self, message: CpuSimMessage) -> Task<Message> {
		if let Err(e) = self.cpu_sim_message_sender.send(message) {
			eprintln!("Could not send CPU sim message: {e}");
		};
		self.cpu_sim_dirty = true;
		self.instructions_panel
			.stick_to_instruction(&self.cpu_sim.read().cpu)
	}

	fn set_simulating(&mut self, simulating: bool) -> Task<Message> {
		self.simulate_cpu = simulating;
		self.send_cpu_sim_message(CpuSimMessage::SetSimulating(simulating))
	}

	fn reset_cpu(&mut self) -> Task<Message> {
		self.send_cpu_sim_message(CpuSimMessage::ResetAndLoadProgram(
			match self.program.clone() {
				ProgramState::Compiled(program) => program,
				_ => Program::default(),
			},
		))
	}

	fn set_realtime_speed(&mut self, realtime_speed: bool) -> Task<Message> {
		self.cpu_sim_realtime_speed = realtime_speed;
		self.send_cpu_sim_message(CpuSimMessage::SetSimRealtimeSpeed(realtime_speed))
	}

	fn do_step(&mut self) -> Task<Message> {
		self.send_cpu_sim_message(CpuSimMessage::Step)
	}

	fn do_skip(&mut self) -> Task<Message> {
		self.send_cpu_sim_message(CpuSimMessage::Skip)
	}

	fn skip_to_instruction(&mut self, address: WordAddress) -> Task<Message> {
		self.send_cpu_sim_message(CpuSimMessage::SkipToInstruction(address))
	}

	fn add_breakpoint(&mut self, address: WordAddress) -> Task<Message> {
		self.send_cpu_sim_message(CpuSimMessage::AddBreakpoint(address))
	}

	fn remove_breakpoint(&mut self, address: WordAddress) -> Task<Message> {
		self.send_cpu_sim_message(CpuSimMessage::RemoveBreakpoint(address))
	}

	fn update_cpu_state(&mut self) -> Task<Message> {
		if self.cpu_sim.update() {
			self.cpu_sim_dirty = false;
			self.instructions_panel
				.stick_to_instruction(&self.cpu_sim.peek_output_buffer().cpu)
		} else {
			Task::none()
		}
	}
}

#[cfg(target_arch = "wasm32")]
impl App {
	fn set_simulating(&mut self, simulating: bool) -> Task<Message> {
		self.simulate_cpu = simulating;
		self.cpu_sim_dirty = true;
		Task::none()
	}
	fn reset_cpu(&mut self) -> Task<Message> {
		self.cpu_sim.state.cpu = Cpu::new(match self.program.clone() {
			ProgramState::Compiled(program) => program,
			_ => Program::default(),
		});
		self.cpu_sim_dirty = true;
		Task::none()
	}
	fn set_realtime_speed(&mut self, realtime_speed: bool) -> Task<Message> {
		self.cpu_sim_realtime_speed = realtime_speed;
		self.cpu_sim_dirty = true;
		Task::none()
	}
	fn do_step(&mut self) -> Task<Message> {
		self.step_cpu();
		self.cpu_sim_dirty = true;
		Task::none()
	}
	fn do_skip(&mut self) -> Task<Message> {
		self.cpu_sim.state.cpu.skip();
		self.cpu_sim_dirty = true;
		Task::none()
	}
	fn skip_to_instruction(&mut self, address: WordAddress) -> Task<Message> {
		self.cpu_sim.state.cpu.set_program_counter(address);
		self.cpu_sim_dirty = true;
		Task::none()
	}
	fn add_breakpoint(&mut self, address: WordAddress) -> Task<Message> {
		self.cpu_sim.state.cpu.add_breakpoint(address);
		self.cpu_sim_dirty = true;
		Task::none()
	}
	fn remove_breakpoint(&mut self, address: WordAddress) -> Task<Message> {
		self.cpu_sim.state.cpu.remove_breakpoint(address);
		self.cpu_sim_dirty = true;
		Task::none()
	}
	fn update_cpu_state(&mut self) -> Task<Message> {
		if self.simulate_cpu {
			self.simulate_cpu_frame();
		}
		self.cpu_sim_dirty = false;
		Task::none()
	}

	fn step_cpu(&mut self) {
		use ardemu_core::CpuStatus;
		match self.cpu_sim.state.cpu.step() {
			Ok(cpu_status) => match cpu_status {
				CpuStatus::Normal
				| CpuStatus::BreakpointHit
				| CpuStatus::BreakHit
				| CpuStatus::ProgramFinished => {}
			},
			Err(e) => eprintln!("failed to step cpu: {e}"),
		}
	}

	/// steps the cpu (a fixed amount per call) and records the result in the buffer.
	fn simulate_cpu_frame(&mut self) {
		let now = Instant::now();
		let start = self.cpu_sim.state.cpu.get_cycle();
		let elapsed = self
			.last_cpu_sim_update
			.map(|last| now.saturating_duration_since(last))
			.unwrap_or_default();
		self.last_cpu_sim_update = Some(now);

		if self.cpu_sim_realtime_speed {
			// 16 MHz realtime is far too fast to emulate step-for-step in a browser,
			// so we cap the number of steps per frame to keep the UI responsive.
			const MAX_STEPS_PER_FRAME: u64 = 1_000_000;
			let target_cycles = (Cpu::FREQUENCY as f64 * elapsed.as_secs_f64()) as u64;
			let steps = target_cycles.min(MAX_STEPS_PER_FRAME);
			for _ in 0..steps {
				self.step_cpu();
			}
		} else {
			// bulk/fast mode
			const BULK_STEP_COUNT: u64 = 100_000;
			for _ in 0..BULK_STEP_COUNT {
				self.step_cpu();
			}
		}

		let cycles_per_second =
			(self.cpu_sim.state.cpu.get_cycle() - start) as f64 / now.elapsed().as_secs_f64();
		self.cpu_sim.write(CpuSim {
			cpu: self.cpu_sim.state.cpu.clone(),
			cycles_per_second,
		});
	}
}
