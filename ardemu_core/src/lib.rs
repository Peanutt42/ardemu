#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

mod error;
pub use error::{
	AsmParseError, AsmParseErrorType, CpuError, LoadElfError, LoadIHexError, LoadProgramError,
};

mod cpu;
pub use cpu::{Cpu, CpuStatus};

mod instruction;
pub use instruction::{Instruction, MemoryAddressRange};

mod program;
pub use program::Program;

mod opcode;
pub use opcode::Opcode;

mod ihex;
pub use ihex::load_ihex_str;

mod elf;
pub use elf::load_elf;

mod register;
pub use register::{
	Imm16, Imm3, Imm8, LowerEvenRegister, PointerRegister, PointerRegisterAction,
	Register::{
		self, R0, R1, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R2, R20, R21, R22, R23,
		R24, R25, R26, R27, R28, R29, R30, R31, R4, R5, R6, R7, R8, R9,
	},
	RegisterAddress, UpperRegister, WordAddress, WordOffset16, WordOffset8, WordRegister,
};

mod bits;
pub use bits::{
	get_bit_from_u16, get_bit_from_u8, set_bit_in_u16, set_bit_in_u8, u8s_from_u16, u8s_to_u16,
};

mod flags;
pub use flags::{FlagType, Flags};

mod assembler;
pub use assembler::{assemble, parse_number_operand, AsmOperand, ParseAsmInstruction};
