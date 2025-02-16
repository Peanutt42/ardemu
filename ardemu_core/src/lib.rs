#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

mod error;
pub use error::{AsmParseError, AsmParseErrorType, CpuError};

mod cpu;
pub use cpu::{Cpu, CpuStatus};

mod instruction;
pub use instruction::Instruction;

mod register;
pub use register::{
	Imm16,
	Register::{self, A, B, C, D, F, H, L, Z},
	RegisterOrImm8,
};

mod asm_parser;
pub use asm_parser::parse_asm;
