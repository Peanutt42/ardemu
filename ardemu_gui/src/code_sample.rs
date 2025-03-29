use iced::widget::text_editor;

use crate::ProgramSource;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CodeSample {
	#[default]
	Fib8,
	Fib16,
	RecursiveFib,
	RustFibIHex,
	RustFibElf,
	BlinkLED,
	ArduinoBlinkSketch,
}
impl CodeSample {
	pub const ALL: &'static [CodeSample] = &[
		CodeSample::Fib8,
		CodeSample::Fib16,
		CodeSample::RecursiveFib,
		CodeSample::RustFibIHex,
		CodeSample::RustFibElf,
		CodeSample::BlinkLED,
		CodeSample::ArduinoBlinkSketch,
	];

	pub fn get_program_source(&self) -> ProgramSource {
		match self {
			Self::Fib8 => {
				let source_code = format!(
					"ldi r16, 10 ; n = 10\n\n{}",
					include_str!("../../sample_programs/fib.asm")
				);
				ProgramSource::Assembly(text_editor::Content::with_text(&source_code))
			}
			Self::Fib16 => {
				let source_code = format!(
					"ldi r16, 10 ; n = 10\n\n{}",
					include_str!("../../sample_programs/fib16.asm")
				);
				ProgramSource::Assembly(text_editor::Content::with_text(&source_code))
			}
			Self::RecursiveFib => {
				let source_code = format!(
					"ldi r16, 10 ; n = 10\n\n{}",
					include_str!("../../sample_programs/recursive_fib.asm")
				);
				ProgramSource::Assembly(text_editor::Content::with_text(&source_code))
			}
			Self::BlinkLED => {
				let source_code = include_str!("../../sample_programs/blink.asm");
				ProgramSource::Assembly(text_editor::Content::with_text(source_code))
			}
			Self::RustFibIHex => ProgramSource::IHexFile(
				include_str!("../../sample_programs/rust_fib.hex").to_string(),
			),
			Self::RustFibElf => ProgramSource::ElfFile(
				include_bytes!("../../sample_programs/rust_fib.elf").to_vec(),
			),
			Self::ArduinoBlinkSketch => ProgramSource::ElfFile(
				include_bytes!(
					"../../sample_programs/arduino_blink_sketch/arduino_blink_sketch.ino.elf"
				)
				.to_vec(),
			),
		}
	}

	/*#[allow(clippy::unwrap_used)]
	pub fn get_program(&self) -> Program {
		match self {
			Self::RustFibIHex => {
				load_ihex_str(include_str!("../../sample_programs/rust_fib.hex")).unwrap()
			}
			Self::RustFibElf => {
				load_elf(include_bytes!("../../sample_programs/rust_fib.elf")).unwrap()
			}
			Self::ArduinoBlinkSketch => load_elf(include_bytes!(
				"../../sample_programs/arduino_blink_sketch/arduino_blink_sketch.ino.elf"
			))
			.unwrap(),
			_ => assemble(&self.get_source_code()).unwrap(),
		}
	}*/
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
				Self::ArduinoBlinkSketch => "Arduino Blink Sketch",
			}
		)
	}
}
