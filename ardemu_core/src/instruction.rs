use crate::{
	register::{HlOrImm16, Imm16, RegisterOrImm8},
	Register,
};
use ardemu_display_instr_macro::DisplayInstruction;
use self_rust_tokenize::SelfRustTokenize;

#[derive(Debug, DisplayInstruction, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub enum Instruction {
	/// moves value into register
	Mw {
		reg: Register,
		value: RegisterOrImm8,
	},
	/// load value in address into register
	Lw {
		register: Register,
		address: HlOrImm16,
	},
	/// stores value of register into address
	Sw {
		address: HlOrImm16,
		register: Register,
	},
	/// pushes value onto stack
	Push { value: RegisterOrImm8 },
	/// pops value from stack into register
	Pop { register: Register },
	/// sets HL to address
	Lda { address: Imm16 },
	/// jumps to HL if value != 0
	Jnz { value: RegisterOrImm8 },
	/// add value to register
	Add {
		reg: Register,
		value: RegisterOrImm8,
	},
	/// sub value to register
	Sub {
		reg: Register,
		value: RegisterOrImm8,
	},
	/// performs and with value and sets result to register
	And {
		reg: Register,
		value: RegisterOrImm8,
	},
	/// performs or with value and sets result to register
	Or {
		reg: Register,
		value: RegisterOrImm8,
	},
	/// performs nor with value and sets result to register
	Nor {
		reg: Register,
		value: RegisterOrImm8,
	},
}
