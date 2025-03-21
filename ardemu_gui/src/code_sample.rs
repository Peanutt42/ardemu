use ardemu_core::{assemble, load_elf, load_ihex_str, Program};

use crate::SourceCodeLanguage;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CodeSample {
	#[default]
	Fib8,
	Fib16,
	RecursiveFib,
	RustFibIHex,
	RustFibElf,
	BlinkLED,
	EmptyArduinoSketch,
}
impl CodeSample {
	pub const ALL: &'static [CodeSample] = &[
		CodeSample::Fib8,
		CodeSample::Fib16,
		CodeSample::RecursiveFib,
		CodeSample::RustFibIHex,
		CodeSample::RustFibElf,
		CodeSample::BlinkLED,
		CodeSample::EmptyArduinoSketch,
	];

	pub fn get_source_code(&self) -> String {
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
			Self::EmptyArduinoSketch => {
				include_str!("../../sample_programs/empty_arduino_sketch/empty_arduino_sketch.ino")
					.to_string()
			}
		}
	}

	pub fn get_language(&self) -> SourceCodeLanguage {
		match self {
			Self::EmptyArduinoSketch => SourceCodeLanguage::Arduino,
			_ => SourceCodeLanguage::Assembly,
		}
	}

	#[allow(clippy::unwrap_used)]
	pub fn get_program(&self) -> Program {
		match self {
			Self::RustFibIHex => {
				load_ihex_str(include_str!("../../sample_programs/rust_fib.hex")).unwrap()
			}
			Self::RustFibElf => {
				load_elf(include_bytes!("../../sample_programs/rust_fib.elf")).unwrap()
			}
			Self::EmptyArduinoSketch => load_elf(include_bytes!(
				"../../sample_programs/empty_arduino_sketch/empty_arduino_sketch.ino.elf"
			))
			.unwrap(),
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
				Self::EmptyArduinoSketch => "Empty Arduino Sketch",
			}
		)
	}
}
