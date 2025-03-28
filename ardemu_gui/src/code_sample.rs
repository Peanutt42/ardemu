use ardemu_core::{assemble, load_elf, load_ihex_str, Program};

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
			Self::ArduinoBlinkSketch => {
				include_str!("../../sample_programs/arduino_blink_sketch/arduino_blink_sketch.ino")
					.to_string()
			}
		}
	}

	pub fn get_language(&self) -> ProgramSource {
		match self {
			Self::ArduinoBlinkSketch => ProgramSource::Arduino,
			_ => ProgramSource::Assembly,
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
			Self::ArduinoBlinkSketch => load_elf(include_bytes!(
				"../../sample_programs/arduino_blink_sketch/arduino_blink_sketch.ino.elf"
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
				Self::ArduinoBlinkSketch => "Arduino Blink Sketch",
			}
		)
	}
}
