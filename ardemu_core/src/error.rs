use std::num::ParseIntError;
use thiserror::Error;

use crate::{Imm16, Register};

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum CpuError {
	#[error("Invalid RAM address {addr}")]
	InvalidRamAddress { addr: Imm16 },
	#[error("Stack overflow")]
	StackOverflow,
	#[error("Stack underflow")]
	StackUnderflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmParseError {
	error: AsmParseErrorType,
	line_number: usize,
}
impl AsmParseError {
	pub fn new(error: AsmParseErrorType, line_number: usize) -> Self {
		Self { error, line_number }
	}
}
impl std::fmt::Display for AsmParseError {
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
	#[error("invalid cpu flag: {0}, expect 0-7")]
	InvalidCpuFlag(String),
	#[error("expected bit location (0-7), not {0}")]
	ExpectedBitLocation(String),
	#[error("invalid io address: {0}, expected 0x00-0x1F (Register IO space)")]
	InvalidRegisterIoAddress(String),
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum LoadProgramError {
	#[error("unsupported instruction: {opcode_32bit:010X} at address {program_address:04X}")]
	UnsupportedInstruction {
		opcode_32bit: u32,
		program_address: u16,
	},
	#[error("invalid alignment! must be a multiple of 2 (16-bit)")]
	InvalidAlignment,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum LoadIHexError {
	#[error("failed to parse ihex file: {0}")]
	Parse(#[from] ihex::ReaderError),
	#[error(transparent)]
	Load(#[from] LoadProgramError),
}

#[derive(Debug, Error)]
pub enum LoadElfError {
	#[error(transparent)]
	Parsing(#[from] elf::ParseError),
	#[error("could not find code section or section was invalid")]
	CouldNotFindCodeSection,
	#[error("compressed elf file not supported")]
	CompressedNotSupported,
	#[error("non zero base code address is not supported")]
	NonZeroBaseCodeAddress,
	#[error(transparent)]
	Load(#[from] LoadProgramError),
}
