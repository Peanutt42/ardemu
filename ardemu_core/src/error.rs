use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum CpuError {
	#[error("Invalid register r{reg}")]
	InvalidRegister { reg: usize },
	#[error("Invalid RAM address {addr:#04x}")]
	InvalidRamAddress { addr: usize },
}

#[derive(Clone, PartialEq, Eq)]
pub struct AsmParseError {
	error: AsmParseErrorType,
	line_number: usize,
	line: String,
}
impl AsmParseError {
	pub fn new(error: AsmParseErrorType, line_number: usize, line: String) -> Self {
		Self {
			error,
			line_number,
			line,
		}
	}
}
impl std::fmt::Debug for AsmParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{} on line {}: '{}'",
			self.error, self.line_number, self.line,
		)
	}
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AsmParseErrorType {
	#[error("invalid instruction")]
	InvalidInstruction,
	#[error("invalid number '{string}': {source}")]
	InvalidNumber {
		string: String,
		source: ParseIntError,
	},
	#[error("undefined label: {0}")]
	UndefinedLabel(String),
}
