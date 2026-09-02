#![deny(arithmetic_overflow)]
#![deny(clippy::checked_conversions)]
#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::cast_sign_loss)]
#![deny(clippy::cast_possible_wrap)]
#![deny(clippy::cast_precision_loss)]
#![deny(clippy::unchecked_time_subtraction)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::panicking_unwrap)]
#![deny(clippy::option_env_unwrap)]
#![deny(unused_must_use)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::join_absolute_paths)]
#![deny(clippy::serde_api_misuse)]
#![deny(clippy::uninit_vec)]
#![deny(unsafe_code)]
#![deny(unnecessary_transmutes)]
#![deny(clippy::transmute_ptr_to_ref)]
#![deny(clippy::transmute_undefined_repr)]

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
	Imm16, Imm3, Imm8, LPMZPointerRegisterAction, LowerEvenRegister, PointerRegister,
	PointerRegisterAction,
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
