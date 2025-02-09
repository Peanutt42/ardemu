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
	line_number: usize,
	error: AsmParseErrorType,
}
impl AsmParseError {
	pub fn new(line_number: usize, error: AsmParseErrorType) -> Self {
		Self { line_number, error }
	}
}
impl std::fmt::Debug for AsmParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "line {}: {}", self.line_number, self.error)
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
