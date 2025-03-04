#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

mod error;
pub use error::{AsmParseError, AsmParseErrorType, CpuError, LoadIHex};

mod cpu;
pub use cpu::{Cpu, CpuStatus};

mod instruction;
pub use instruction::Instruction;

mod opcode;
pub use opcode::Opcode;

mod ihex;
pub use ihex::load_ihex_str;

mod register;
pub use register::{
	Imm16, Imm3, Imm8, LowerEvenRegister,
	Register::{
		self, R0, R1, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R2, R20, R21, R22, R23,
		R24, R25, R26, R27, R28, R29, R30, R31, R4, R5, R6, R7, R8, R9,
	},
	RegisterAddress, UpperRegister, WordRegister,
};

mod bits;
pub use bits::{get_bit_from_u8, set_bit_in_u8, u8s_from_u16, u8s_to_u16};

mod flags;
pub use flags::{FlagType, Flags};

mod assembler;
pub use assembler::{assemble, parse_number_operand, AsmOperand, ParseAsmInstruction};
