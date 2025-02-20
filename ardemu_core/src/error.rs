use std::num::ParseIntError;
use thiserror::Error;

use crate::Register;

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum CpuError {
	#[error("Invalid RAM address {addr:#04x}")]
	InvalidRamAddress { addr: u16 },
	#[error("Stack overflow")]
	StackOverflow,
	#[error("Stack underflow")]
	StackUnderflow,
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
	#[error("invalid argument count, expected {expected_count}, got {actual_count}")]
	InvalidArgumentCount {
		expected_count: usize,
		actual_count: usize,
	},
	#[error("invalid argument count, expected any of {allowed_counts:?}, got {actual_count}")]
	InvalidDynamicArgumentCount {
		allowed_counts: Vec<usize>,
		actual_count: usize,
	},
	#[error("invalid instruction: '{0}'")]
	InvalidInstruction(String),
	#[error("invalid register: '{0}'")]
	InvalidRegister(String),
	#[error("invalid number '{string}': {source}")]
	InvalidNumber {
		string: String,
		source: ParseIntError,
	},
	#[error("undefined label: '{0}'")]
	UndefinedLabel(String),
	#[error("expected upper registers (r16-r31), not {0}")]
	ExpectedUpperRegister(Register),
	#[error("expected word registers (r24, r26, r28, r30), not {0}")]
	ExpectedWordRegister(Register),
	#[error("Invalid low register for 16 bit value register pair cannot be r31, as there is no r32 to be the high register")]
	InvalidRegisterPairLowRegister,
}
