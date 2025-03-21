use std::path::PathBuf;

use ardemu_core::{assemble, Program};
use iced::{widget::text_editor, Element, Font};

use crate::{
	arduino_sketch::compile_arduino_sketch, code_editor::code_editor_keybindings, highlighter,
	style::text_editor_style, Message,
};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SourceCodeLanguage {
	#[default]
	Assembly,
	Arduino,
}
impl SourceCodeLanguage {
	pub const ALL: &'static [SourceCodeLanguage] = &[Self::Assembly, Self::Arduino];

	pub fn editor_view<'a>(
		&'a self,
		source_code: &'a text_editor::Content,
	) -> Element<'a, Message> {
		match self {
			Self::Assembly => text_editor(source_code)
				.highlight_with::<highlighter::Highlighter>(
					highlighter::Settings {},
					highlighter::Highlight::to_format,
				)
				.font(Font::MONOSPACE)
				.style(text_editor_style)
				.on_action(Message::SourceCodeChanged)
				.key_binding(move |key_press| {
					code_editor_keybindings(key_press, Message::SourceCodeUnindent)
				})
				.into(),
			Self::Arduino => text_editor(source_code)
				.highlight("cpp", iced::highlighter::Theme::Base16Eighties)
				.font(Font::MONOSPACE)
				.style(text_editor_style)
				.on_action(Message::SourceCodeChanged)
				.key_binding(move |key_press| {
					code_editor_keybindings(key_press, Message::SourceCodeUnindent)
				})
				.into(),
		}
	}

	pub async fn compile(
		self,
		source_code: String,
		arduino_cli_filepath: Option<PathBuf>,
	) -> Result<Program, String> {
		let result = tokio::task::spawn_blocking(move || match self {
			Self::Assembly => assemble(&source_code).map_err(|e| e.to_string()),
			Self::Arduino => match arduino_cli_filepath {
				Some(arduino_cli_filepath) => {
					compile_arduino_sketch(&source_code, arduino_cli_filepath)
				}
				None => Err("Arduino CLI filepath not provided".to_string()),
			},
		})
		.await;
		match result {
			Ok(result) => result,
			Err(e) => Err(format!("Failed to join async blocking 'compile' task: {e}")),
		}
	}
}
impl std::fmt::Display for SourceCodeLanguage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Assembly => "Assembly",
				Self::Arduino => "Arduino",
			}
		)
	}
}
