use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum CpuError {
	#[error("Invalid RAM address {addr:#04x}")]
	InvalidRamAddress { addr: u32 },
}

#[derive(Clone, PartialEq, Eq)]
pub struct AsmParseError {
	error: AsmParseErrorType,
	line_number: usize,
}
impl AsmParseError {
	pub fn new(error: AsmParseErrorType, line_number: usize) -> Self {
		Self { error, line_number }
	}
}
impl std::fmt::Debug for AsmParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} on line {}", self.error, self.line_number)
	}
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AsmParseErrorType {
	#[error("invalid instruction: '{0}'")]
	InvalidInstruction(String),
	#[error("invalid register: '{0}', must be between r0 and r31!")]
	InvalidRegister(String),
	#[error("invalid number '{string}': {source}")]
	InvalidNumber {
		string: String,
		source: ParseIntError,
	},
	#[error("undefined label: '{0}'")]
	UndefinedLabel(String),
}
