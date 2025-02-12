use crate::{register::RegisterOrImmediate, Register};
use ardemu_display_instr_macro::DisplayInstruction;
use self_rust_tokenize::SelfRustTokenize;

#[derive(Debug, DisplayInstruction, Clone, Copy, PartialEq, Eq, SelfRustTokenize)]
pub enum Instruction {
	/// moves value into register
	Move {
		reg: Register,
		value: RegisterOrImmediate,
	},
	/// jumps relative to current instruction
	Jmp { offset: i32 },
	/// add value to register
	Add {
		reg: Register,
		value: RegisterOrImmediate,
	},
	/// store value in memory at address
	Store {
		value: RegisterOrImmediate,
		addr: u32,
	},
}
