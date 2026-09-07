#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use ardemu_core::{assemble, load_elf, load_ihex_str};
#[cfg(not(target_arch = "wasm32"))]
use iced::alignment::Vertical;
use iced::{widget::text_editor, Element, Task};
#[cfg(not(target_arch = "wasm32"))]
use iced::{
	widget::{
		button, container, row, scrollable,
		scrollable::{Direction, Scrollbar},
		text,
	},
	Length::Fill,
	Padding,
};
#[cfg(not(target_arch = "wasm32"))]
use iced_aw::style::colors::RED;

#[cfg(not(target_arch = "wasm32"))]
use crate::{arduino_sketch::compile_arduino_sketch_stream, style::button_style};
use crate::{
	code_editor::{code_editor_keybindings, unindent_text},
	highlighter,
	settings::Settings,
	style::text_editor_style,
	Message, ProgramState,
};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ProgramSourceType {
	#[default]
	Assembly,
	#[cfg(not(target_arch = "wasm32"))]
	Arduino,
	ElfFile,
	IHexFile,
}
impl ProgramSourceType {
	pub const ALL: &'static [ProgramSourceType] = &[
		Self::Assembly,
		#[cfg(not(target_arch = "wasm32"))]
		Self::Arduino,
		Self::ElfFile,
		Self::IHexFile,
	];
}
impl std::fmt::Display for ProgramSourceType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Assembly => "Assembly",
				#[cfg(not(target_arch = "wasm32"))]
				Self::Arduino => "Arduino",
				Self::ElfFile => "Elf File",
				Self::IHexFile => "IHex File",
			}
		)
	}
}

#[derive(Debug, Clone)]
pub enum ProgramSourceMessage {
	Compile,
	#[cfg(not(target_arch = "wasm32"))]
	CompileCliOutput(String),
	AssemblySourceCodeChanged(text_editor::Action),
	AssemblySourceCodeUnindent,
	#[cfg(not(target_arch = "wasm32"))]
	ArduinoSourceCodeChanged(text_editor::Action),
	#[cfg(not(target_arch = "wasm32"))]
	ArduinoSourceCodeUnindent,
	#[cfg(not(target_arch = "wasm32"))]
	BrowseArduinoCliDialog,
	#[cfg(not(target_arch = "wasm32"))]
	BrowseArduinoCliDialogCanceled,
}

impl From<ProgramSourceMessage> for Message {
	fn from(value: ProgramSourceMessage) -> Self {
		Message::ProgramSourceMessage(value)
	}
}

#[derive(Debug)]
pub enum ProgramSource {
	Assembly(text_editor::Content),
	#[cfg(not(target_arch = "wasm32"))]
	Arduino(text_editor::Content),
	#[cfg(not(target_arch = "wasm32"))]
	ElfFilepath(PathBuf),
	#[cfg(not(target_arch = "wasm32"))]
	IHexFilepath(PathBuf),
	ElfFile(Vec<u8>),
	IHexFile(String),
}
impl ProgramSource {
	pub fn get_type(&self) -> ProgramSourceType {
		match self {
			Self::Assembly(_) => ProgramSourceType::Assembly,
			#[cfg(not(target_arch = "wasm32"))]
			Self::Arduino(_) => ProgramSourceType::Arduino,
			#[cfg(not(target_arch = "wasm32"))]
			Self::ElfFilepath(_) => ProgramSourceType::ElfFile,
			#[cfg(not(target_arch = "wasm32"))]
			Self::IHexFilepath(_) => ProgramSourceType::IHexFile,
			Self::ElfFile(_) => ProgramSourceType::ElfFile,
			Self::IHexFile(_) => ProgramSourceType::IHexFile,
		}
	}

	pub fn default_assembly_source_code() -> Self {
		Self::Assembly(text_editor::Content::new())
	}

	#[cfg(not(target_arch = "wasm32"))]
	pub fn default_arduino_sketch_source_code() -> Self {
		Self::Arduino(text_editor::Content::with_text(
			r"
void setup() {
  // put your setup code here, to run once:

}

void loop() {
  // put your main code here, to run repeatedly:

}",
		))
	}

	#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
	pub fn update(
		&mut self,
		message: ProgramSourceMessage,
		is_program_up_to_date: &mut bool,
		program: &mut ProgramState,
		settings: &Settings,
	) -> Task<Message> {
		#[cfg(not(target_arch = "wasm32"))]
		let map_blocking_task_error = |e| format!("Failed to join async blocking 'compile' task: {e}");

		match message {
			ProgramSourceMessage::Compile => {
				*program = ProgramState::Compiling {
					#[cfg(not(target_arch = "wasm32"))]
					cli_output: if matches!(self, Self::Arduino(_)) {
						Some(String::new())
					} else {
						None
					},
					#[cfg(target_arch = "wasm32")]
					cli_output: None,
				};
				match self {
					Self::Assembly(source_code_content) => {
						let source_code = source_code_content.text();
						#[cfg(not(target_arch = "wasm32"))]
						{
							Task::perform(
								async move {
									tokio::task::spawn_blocking(move || {
										assemble(&source_code).map_err(|e| e.to_string())
									})
									.await
									.map_err(map_blocking_task_error)?
								},
								Message::LoadProgram,
							)
						}
						#[cfg(target_arch = "wasm32")]
						{
							Task::perform(
								async move { assemble(&source_code).map_err(|e| e.to_string()) },
								Message::LoadProgram,
							)
						}
					}
					#[cfg(not(target_arch = "wasm32"))]
					Self::Arduino(source_code_content) => match &settings.arduino_cli_filepath {
						Some(arduino_cli_filepath) => {
							let source_code = source_code_content.text();
							let arduino_cli_filepath = arduino_cli_filepath.clone();

							Task::stream(compile_arduino_sketch_stream(
								source_code,
								arduino_cli_filepath,
							))
						}
						None => Task::done(Message::LoadProgram(Err(
							"arduino cli filepath not set!".to_string(),
						))),
					},
					#[cfg(not(target_arch = "wasm32"))]
					Self::ElfFilepath(filepath) => {
						let filepath = filepath.clone();
						Task::perform(
							async move {
								let elf_file_content =
									tokio::fs::read(filepath).await.map_err(|e| e.to_string())?;

								tokio::task::spawn_blocking(move || {
									load_elf(&elf_file_content).map_err(|e| e.to_string())
								})
								.await
								.map_err(map_blocking_task_error)?
							},
							Message::LoadProgram,
						)
					}
					Self::ElfFile(elf_file_content) => {
						let elf_file_content = elf_file_content.clone();
						#[cfg(not(target_arch = "wasm32"))]
						{
							Task::perform(
								async move {
									tokio::task::spawn_blocking(move || {
										load_elf(&elf_file_content).map_err(|e| e.to_string())
									})
									.await
									.map_err(map_blocking_task_error)?
								},
								Message::LoadProgram,
							)
						}
						#[cfg(target_arch = "wasm32")]
						{
							Task::perform(
								async move { load_elf(&elf_file_content).map_err(|e| e.to_string()) },
								Message::LoadProgram,
							)
						}
					}
					#[cfg(not(target_arch = "wasm32"))]
					Self::IHexFilepath(filepath) => {
						let filepath = filepath.clone();
						Task::perform(
							async move {
								let ihex_file_content = tokio::fs::read_to_string(filepath)
									.await
									.map_err(|e| e.to_string())?;

								tokio::task::spawn_blocking(move || {
									load_ihex_str(&ihex_file_content).map_err(|e| e.to_string())
								})
								.await
								.map_err(map_blocking_task_error)?
							},
							Message::LoadProgram,
						)
					}
					Self::IHexFile(ihex_file_content) => {
						let ihex_file_content = ihex_file_content.clone();
						#[cfg(not(target_arch = "wasm32"))]
						{
							Task::perform(
								async move {
									tokio::task::spawn_blocking(move || {
										load_ihex_str(&ihex_file_content).map_err(|e| e.to_string())
									})
									.await
									.map_err(map_blocking_task_error)?
								},
								Message::LoadProgram,
							)
						}
						#[cfg(target_arch = "wasm32")]
						{
							Task::perform(
								async move {
									load_ihex_str(&ihex_file_content).map_err(|e| e.to_string())
								},
								Message::LoadProgram,
							)
						}
					}
				}
			}
			#[cfg(not(target_arch = "wasm32"))]
			ProgramSourceMessage::CompileCliOutput(output) => {
				if let ProgramState::Compiling {
					cli_output: Some(cli_output),
				} = program
				{
					cli_output.push_str(&output);
				}
				Task::none()
			}
			ProgramSourceMessage::AssemblySourceCodeChanged(action) => {
				if let ProgramSource::Assembly(source_code_content) = self {
					let is_edit = action.is_edit();
					source_code_content.perform(action);
					if is_edit {
						*is_program_up_to_date = false;
					}
				}

				Task::none()
			}
			ProgramSourceMessage::AssemblySourceCodeUnindent => {
				if let ProgramSource::Assembly(source_code_content) = self {
					unindent_text(source_code_content);
					*is_program_up_to_date = false;
				}

				Task::none()
			}
			#[cfg(not(target_arch = "wasm32"))]
			ProgramSourceMessage::ArduinoSourceCodeChanged(action) => {
				if let ProgramSource::Arduino(source_code_content) = self {
					let is_edit = action.is_edit();
					source_code_content.perform(action);
					if is_edit {
						*is_program_up_to_date = false;
					}
				}

				Task::none()
			}
			#[cfg(not(target_arch = "wasm32"))]
			ProgramSourceMessage::ArduinoSourceCodeUnindent => {
				if let ProgramSource::Arduino(source_code_content) = self {
					unindent_text(source_code_content);
					*is_program_up_to_date = false;
				}

				Task::none()
			}
			#[cfg(not(target_arch = "wasm32"))]
			ProgramSourceMessage::BrowseArduinoCliDialog => Task::perform(
				rfd::AsyncFileDialog::new()
					.set_file_name("arduino_cli")
					.pick_file(),
				|result| match result {
					Some(file_handle) => {
						Message::SetArduinoCliPath(file_handle.path().to_path_buf())
					}
					None => ProgramSourceMessage::BrowseArduinoCliDialogCanceled.into(),
				},
			),
			#[cfg(not(target_arch = "wasm32"))]
			ProgramSourceMessage::BrowseArduinoCliDialogCanceled => Task::none(),
		}
	}

	/// (code_editor_view, extra_view)
	#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
	pub fn view<'a>(
		&'a self,
		settings: &'a Settings,
	) -> (Option<Element<'a, Message>>, Option<Element<'a, Message>>) {
		match self {
			Self::Assembly(source_code_content) => (
				Some(
					text_editor(source_code_content)
						.highlight_with::<highlighter::Highlighter>(
							highlighter::Settings {},
							highlighter::Highlight::to_format,
						)
						.style(text_editor_style)
						.on_action(|action| {
							ProgramSourceMessage::AssemblySourceCodeChanged(action).into()
						})
						.key_binding(move |key_press| {
							code_editor_keybindings(
								key_press,
								ProgramSourceMessage::AssemblySourceCodeUnindent.into(),
							)
						})
						.into(),
				),
				None,
			),
			#[cfg(not(target_arch = "wasm32"))]
			Self::Arduino(source_code_content) => (
				Some(
					text_editor(source_code_content)
						.highlight("cpp", iced::highlighter::Theme::Base16Eighties)
						.style(text_editor_style)
						.on_action(|action| {
							ProgramSourceMessage::ArduinoSourceCodeChanged(action).into()
						})
						.key_binding(move |key_press| {
							code_editor_keybindings(
								key_press,
								ProgramSourceMessage::ArduinoSourceCodeUnindent.into(),
							)
						})
						.into(),
				),
				Some({
					let path_scrollbar_padding = Padding::default().bottom(5);

					row![
						container(text("Arduino CLI Path:")).padding(path_scrollbar_padding),
						scrollable(match &settings.arduino_cli_filepath {
							Some(path) => text(path.to_string_lossy()),
							None => text("not set!").color(RED),
						})
						.width(Fill)
						.direction(Direction::Horizontal(
							Scrollbar::new().width(0).scroller_width(5).spacing(5)
						)),
						container(
							button("Browse")
								.style(button_style)
								.on_press(ProgramSourceMessage::BrowseArduinoCliDialog.into())
						)
						.padding(path_scrollbar_padding),
					]
					.spacing(10)
					.align_y(Vertical::Center)
					.into()
				}),
			),
			#[cfg(not(target_arch = "wasm32"))]
			Self::ElfFile(_) | Self::IHexFile(_) | Self::ElfFilepath(_) | Self::IHexFilepath(_) => {
				(None, None)
			}
			#[cfg(target_arch = "wasm32")]
			Self::ElfFile(_) | Self::IHexFile(_) => (None, None),
		}
	}

	/// returns 'None' if arduino cli is not set in Arduino Mode
	#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
	pub fn compile_message_maybe(&self, settings: &Settings) -> Option<Message> {
		#[cfg(not(target_arch = "wasm32"))]
		{
			if matches!(self, Self::Arduino(_)) && settings.arduino_cli_filepath.is_none() {
				return None;
			}
		}
		Some(ProgramSourceMessage::Compile.into())
	}
}
impl Clone for ProgramSource {
	fn clone(&self) -> Self {
		match self {
			Self::Assembly(source_code_content) => {
				Self::Assembly(text_editor::Content::with_text(&source_code_content.text()))
			}
			#[cfg(not(target_arch = "wasm32"))]
			Self::Arduino(source_code_content) => {
				Self::Arduino(text_editor::Content::with_text(&source_code_content.text()))
			}
			Self::ElfFile(elf_file_content) => Self::ElfFile(elf_file_content.clone()),
			#[cfg(not(target_arch = "wasm32"))]
			Self::ElfFilepath(filepath) => Self::ElfFilepath(filepath.clone()),
			Self::IHexFile(ihex_file_content) => Self::IHexFile(ihex_file_content.clone()),
			#[cfg(not(target_arch = "wasm32"))]
			Self::IHexFilepath(filepath) => Self::IHexFilepath(filepath.clone()),
		}
	}
}
