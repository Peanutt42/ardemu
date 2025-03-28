use std::path::PathBuf;

use ardemu_core::{assemble, load_elf, load_ihex_str, Program};
use iced::{widget::text_editor, Element, Font};

use crate::{
	arduino_sketch::compile_arduino_sketch, code_editor::code_editor_keybindings, highlighter,
	style::text_editor_style, Message,
};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ProgramSource {
	#[default]
	Assembly,
	Arduino,
	ElfFile,
	IHexFile,
}
impl ProgramSource {
	pub const ALL: &'static [ProgramSource] =
		&[Self::Assembly, Self::Arduino, Self::ElfFile, Self::IHexFile];

	pub fn view<'a>(
		&'a self,
		assembly_source_code: &'a text_editor::Content,
		arduino_source_code: &'a text_editor::Content,
	) -> Option<Element<'a, Message>> {
		match self {
			Self::Assembly => Some(
				text_editor(assembly_source_code)
					.highlight_with::<highlighter::Highlighter>(
						highlighter::Settings {},
						highlighter::Highlight::to_format,
					)
					.font(Font::MONOSPACE)
					.style(text_editor_style)
					.on_action(Message::AssemblySourceCodeChanged)
					.key_binding(move |key_press| {
						code_editor_keybindings(key_press, Message::AssemblySourceCodeUnindent)
					})
					.into(),
			),
			Self::Arduino => Some(
				text_editor(arduino_source_code)
					.highlight("cpp", iced::highlighter::Theme::Base16Eighties)
					.font(Font::MONOSPACE)
					.style(text_editor_style)
					.on_action(Message::ArduinoSourceCodeChanged)
					.key_binding(move |key_press| {
						code_editor_keybindings(key_press, Message::ArduinoSourceCodeUnindent)
					})
					.into(),
			),
			Self::ElfFile | Self::IHexFile => None,
		}
	}

	pub fn get_source_code_text(
		&self,
		assembly_source_code_content: &text_editor::Content,
		arduino_source_code_content: &text_editor::Content,
	) -> String {
		match self {
			Self::Assembly => assembly_source_code_content.text(),
			Self::Arduino => arduino_source_code_content.text(),
			Self::ElfFile | Self::IHexFile => String::new(),
		}
	}

	pub async fn compile(
		self,
		source_code: String,
		elf_filepath: Option<PathBuf>,
		ihex_filepath: Option<PathBuf>,
		arduino_cli_filepath: Option<PathBuf>,
	) -> Result<Program, String> {
		let map_blocking_task_error =
			|e| format!("Failed to join async blocking 'compile' task: {e}");

		match self {
			Self::Assembly => tokio::task::spawn_blocking(move || {
				assemble(&source_code).map_err(|e| e.to_string())
			})
			.await
			.map_err(map_blocking_task_error)?,
			Self::Arduino => tokio::task::spawn_blocking(move || match arduino_cli_filepath {
				Some(arduino_cli_filepath) => {
					compile_arduino_sketch(&source_code, arduino_cli_filepath)
				}
				None => Err("Arduino CLI filepath not provided".to_string()),
			})
			.await
			.map_err(map_blocking_task_error)?,
			Self::ElfFile => match elf_filepath {
				Some(elf_filepath) => {
					let elf_file_content = tokio::fs::read(elf_filepath)
						.await
						.map_err(|e| e.to_string())?;
					tokio::task::spawn_blocking(move || {
						load_elf(&elf_file_content).map_err(|e| e.to_string())
					})
					.await
					.map_err(map_blocking_task_error)?
				}
				None => Err("No elf filepath provided!".to_string()),
			},
			Self::IHexFile => match ihex_filepath {
				Some(ihex_filepath) => {
					let ihex_file_content = tokio::fs::read_to_string(ihex_filepath)
						.await
						.map_err(|e| e.to_string())?;
					tokio::task::spawn_blocking(move || {
						load_ihex_str(&ihex_file_content).map_err(|e| e.to_string())
					})
					.await
					.map_err(map_blocking_task_error)?
				}
				None => Err("No ihex filepath provided!".to_string()),
			},
		}
	}
}
impl std::fmt::Display for ProgramSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Assembly => "Assembly",
				Self::Arduino => "Arduino",
				Self::ElfFile => "Elf File",
				Self::IHexFile => "IHex File",
			}
		)
	}
}
