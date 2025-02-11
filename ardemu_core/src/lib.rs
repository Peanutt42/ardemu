#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

mod error;
pub use error::{AsmParseError, AsmParseErrorType, CpuError};

mod cpu;
pub use cpu::Cpu;

mod instruction;
pub use instruction::Instruction;

mod register;
pub use register::Register;

mod asm_parser;
pub use asm_parser::parse_asm;
